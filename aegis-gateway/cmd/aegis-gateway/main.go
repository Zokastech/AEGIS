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
	if cfg.Security.MTLS.Require && cfg.Security.TLS.Enabled && !cfg.Security.Development.DisableAuth && !insecureHTTP {
		if cfg.Security.TLS.ClientCAFile == "" {
			log.Fatal().Msg("security: mtls.require=true impose tls.client_ca_file (ou development.disable_auth / AEGIS_INSECURE_HTTP=1 en local)")
		}
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

	ld.OnChange(func(nc config.Config) {
		log.Info().Interface("http_listen", nc.HTTPListen).Msg("config hot-reload")
	})

	e := echo.New()
	e.HideBanner = true
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

	pingEngine := func(ctx context.Context) error {
		return svc.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
			if strings.TrimSpace(eng.Version()) == "" {
				return fmt.Errorf("moteur: version vide")
			}
			return nil
		})
	}
	health.Attach(e, health.Opts{PingEngine: pingEngine})
	go waitStartup(context.Background(), pingEngine)

	rest.AttachRoutes(e, svc, cfg)

	grpcS := grpc.NewServer()
	grpcapi.RegisterAegisGatewayServer(grpcS, &grpcapi.Server{Services: svc})
	reflection.Register(grpcS)

	glis, err := net.Listen("tcp", cfg.GRPCListen)
	if err != nil {
		log.Fatal().Err(err).Str("addr", cfg.GRPCListen).Msg("grpc listen")
	}
	go func() {
		log.Info().Str("addr", cfg.GRPCListen).Msg("grpc")
		if err := grpcS.Serve(glis); err != nil {
			log.Error().Err(err).Msg("grpc serve")
		}
	}()

	go func() {
		e.Server.ReadTimeout = cfg.ReadTimeout
		e.Server.WriteTimeout = cfg.WriteTimeout
		if useTLS {
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
			tlsCfg := &tls.Config{
				Certificates: []tls.Certificate{cert},
				MinVersion:   tls.VersionTLS12,
			}
			mtlsRequire := cfg.Security.MTLS.Require && !cfg.Security.Development.DisableAuth
			if clientPart, err := auth.BuildTLSClientAuth(cfg.Security.TLS.ClientCAFile, mtlsRequire); err != nil {
				log.Warn().Err(err).Msg("mTLS client CA")
			} else if clientPart != nil {
				tlsCfg.ClientAuth = clientPart.ClientAuth
				tlsCfg.ClientCAs = clientPart.ClientCAs
			}
			ln, err := tls.Listen("tcp", cfg.HTTPListen, tlsCfg)
			if err != nil {
				log.Fatal().Err(err).Str("addr", cfg.HTTPListen).Msg("tls listen")
			}
			e.Listener = ln
			log.Info().Str("addr", cfg.HTTPListen).Bool("tls", true).Bool("mtls", mtlsRequire).Msg("https")
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

func tlsSANs(addr string) []string {
	host, _, err := net.SplitHostPort(addr)
	if err != nil {
		return []string{"localhost"}
	}
	if host == "" || host == "0.0.0.0" || host == "::" || host == "[::]" {
		return []string{"localhost"}
	}
	return []string{host}
}
