// AEGIS — zokastech.fr — Apache 2.0 / MIT

package proxy

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/zokastech/aegis/aegis-llm-proxy/internal/config"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/engine"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/stats"
)

const email = "jean.dupont@example.fr"

// fakeEngine mimics the AEGIS gateway /v1/analyze and /v1/anonymize endpoints:
// any occurrence of `email` is reported / replaced by [EMAIL_1].
func fakeEngine(t *testing.T) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var in struct {
			Text string `json:"text"`
		}
		_ = json.NewDecoder(r.Body).Decode(&in)
		w.Header().Set("Content-Type", "application/json")
		switch r.URL.Path {
		case "/v1/analyze":
			ents := []map[string]interface{}{}
			if strings.Contains(in.Text, email) {
				ents = append(ents, map[string]interface{}{"entity_type": "EMAIL", "score": 0.99, "text": email})
			}
			_ = json.NewEncoder(w).Encode(map[string]interface{}{"result": map[string]interface{}{"entities": ents}})
		case "/v1/anonymize":
			out := strings.ReplaceAll(in.Text, email, "[EMAIL_1]")
			tr := []map[string]interface{}{}
			if out != in.Text {
				tr = append(tr, map[string]interface{}{"replacement": "[EMAIL_1]", "original_text": email, "entity_type": "EMAIL"})
			}
			_ = json.NewEncoder(w).Encode(map[string]interface{}{"result": map[string]interface{}{
				"anonymized": map[string]interface{}{"text": out, "transformations": tr},
			}})
		default:
			http.NotFound(w, r)
		}
	}))
}

type upstreamRecorder struct {
	mu     sync.Mutex
	bodies []string
	srv    *httptest.Server
}

func (u *upstreamRecorder) last() string {
	u.mu.Lock()
	defer u.mu.Unlock()
	if len(u.bodies) == 0 {
		return ""
	}
	return u.bodies[len(u.bodies)-1]
}

// fakeUpstream echoes the received prompt in an OpenAI chat response (or an SSE stream).
func fakeUpstream(t *testing.T) *upstreamRecorder {
	t.Helper()
	rec := &upstreamRecorder{}
	rec.srv = httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		b, _ := io.ReadAll(r.Body)
		rec.mu.Lock()
		rec.bodies = append(rec.bodies, string(b))
		rec.mu.Unlock()
		var req struct {
			Stream   bool `json:"stream"`
			Messages []struct {
				Content string `json:"content"`
			} `json:"messages"`
		}
		_ = json.Unmarshal(b, &req)
		content := ""
		if len(req.Messages) > 0 {
			content = req.Messages[0].Content
		}
		if req.Stream {
			w.Header().Set("Content-Type", "text/event-stream")
			_, _ = io.WriteString(w, "data: "+content+"\n\n")
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]interface{}{
			"choices": []interface{}{map[string]interface{}{"message": map[string]interface{}{"role": "assistant", "content": "reçu: " + content}}},
		})
	}))
	return rec
}

func newHandler(t *testing.T, mode config.Mode, engineURL, upstreamURL string, mut func(*config.Config)) *Handler {
	t.Helper()
	cfg := &config.Config{
		UpstreamURL: upstreamURL,
		Mode:        mode,
		Engine:      config.EngineBlock{Type: config.EngineHTTP, BaseURL: engineURL, TimeoutSeconds: 5},
		Dashboard:   config.DashboardConfig{Enabled: true, Prefix: "/internal/llm-proxy"},
	}
	if mut != nil {
		mut(cfg)
	}
	h, err := New(cfg, engine.NewHTTP(engineURL, 5*time.Second), &stats.Registry{})
	if err != nil {
		t.Fatal(err)
	}
	return h
}

func chatBody(stream bool) []byte {
	b, _ := json.Marshal(map[string]interface{}{
		"model":    "gpt-test",
		"stream":   stream,
		"messages": []map[string]string{{"role": "user", "content": "Écris à " + email}},
	})
	return b
}

func do(h http.Handler, method, path string, body []byte) *httptest.ResponseRecorder {
	req := httptest.NewRequest(method, path, bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

func TestAnonymizeModeStripsPIIAndRestoresResponse(t *testing.T) {
	eng := fakeEngine(t)
	defer eng.Close()
	up := fakeUpstream(t)
	defer up.srv.Close()
	h := newHandler(t, config.ModeAnonymize, eng.URL, up.srv.URL, nil)

	rec := do(h, http.MethodPost, "/v1/chat/completions", chatBody(false))
	if rec.Code != http.StatusOK {
		t.Fatalf("status %d: %s", rec.Code, rec.Body.String())
	}
	if strings.Contains(up.last(), email) {
		t.Fatalf("PII envoyée à l’amont: %s", up.last())
	}
	if !strings.Contains(up.last(), "[EMAIL_1]") {
		t.Fatalf("placeholder absent côté amont: %s", up.last())
	}
	if !strings.Contains(rec.Body.String(), email) {
		t.Fatalf("réponse non restaurée: %s", rec.Body.String())
	}
}

// Regression: streaming requests used to bypass anonymization entirely.
func TestAnonymizeModeStreamDoesNotLeakPII(t *testing.T) {
	eng := fakeEngine(t)
	defer eng.Close()
	up := fakeUpstream(t)
	defer up.srv.Close()
	h := newHandler(t, config.ModeAnonymize, eng.URL, up.srv.URL, nil)

	rec := do(h, http.MethodPost, "/v1/chat/completions", chatBody(true))
	if rec.Code != http.StatusOK {
		t.Fatalf("status %d: %s", rec.Code, rec.Body.String())
	}
	if strings.Contains(up.last(), email) {
		t.Fatalf("flux: PII envoyée à l’amont: %s", up.last())
	}
}

func TestBlockModeBlocksPII(t *testing.T) {
	eng := fakeEngine(t)
	defer eng.Close()
	up := fakeUpstream(t)
	defer up.srv.Close()
	h := newHandler(t, config.ModeBlock, eng.URL, up.srv.URL, nil)

	rec := do(h, http.MethodPost, "/v1/chat/completions", chatBody(false))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("want 403 got %d", rec.Code)
	}
	if up.last() != "" {
		t.Fatal("la requête bloquée a atteint l’amont")
	}
	if !strings.Contains(rec.Body.String(), "aegis_blocked") {
		t.Fatalf("corps inattendu: %s", rec.Body.String())
	}
}

func TestBlockModeFailsClosedWhenEngineDown(t *testing.T) {
	up := fakeUpstream(t)
	defer up.srv.Close()
	h := newHandler(t, config.ModeBlock, "http://127.0.0.1:1", up.srv.URL, nil)

	rec := do(h, http.MethodPost, "/v1/chat/completions", chatBody(false))
	if rec.Code != http.StatusServiceUnavailable {
		t.Fatalf("want 503 got %d", rec.Code)
	}
	if up.last() != "" {
		t.Fatal("fail-open: requête transmise sans analyse")
	}
}

func TestBlockModeFailOpenIsOptIn(t *testing.T) {
	up := fakeUpstream(t)
	defer up.srv.Close()
	h := newHandler(t, config.ModeBlock, "http://127.0.0.1:1", up.srv.URL, func(c *config.Config) { c.FailOpen = true })

	rec := do(h, http.MethodPost, "/v1/chat/completions", chatBody(false))
	if rec.Code != http.StatusOK {
		t.Fatalf("want 200 got %d", rec.Code)
	}
}

func TestBodyTooLarge(t *testing.T) {
	eng := fakeEngine(t)
	defer eng.Close()
	up := fakeUpstream(t)
	defer up.srv.Close()
	h := newHandler(t, config.ModeTransparent, eng.URL, up.srv.URL, func(c *config.Config) { c.MaxBodyBytes = 64 })

	rec := do(h, http.MethodPost, "/v1/chat/completions", bytes.Repeat([]byte("a"), 1024))
	if rec.Code != http.StatusRequestEntityTooLarge {
		t.Fatalf("want 413 got %d", rec.Code)
	}
}

// Regression: unknown paths caused infinite recursion in ClassifyPath (fatal stack overflow).
func TestUnknownPathIsForwardedWithoutCrash(t *testing.T) {
	eng := fakeEngine(t)
	defer eng.Close()
	up := fakeUpstream(t)
	defer up.srv.Close()
	h := newHandler(t, config.ModeAnonymize, eng.URL, up.srv.URL, nil)

	rec := do(h, http.MethodPost, "/v1/embeddings?x=1", []byte(`{"input":"hello"}`))
	if rec.Code != http.StatusOK {
		t.Fatalf("want 200 got %d", rec.Code)
	}
}

func TestUpstreamErrorDoesNotLeakInternals(t *testing.T) {
	eng := fakeEngine(t)
	defer eng.Close()
	h := newHandler(t, config.ModeTransparent, eng.URL, "http://127.0.0.1:1", nil)

	rec := do(h, http.MethodPost, "/v1/chat/completions", chatBody(false))
	if rec.Code != http.StatusBadGateway {
		t.Fatalf("want 502 got %d", rec.Code)
	}
	if strings.Contains(rec.Body.String(), "127.0.0.1") {
		t.Fatalf("détails internes exposés: %s", rec.Body.String())
	}
}

func TestHealthAndDashboard(t *testing.T) {
	h := newHandler(t, config.ModeTransparent, "http://127.0.0.1:1", "http://127.0.0.1:1", func(c *config.Config) {
		c.Dashboard.Prefix = `/x"><script>alert(1)</script>`
	})
	if rec := do(h, http.MethodGet, "/health", nil); rec.Code != http.StatusOK || rec.Body.String() != "ok" {
		t.Fatalf("health: %d %q", rec.Code, rec.Body.String())
	}
	rec := do(h, http.MethodGet, `/x"><script>alert(1)</script>`, nil)
	if strings.Contains(rec.Body.String(), "<script>") {
		t.Fatalf("XSS dans la page dashboard: %s", rec.Body.String())
	}
	if rec.Header().Get("Content-Security-Policy") == "" {
		t.Fatal("CSP absente")
	}
}

func TestStatsEndpoint(t *testing.T) {
	eng := fakeEngine(t)
	defer eng.Close()
	up := fakeUpstream(t)
	defer up.srv.Close()
	h := newHandler(t, config.ModeBlock, eng.URL, up.srv.URL, nil)
	_ = do(h, http.MethodPost, "/v1/chat/completions", chatBody(false))

	rec := do(h, http.MethodGet, "/internal/llm-proxy/stats", nil)
	var m map[string]uint64
	if err := json.Unmarshal(rec.Body.Bytes(), &m); err != nil {
		t.Fatalf("stats JSON: %v (%s)", err, rec.Body.String())
	}
	if len(m) == 0 {
		t.Fatalf("stats vides: %s", rec.Body.String())
	}
}
