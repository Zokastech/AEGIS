// AEGIS — zokastech.fr — Apache 2.0 / MIT

package main

import (
	"flag"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/zokastech/aegis/aegis-llm-proxy/internal/config"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/engine"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/proxy"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/stats"
)

func main() {
	cfgPath := flag.String("config", "config.yaml", "chemin fichier YAML")
	flag.Parse()

	cfg, err := config.Load(*cfgPath)
	if err != nil {
		log.Fatalf("config: %v", err)
	}
	if cfg.Engine.Type != config.EngineHTTP {
		log.Fatalf("seul engine.type=http est supporté pour l’instant")
	}

	timeout := time.Duration(cfg.Engine.TimeoutSeconds) * time.Second
	eng, err := engine.NewHTTPWithTLS(cfg.Engine.BaseURL, timeout, cfg.Engine.CAFile, os.Getenv(cfg.Engine.APIKeyEnv))
	if err != nil {
		log.Fatalf("engine: %v", err)
	}
	if eng.APIKey == "" {
		log.Printf("avertissement : %s vide — la passerelle AEGIS v0.2.0 exige une clé API (rôle operator)", cfg.Engine.APIKeyEnv)
	}
	st := &stats.Registry{}
	h, err := proxy.New(cfg, eng, st)
	if err != nil {
		log.Fatalf("proxy: %v", err)
	}

	addr := cfg.Listen
	if p := os.Getenv("AEGIS_LLM_LISTEN"); p != "" {
		addr = p
	}
	log.Printf("AEGIS LLM proxy écoute %s → amont %s (mode=%s)", addr, cfg.UpstreamURL, cfg.Mode)
	srv := &http.Server{
		Addr:              addr,
		Handler:           h,
		ReadHeaderTimeout: 10 * time.Second,
		ReadTimeout:       2 * time.Minute,
		IdleTimeout:       2 * time.Minute,
		MaxHeaderBytes:    64 << 10,
		// No WriteTimeout: LLM streaming answers can legitimately last several minutes.
	}
	if err := srv.ListenAndServe(); err != nil {
		log.Fatal(err)
	}
}
