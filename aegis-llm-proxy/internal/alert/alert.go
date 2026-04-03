// AEGIS — zokastech.fr — Apache 2.0 / MIT

package alert

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"time"
)

var httpClient = &http.Client{Timeout: 8 * time.Second}

// Payload envoyé au webhook (mode alert).
type Payload struct {
	Event     string `json:"event"`
	Path      string `json:"path"`
	Summary   string `json:"summary"`
	Mode      string `json:"mode"`
	Timestamp string `json:"timestamp"`
}

// SendWebhook POST JSON sans bloquer le chemin critique (erreurs ignorées côté appelant).
func SendWebhook(ctx context.Context, url string, p Payload) {
	if url == "" {
		return
	}
	p.Timestamp = time.Now().UTC().Format(time.RFC3339)
	if p.Event == "" {
		p.Event = "aegis.llm.pii_detected"
	}
	raw, _ := json.Marshal(p)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(raw))
	if err != nil {
		return
	}
	req.Header.Set("Content-Type", "application/json")
	_, _ = httpClient.Do(req)
}
