// AEGIS — zokastech.fr — Apache 2.0 / MIT

package config

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/spf13/pflag"
	"github.com/spf13/viper"
)

func TestValidate(t *testing.T) {
	ok := DefaultSecurity()
	if err := ok.Validate(false); err != nil {
		t.Fatalf("défaut refusé: %v", err)
	}
	c := DefaultSecurity()
	c.Development.DisableAuth = true
	if err := c.Validate(false); err == nil {
		t.Fatal("disable_auth accepté en production")
	}
	c.Production = false
	if err := c.Validate(false); err != nil {
		t.Fatalf("disable_auth en dev refusé: %v", err)
	}
	c = DefaultSecurity()
	c.MTLS.Require = true
	if err := c.Validate(false); err == nil {
		t.Fatal("mtls sans CA accepté")
	}
	if err := c.Validate(true); err != nil {
		t.Fatalf("mtls ignoré en HTTP clair: %v", err)
	}
	c = DefaultSecurity()
	c.JWT.Enabled = true
	c.JWT.HMACSecret = "short"
	if err := c.Validate(false); err == nil {
		t.Fatal("secret JWT court accepté")
	}
}

func TestDefaultSecurityHardening(t *testing.T) {
	d := DefaultSecurity()
	if !d.Production || !d.TLS.Enabled || d.Development.DisableAuth || d.GRPC.Reflection || d.MaxBodyBytes <= 0 {
		t.Fatalf("défauts non sûrs: %+v", d)
	}
}

func TestMergeSecurityReflectionDefaultsToNonProd(t *testing.T) {
	v := viper.New()
	v.Set("security.production", false)
	out := MergeSecurityConfig(DefaultSecurity(), SecurityConfig{Production: false}, v)
	if !out.GRPC.Reflection {
		t.Fatal("réflexion attendue en dev")
	}
	v = viper.New()
	out = MergeSecurityConfig(DefaultSecurity(), SecurityConfig{}, v)
	if out.GRPC.Reflection {
		t.Fatal("réflexion active en production par défaut")
	}
	v = viper.New()
	v.Set("security.trusted_proxies", []string{"10.0.0.0/8"})
	out = MergeSecurityConfig(DefaultSecurity(), SecurityConfig{TrustedProxies: []string{"10.0.0.0/8"}, MaxBodyBytes: 42}, v)
	if len(out.TrustedProxies) != 1 || out.MaxBodyBytes != 42 {
		t.Fatalf("merge: %+v", out)
	}
}

func TestRejectProtectedKeys(t *testing.T) {
	for _, y := range []string{"security: {}\n", "Security:\n  production: false\n", "admin_api_keys: [a]\n", "http_listen: x\n"} {
		if err := rejectProtectedKeys([]byte(y)); err == nil {
			t.Errorf("%q accepté", y)
		}
	}
	if err := rejectProtectedKeys([]byte("pool_size: 3\nrate_limit_rps: 5\n")); err != nil {
		t.Fatalf("clés autorisées refusées: %v", err)
	}
	if err := rejectProtectedKeys([]byte("a: [\n")); err == nil || !strings.Contains(err.Error(), "yaml") {
		t.Fatalf("YAML invalide: %v", err)
	}
}

func TestExplicitSecurityEnv(t *testing.T) {
	t.Setenv("AEGIS_SECURITY_PRODUCTION", "false")
	t.Setenv("AEGIS_SECURITY_DEVELOPMENT_DISABLE_AUTH", "true")
	c := Config{Security: DefaultSecurity()}
	applyExplicitSecurityEnv(&c)
	if c.Security.Production || !c.Security.Development.DisableAuth {
		t.Fatalf("env: %+v", c.Security)
	}
}

// Regression: security files use top-level keys and were merged at the root, so every
// setting (mTLS, CORS, JWT, keys path…) was silently ignored before v0.2.0.
func TestSecurityConfigFileIsApplied(t *testing.T) {
	dir := t.TempDir()
	p := filepath.Join(dir, "sec.yaml")
	if err := os.WriteFile(p, []byte("production: false\napi_keys:\n  file_path: /tmp/k.yaml\nmtls:\n  require: true\ntls:\n  client_ca_file: /ca.pem\ncors:\n  allow_origins: [\"https://a.example\"]\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	ld := NewLoader()
	fs := pflag.NewFlagSet("t", pflag.ContinueOnError)
	ld.BindFlags(fs)
	oldArgs := os.Args
	os.Args = []string{"aegis-gateway", "--security-config", p}
	defer func() { os.Args = oldArgs }()
	cfg, err := ld.Load(fs)
	if err != nil {
		t.Fatal(err)
	}
	s := cfg.Security
	if s.Production || s.APIKeys.FilePath != "/tmp/k.yaml" || !s.MTLS.Require || s.TLS.ClientCAFile != "/ca.pem" || len(s.CORS.AllowOrigins) != 1 {
		t.Fatalf("fichier sécurité ignoré: %+v", s)
	}
}

func TestSecurityConfigAlreadyWrapped(t *testing.T) {
	ld := NewLoader()
	if err := ld.mergeSecurityYAML([]byte("security:\n  production: false\n")); err != nil {
		t.Fatal(err)
	}
	if ld.v.GetBool("security.production") || !ld.v.IsSet("security.production") {
		t.Fatal("fichier déjà encapsulé mal fusionné")
	}
	if err := ld.mergeSecurityYAML([]byte("a: [\n")); err == nil {
		t.Fatal("YAML invalide accepté")
	}
}
