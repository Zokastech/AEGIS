// AEGIS — zokastech.fr — Apache 2.0 / MIT

package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"net"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/rs/zerolog/log"
	"github.com/spf13/pflag"
	"github.com/zokastech/aegis/aegis-gateway/api/grpc/grpcapi"
	"github.com/zokastech/aegis/aegis-gateway/api/rest"
	"github.com/zokastech/aegis/aegis-gateway/auth"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/config"
	"github.com/zokastech/aegis/aegis-gateway/health"
	"github.com/zokastech/aegis/aegis-gateway/metrics"
	"github.com/zokastech/aegis/aegis-gateway/middleware"
	"github.com/zokastech/aegis/aegis-gateway/policy"
	"github.com/zokastech/aegis/aegis-gateway/security"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/reflection"
)

func main() {
	middleware.SetGlobalLogLevel(os.Getenv("AEGIS_LOG_LEVEL"))

	ld := config.NewLoader()
	ld.BindFlags(pflag.CommandLine)
	cfg, err := ld.Load(pflag.CommandLine)
	if err != nil {
		log.Fatal().Err(err).Msg("config")
	}
	insecureHTTP := os.Getenv("AEGIS_INSECURE_HTTP") == "1"
	if err := cfg.Security.Validate(insecureHTTP); err != nil {
		log.Fatal().Err(err).Msg("configuration sécurité refusée")
	}
	if cfg.Security.Development.DisableAuth {
		log.Warn().Msg("security: development.disable_auth=true — AUCUNE authentification (dev uniquement)")
	}

	factory := newEngineFactory(cfg)
	pool := bridge.NewPool(cfg.PoolSize, cfg.EngineTimeout, factory)
	breaker := bridge.NewDefaultBreaker()

	gw, err := rest.BootstrapGatewaySecurity(cfg)
	if err != nil {
		log.Fatal().Err(err).Msg("bootstrap sécurité gateway")
	}

	mapStore := policy.NewMemoryMappingStore()
	polEngine, err := policy.DefaultEngine(cfg.PolicyDir, mapStore)
	if err != nil {
		log.Fatal().Err(err).Msg("chargement politiques conformité")
	}

	svc := &rest.Services{
		Pool:          pool,
		Breaker:       breaker,
		EngineTimeout: cfg.EngineTimeout,
		Loader:        ld,
		Gateway:       gw,
		Policy:        polEngine,
	}
	svc.AnalyzeUC = rest.NewAnalyzeUseCase(svc)

	readiness := func(ctx context.Context) error {
		return svc.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
			if strings.TrimSpace(eng.Version()) == "" {
				return fmt.Errorf("moteur: version vide")
			}
			return eng.NerOnnxReady()
		})
	}
	svc.Readiness = readiness

	ld.OnChange(func(nc config.Config) {
		log.Info().Interface("http_listen", nc.HTTPListen).Msg("config hot-reload")
	})

	e := echo.New()
	e.HideBanner = true
	ipx, err := middleware.TrustedProxyIPExtractor(cfg.Security.TrustedProxies)
	if err != nil {
		log.Fatal().Err(err).Msg("security.trusted_proxies")
	}
	e.IPExtractor = ipx
	e.Use(middleware.BodyLimit(cfg.Security.MaxBodyBytes))
	e.Use(middleware.RequestID())
	e.Use(middleware.ZerologLogger())
	e.Use(middleware.RecoverPanic())
	e.Use(middleware.Gzip())

	useTLS := cfg.Security.TLS.Enabled && !insecureHTTP
	tlsActive := useTLS
	e.Use(middleware.SecurityHeaders(tlsActive))
	e.Use(middleware.CORSFromSecurity(cfg.Security.Production, cfg.Security.CORS.AllowOrigins))
	e.Use(middleware.RateLimitIP(cfg.RateLimitRPS, cfg.RateLimitBurst))
	e.Use(middleware.RateLimitAPIKey(cfg.APIKeyHeader, cfg.RateLimitRPS*2, cfg.RateLimitBurst))
	e.Use(func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			metrics.ActiveConnections.Inc()
			defer metrics.ActiveConnections.Dec()
			return next(c)
		}
	})

	health.Attach(e, health.Opts{PingEngine: readiness})
	go waitStartup(context.Background(), readiness)

	rest.AttachRoutes(e, svc, cfg)

	useTLSListener := useTLS
	var tlsCfg *tls.Config
	if useTLSListener {
		cert, err := security.EnsureServerCertificate(
			cfg.Security.TLS.CertFile,
			cfg.Security.TLS.KeyFile,
			cfg.Security.TLS.AutoGenCertDir,
			cfg.Security.TLS.AutoGenerateSelfSigned,
			tlsSANs(cfg.HTTPListen),
		)
		if err != nil {
			log.Fatal().Err(err).Msg("tls certificat serveur")
		}
		tlsCfg = &tls.Config{
			Certificates: []tls.Certificate{cert},
			MinVersion:   tls.VersionTLS12,
		}
		mtlsRequire := cfg.Security.MTLS.Require && !cfg.Security.Development.DisableAuth
		clientPart, err := auth.BuildTLSClientAuth(cfg.Security.TLS.ClientCAFile, mtlsRequire)
		if err != nil {
			// Fail closed: never start without the client verification the operator asked for.
			log.Fatal().Err(err).Msg("mTLS client CA")
		}
		if clientPart != nil {
			tlsCfg.ClientAuth = clientPart.ClientAuth
			tlsCfg.ClientCAs = clientPart.ClientCAs
		}
	}

	authn := &grpcapi.Authenticator{
		Gateway:         gw,
		APIKeyHeader:    cfg.APIKeyHeader,
		AllowReflection: cfg.Security.GRPC.Reflection,
	}
	maxRecv := cfg.Security.GRPC.MaxRecvMsgBytes
	if maxRecv <= 0 {
		maxRecv = 10 << 20
	}
	grpcOpts := []grpc.ServerOption{
		grpc.ChainUnaryInterceptor(authn.Unary()),
		grpc.ChainStreamInterceptor(authn.Stream()),
		grpc.MaxRecvMsgSize(maxRecv),
	}
	if tlsCfg != nil {
		grpcOpts = append(grpcOpts, grpc.Creds(credentials.NewTLS(tlsCfg.Clone())))
	} else {
		log.Warn().Msg("grpc: listener en clair (TLS désactivé) — réservé au développement local")
	}
	grpcS := grpc.NewServer(grpcOpts...)
	grpcapi.RegisterAegisGatewayServer(grpcS, &grpcapi.Server{Services: svc})
	if cfg.Security.GRPC.Reflection {
		reflection.Register(grpcS)
	}

	glis, err := net.Listen("tcp", cfg.GRPCListen)
	if err != nil {
		log.Fatal().Err(err).Str("addr", cfg.GRPCListen).Msg("grpc listen")
	}
	go func() {
		log.Info().Str("addr", cfg.GRPCListen).Bool("tls", tlsCfg != nil).Bool("reflection", cfg.Security.GRPC.Reflection).Msg("grpc")
		if err := grpcS.Serve(glis); err != nil {
			log.Error().Err(err).Msg("grpc serve")
		}
	}()

	go func() {
		e.Server.ReadTimeout = cfg.ReadTimeout
		e.Server.WriteTimeout = cfg.WriteTimeout
		e.Server.ReadHeaderTimeout = 10 * time.Second
		e.Server.IdleTimeout = 120 * time.Second
		e.Server.MaxHeaderBytes = 64 << 10
		if tlsCfg != nil {
			ln, err := tls.Listen("tcp", cfg.HTTPListen, tlsCfg)
			if err != nil {
				log.Fatal().Err(err).Str("addr", cfg.HTTPListen).Msg("tls listen")
			}
			e.Listener = ln
			log.Info().Str("addr", cfg.HTTPListen).Bool("tls", true).Bool("mtls", tlsCfg.ClientAuth == tls.RequireAndVerifyClientCert).Msg("https")
			if err := e.Start(""); err != nil {
				log.Error().Err(err).Msg("https")
			}
			return
		}
		log.Info().Str("addr", cfg.HTTPListen).Bool("tls", false).Msg("http")
		if err := e.Start(cfg.HTTPListen); err != nil {
			log.Error().Err(err).Msg("http")
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	ctx, cancel := context.WithTimeout(context.Background(), cfg.ShutdownTimeout)
	defer cancel()
	grpcS.GracefulStop()
	_ = glis.Close()
	if err := e.Shutdown(ctx); err != nil {
		log.Error().Err(err).Msg("http shutdown")
	}
	time.Sleep(50 * time.Millisecond)
}

func waitStartup(ctx context.Context, ping func(context.Context) error) {
	deadline, cancel := context.WithTimeout(ctx, 120*time.Second)
	defer cancel()
	tick := time.NewTicker(2 * time.Second)
	defer tick.Stop()
	for {
		select {
		case <-deadline.Done():
			log.Warn().Msg("health: startup — timeout sans contact moteur (startup probe reste en échec)")
			return
		case <-tick.C:
			pctx, pcancel := context.WithTimeout(context.Background(), 5*time.Second)
			err := ping(pctx)
			pcancel()
			if err == nil {
				health.MarkStartupOK()
				log.Info().Msg("health: startup — moteur joignable")
				return
			}
		}
	}
}

// tlsSANs returns SANs for the self-signed certificate: listen host (or localhost)
// plus AEGIS_TLS_EXTRA_SANS (comma-separated, e.g. "aegis-gateway,gateway.example.org").
func tlsSANs(addr string) []string {
	out := []string{"localhost"}
	if host, _, err := net.SplitHostPort(addr); err == nil {
		if host != "" && host != "0.0.0.0" && host != "::" && host != "[::]" {
			out = []string{host}
		}
	}
	for _, h := range strings.Split(os.Getenv("AEGIS_TLS_EXTRA_SANS"), ",") {
		if h = strings.TrimSpace(h); h != "" {
			out = append(out, h)
		}
	}
	return out
}
