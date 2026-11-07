// AEGIS — zokastech.fr — Apache 2.0 / MIT

package security

import (
	"crypto/x509"
	"os"
	"path/filepath"
	"testing"
)

func TestEnsureServerCertificateGeneratesOnceWithSafePerms(t *testing.T) {
	dir := filepath.Join(t.TempDir(), "tls")
	cf, kf := filepath.Join(dir, "server.crt"), filepath.Join(dir, "server.key")
	c1, err := EnsureServerCertificate(cf, kf, dir, true, []string{"aegis-gateway", "10.0.0.1"})
	if err != nil {
		t.Fatal(err)
	}
	st, _ := os.Stat(kf)
	if st.Mode().Perm() != 0o600 {
		t.Fatalf("clé privée en %o", st.Mode().Perm())
	}
	leaf, err := x509.ParseCertificate(c1.Certificate[0])
	if err != nil {
		t.Fatal(err)
	}
	if err := leaf.VerifyHostname("aegis-gateway"); err != nil {
		t.Fatalf("SAN DNS manquant: %v", err)
	}
	if err := leaf.VerifyHostname("10.0.0.1"); err != nil {
		t.Fatalf("SAN IP manquant: %v", err)
	}
	c2, err := EnsureServerCertificate(cf, kf, dir, true, nil)
	if err != nil {
		t.Fatal(err)
	}
	if string(c1.Certificate[0]) != string(c2.Certificate[0]) {
		t.Fatal("certificat régénéré alors qu’il existe")
	}
}

func TestEnsureServerCertificateNoAutogen(t *testing.T) {
	dir := t.TempDir()
	if _, err := EnsureServerCertificate(filepath.Join(dir, "a"), filepath.Join(dir, "b"), dir, false, nil); err == nil {
		t.Fatal("absence de cert acceptée sans auto-génération")
	}
}
