// AEGIS — zokastech.fr — Apache 2.0 / MIT

package alert

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
)

func TestSendWebhookPayloadAndNoRedirect(t *testing.T) {
	var redirected atomic.Bool
	target := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { redirected.Store(true) }))
	defer target.Close()
	var got Payload
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewDecoder(r.Body).Decode(&got)
		http.Redirect(w, r, target.URL, http.StatusTemporaryRedirect)
	}))
	defer srv.Close()

	SendWebhook(context.Background(), srv.URL, Payload{Path: "/v1/chat/completions", Summary: "1 entité(s)"})
	if got.Event != "aegis.llm.pii_detected" || got.Timestamp == "" || got.Path == "" {
		t.Fatalf("payload: %+v", got)
	}
	if redirected.Load() {
		t.Fatal("redirection suivie (risque SSRF)")
	}
	SendWebhook(context.Background(), "", Payload{}) // no-op
}
