// AEGIS — zokastech.fr — Apache 2.0 / MIT

package engine

import (
	"bytes"
	"context"
	"crypto/tls"
	"crypto/x509"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"time"
)

// Client parle au REST gateway AEGIS (/v1/analyze, /v1/anonymize).
// maxEngineResponse bounds engine responses (defence against a misbehaving engine).
const maxEngineResponse = 64 << 20

type Client struct {
	BaseURL    string
	HTTPClient *http.Client
	// APIKey is sent as X-API-Key to the AEGIS gateway (empty = no header).
	APIKey string
}

// NewHTTPWithTLS builds a client trusting caFile (PEM) in addition to system roots.
func NewHTTPWithTLS(baseURL string, timeout time.Duration, caFile, apiKey string) (*Client, error) {
	c := NewHTTP(baseURL, timeout)
	c.APIKey = apiKey
	if caFile == "" {
		return c, nil
	}
	pem, err := os.ReadFile(caFile)
	if err != nil {
		return nil, fmt.Errorf("engine.ca_file: %w", err)
	}
	pool, err := x509.SystemCertPool()
	if err != nil || pool == nil {
		pool = x509.NewCertPool()
	}
	if !pool.AppendCertsFromPEM(pem) {
		return nil, fmt.Errorf("engine.ca_file: aucun certificat PEM valide")
	}
	tr := http.DefaultTransport.(*http.Transport).Clone()
	tr.TLSClientConfig = &tls.Config{RootCAs: pool, MinVersion: tls.VersionTLS12}
	c.HTTPClient.Transport = tr
	return c, nil
}

func (c *Client) setAuth(req *http.Request) {
	if c.APIKey != "" {
		req.Header.Set("X-API-Key", c.APIKey)
	}
}

// NewHTTP crée un client avec timeout.
func NewHTTP(baseURL string, timeout time.Duration) *Client {
	return &Client{
		BaseURL: strings.TrimSuffix(baseURL, "/"),
		HTTPClient: &http.Client{
			Timeout: timeout,
		},
	}
}

// Analyze retourne le JSON brut du champ result (AnalysisResult moteur).
func (c *Client) Analyze(ctx context.Context, text, analysisConfigJSON string) ([]byte, error) {
	body := map[string]string{
		"text": text,
	}
	if analysisConfigJSON != "" {
		body["analysis_config_json"] = analysisConfigJSON
	}
	raw, _ := json.Marshal(body)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.BaseURL+"/v1/analyze", bytes.NewReader(raw))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	c.setAuth(req)
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	b, err := io.ReadAll(io.LimitReader(resp.Body, maxEngineResponse))
	if err != nil {
		return nil, err
	}
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("analyze %s: %s", resp.Status, string(b))
	}
	var wrap struct {
		Result json.RawMessage `json:"result"`
	}
	if err := json.Unmarshal(b, &wrap); err != nil {
		return nil, err
	}
	if len(wrap.Result) == 0 || string(wrap.Result) == "null" {
		return nil, fmt.Errorf("analyze: résultat vide")
	}
	return wrap.Result, nil
}

// Anonymize retourne le JSON brut du champ result (objet anonymized + analysis).
func (c *Client) Anonymize(ctx context.Context, text, configJSON string) ([]byte, error) {
	body := map[string]string{
		"text":        text,
		"config_json": configJSON,
	}
	raw, _ := json.Marshal(body)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.BaseURL+"/v1/anonymize", bytes.NewReader(raw))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	c.setAuth(req)
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	b, err := io.ReadAll(io.LimitReader(resp.Body, maxEngineResponse))
	if err != nil {
		return nil, err
	}
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("anonymize %s: %s", resp.Status, string(b))
	}
	var wrap struct {
		Result json.RawMessage `json:"result"`
	}
	if err := json.Unmarshal(b, &wrap); err != nil {
		return nil, err
	}
	return wrap.Result, nil
}
