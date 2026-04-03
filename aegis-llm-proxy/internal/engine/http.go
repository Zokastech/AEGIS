// AEGIS — zokastech.fr — Apache 2.0 / MIT

package engine

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

// Client parle au REST gateway AEGIS (/v1/analyze, /v1/anonymize).
type Client struct {
	BaseURL    string
	HTTPClient *http.Client
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
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	b, err := io.ReadAll(resp.Body)
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
	if len(wrap.Result) == 0 {
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
	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	b, err := io.ReadAll(resp.Body)
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
