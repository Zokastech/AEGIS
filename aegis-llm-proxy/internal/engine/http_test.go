// AEGIS — zokastech.fr — Apache 2.0 / MIT

package engine

import (
	"context"
	"encoding/pem"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestAnalyzeAndAnonymize(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.Path {
		case "/v1/analyze":
			_, _ = w.Write([]byte(`{"result":{"entities":[]}}`))
		case "/v1/anonymize":
			_, _ = w.Write([]byte(`{"result":{"text":"x"}}`))
		default:
			w.WriteHeader(http.StatusInternalServerError)
		}
	}))
	defer srv.Close()
	c := NewHTTP(srv.URL+"/", 2*time.Second)
	if b, err := c.Analyze(context.Background(), "t", `{"score_threshold":0.5}`); err != nil || string(b) != `{"entities":[]}` {
		t.Fatalf("analyze: %s %v", b, err)
	}
	if b, err := c.Anonymize(context.Background(), "t", "{}"); err != nil || string(b) != `{"text":"x"}` {
		t.Fatalf("anonymize: %s %v", b, err)
	}
}

func TestErrorsPropagated(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/v1/analyze" {
			_, _ = w.Write([]byte(`{"result":null}`))
			return
		}
		w.WriteHeader(http.StatusBadGateway)
	}))
	defer srv.Close()
	c := NewHTTP(srv.URL, 2*time.Second)
	if _, err := c.Analyze(context.Background(), "t", ""); err == nil {
		t.Fatal("résultat vide accepté")
	}
	if _, err := c.Anonymize(context.Background(), "t", ""); err == nil {
		t.Fatal("HTTP 502 accepté")
	}
	if _, err := NewHTTP("http://127.0.0.1:1", time.Second).Analyze(context.Background(), "t", ""); err == nil {
		t.Fatal("moteur injoignable accepté")
	}
}

func TestAPIKeyHeaderSent(t *testing.T) {
	var got string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		got = r.Header.Get("X-API-Key")
		_, _ = w.Write([]byte(`{"result":{"entities":[]}}`))
	}))
	defer srv.Close()
	c, err := NewHTTPWithTLS(srv.URL, time.Second, "", "k-123")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := c.Analyze(context.Background(), "t", ""); err != nil {
		t.Fatal(err)
	}
	if got != "k-123" {
		t.Fatalf("X-API-Key = %q", got)
	}
	if _, err := NewHTTPWithTLS(srv.URL, time.Second, "/nonexistent.pem", ""); err == nil {
		t.Fatal("ca_file absent accepté")
	}
}

func TestPrivateCATrusted(t *testing.T) {
	srv := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"result":{"entities":[]}}`))
	}))
	defer srv.Close()
	// Without the CA: TLS verification must fail (never skipped).
	if _, err := NewHTTP(srv.URL, time.Second).Analyze(context.Background(), "t", ""); err == nil {
		t.Fatal("certificat non fiable accepté")
	}
	pemPath := filepath.Join(t.TempDir(), "ca.pem")
	certPEM := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: srv.Certificate().Raw})
	if err := os.WriteFile(pemPath, certPEM, 0o600); err != nil {
		t.Fatal(err)
	}
	c, err := NewHTTPWithTLS(srv.URL, time.Second, pemPath, "")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := c.Analyze(context.Background(), "t", ""); err != nil {
		t.Fatalf("CA privée non prise en compte: %v", err)
	}
}
