// AEGIS — zokastech.fr — Apache 2.0 / MIT

package main

import (
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/zokastech/aegis/aegis-gateway/auth/apikey"
)

func captureStdout(t *testing.T, fn func() error) (string, error) {
	t.Helper()
	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w
	err := fn()
	_ = w.Close()
	os.Stdout = old
	b, _ := io.ReadAll(r)
	return strings.TrimSpace(string(b)), err
}

func TestKeygenLifecycle(t *testing.T) {
	t.Setenv("AEGIS_API_KEY_PEPPER", "pepper-for-tests")
	f := filepath.Join(t.TempDir(), "keys.yaml")

	secret, err := captureStdout(t, func() error { return run([]string{"-file", f, "-role", "admin", "-id", "ops"}) })
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(secret, "aegis_") || len(secret) < 40 {
		t.Fatalf("secret faible ou mal formé: %q", secret)
	}
	raw, _ := os.ReadFile(f)
	if strings.Contains(string(raw), secret) || strings.Contains(string(raw), "pepper-for-tests") {
		t.Fatal("secret ou pepper écrit en clair")
	}
	st, _ := apikey.LoadFileStore(f, "pepper-for-tests")
	if id, role, ok := st.Validate(secret); !ok || id != "ops" || role != "admin" {
		t.Fatalf("clé générée invalide: %q %q %v", id, role, ok)
	}

	listing, err := captureStdout(t, func() error { return run([]string{"-file", f, "-list"}) })
	if err != nil || !strings.Contains(listing, "ops\tadmin") || strings.Contains(listing, secret) {
		t.Fatalf("list: %q %v", listing, err)
	}

	newSecret, err := captureStdout(t, func() error { return run([]string{"-file", f, "-rotate", "ops", "-id", "ops-2"}) })
	if err != nil {
		t.Fatal(err)
	}
	st, _ = apikey.LoadFileStore(f, "pepper-for-tests")
	if _, _, ok := st.Validate(secret); ok {
		t.Fatal("ancienne clé valide après rotation")
	}
	if _, _, ok := st.Validate(newSecret); !ok {
		t.Fatal("nouvelle clé invalide")
	}

	if _, err := captureStdout(t, func() error { return run([]string{"-file", f, "-revoke", "ops-2"}) }); err != nil {
		t.Fatal(err)
	}
	st, _ = apikey.LoadFileStore(f, "pepper-for-tests")
	if _, _, ok := st.Validate(newSecret); ok {
		t.Fatal("clé révoquée valide")
	}
}

func TestKeygenRejectsBadInput(t *testing.T) {
	t.Setenv("AEGIS_API_KEY_PEPPER", "p")
	f := filepath.Join(t.TempDir(), "keys.yaml")
	if _, err := captureStdout(t, func() error { return run([]string{"-file", f, "-role", "root"}) }); err == nil {
		t.Fatal("rôle inconnu accepté")
	}
	t.Setenv("AEGIS_API_KEY_PEPPER", "")
	if _, err := captureStdout(t, func() error { return run([]string{"-file", f, "-role", "viewer"}) }); err == nil {
		t.Fatal("génération sans pepper acceptée")
	}
}
