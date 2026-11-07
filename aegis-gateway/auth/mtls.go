// AEGIS — zokastech.fr — Apache 2.0 / MIT

package auth

import (
	"crypto/tls"
	"crypto/x509"
	"errors"
	"net/http"
	"os"
)

// ClientCN returns the Common Name of a *verified* client TLS certificate (mTLS).
//
// Only certificates that chained to the configured client CA (tls.ConnectionState.VerifiedChains)
// are trusted. PeerCertificates alone is attacker-controlled when the listener does not verify
// client certificates (e.g. RequestClientCert), so it must never be used for authentication.
func ClientCN(r *http.Request) string {
	if r == nil {
		return ""
	}
	return VerifiedClientCN(r.TLS)
}

// VerifiedClientCN extracts the leaf CN from the first verified chain, or "".
func VerifiedClientCN(cs *tls.ConnectionState) string {
	if cs == nil || len(cs.VerifiedChains) == 0 || len(cs.VerifiedChains[0]) == 0 {
		return ""
	}
	return cs.VerifiedChains[0][0].Subject.CommonName
}

// BuildTLSClientAuth returns a TLS config fragment that requires and verifies client certificates.
// Returns (nil, nil) when mTLS is not required.
func BuildTLSClientAuth(caFile string, require bool) (*tls.Config, error) {
	if !require {
		return nil, nil
	}
	if caFile == "" {
		return nil, errors.New("mtls: tls.client_ca_file requis quand mtls.require=true")
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
