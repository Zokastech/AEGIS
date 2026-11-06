// AEGIS — zokastech.fr — Apache 2.0 / MIT

package config

import (
	"os"
	"path/filepath"
	"testing"
)

func write(t *testing.T, body string) string {
	t.Helper()
	p := filepath.Join(t.TempDir(), "c.yaml")
	if err := os.WriteFile(p, []byte(body), 0o600); err != nil {
		t.Fatal(err)
	}
	return p
}

func TestLoadDefaults(t *testing.T) {
	c, err := Load(write(t, "upstream_url: https://api.openai.com\n"))
	if err != nil {
		t.Fatal(err)
	}
	if c.Mode != ModeAnonymize || c.MaxBodyBytes != 10<<20 || c.FailOpen || c.Engine.Type != EngineHTTP {
		t.Fatalf("défauts inattendus: %+v", c)
	}
}

func TestLoadRequiresUpstream(t *testing.T) {
	if _, err := Load(write(t, "mode: block\n")); err == nil {
		t.Fatal("upstream_url manquant accepté")
	}
}

func TestUnknownModeRejected(t *testing.T) {
	if _, err := Load(write(t, "upstream_url: https://x\nmode: yolo\n")); err == nil {
		t.Fatal("mode inconnu accepté (se comportait comme transparent)")
	}
	t.Setenv("AEGIS_LLM_MODE", "nope")
	if _, err := Load(write(t, "upstream_url: https://x\n")); err == nil {
		t.Fatal("mode inconnu via env accepté")
	}
}

func TestEnvOverrides(t *testing.T) {
	t.Setenv("AEGIS_UPSTREAM_URL", "https://override")
	t.Setenv("AEGIS_ENGINE_URL", "http://engine:9/")
	t.Setenv("AEGIS_LLM_MODE", "block")
	c, err := Load(write(t, "upstream_url: https://x\n"))
	if err != nil {
		t.Fatal(err)
	}
	if c.UpstreamURL != "https://override" || c.Engine.BaseURL != "http://engine:9" || c.Mode != ModeBlock {
		t.Fatalf("overrides: %+v", c)
	}
}
