// AEGIS — zokastech.fr — Apache 2.0 / MIT

package auth

import (
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"net/http"
	"os"
	"path/filepath"
	"testing"
)

func TestClientCNRequiresVerifiedChain(t *testing.T) {
	forged := &x509.Certificate{Subject: pkix.Name{CommonName: "admin-service"}}
	// Unverified peer certificate (attacker-controlled when client auth is not verified).
	r := &http.Request{TLS: &tls.ConnectionState{PeerCertificates: []*x509.Certificate{forged}}}
	if cn := ClientCN(r); cn != "" {
		t.Fatalf("CN non vérifié accepté: %q", cn)
	}
	r.TLS.VerifiedChains = [][]*x509.Certificate{{forged}}
	if cn := ClientCN(r); cn != "admin-service" {
		t.Fatalf("CN vérifié ignoré: %q", cn)
	}
	if ClientCN(nil) != "" || ClientCN(&http.Request{}) != "" {
		t.Fatal("nil")
	}
}

func TestBuildTLSClientAuth(t *testing.T) {
	if c, err := BuildTLSClientAuth("", false); c != nil || err != nil {
		t.Fatal("mTLS non requis doit renvoyer nil,nil")
	}
	if _, err := BuildTLSClientAuth("", true); err == nil {
		t.Fatal("mTLS requis sans CA doit échouer (fail closed)")
	}
	bad := filepath.Join(t.TempDir(), "ca.pem")
	_ = os.WriteFile(bad, []byte("not a pem"), 0o600)
	if _, err := BuildTLSClientAuth(bad, true); err == nil {
		t.Fatal("CA invalide acceptée")
	}
	if _, err := BuildTLSClientAuth(filepath.Join(t.TempDir(), "absent.pem"), true); err == nil {
		t.Fatal("CA absente acceptée")
	}
}
