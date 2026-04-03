// AEGIS — zokastech.fr — Apache 2.0 / MIT

package auth

import (
	"crypto/tls"
	"crypto/x509"
	"errors"
	"net/http"
	"os"
)

// ClientCN returns the client TLS certificate Common Name (mTLS).
func ClientCN(r *http.Request) string {
	if r == nil || r.TLS == nil || len(r.TLS.PeerCertificates) == 0 {
		return ""
	}
	return r.TLS.PeerCertificates[0].Subject.CommonName
}

// BuildTLSClientAuth configures client-certificate requirements (production mTLS).
func BuildTLSClientAuth(caFile string, require bool) (*tls.Config, error) {
	if !require || caFile == "" {
		return nil, nil
	}
	pool := x509.NewCertPool()
	pemData, err := os.ReadFile(caFile)
	if err != nil {
		return nil, err
	}
	if !pool.AppendCertsFromPEM(pemData) {
		return nil, errors.New("mtls: CA client invalide ou vide")
	}
	return &tls.Config{
		ClientAuth: tls.RequireAndVerifyClientCert,
		ClientCAs:  pool,
		MinVersion: tls.VersionTLS12,
	}, nil
}
