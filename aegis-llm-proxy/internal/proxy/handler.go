// AEGIS — zokastech.fr — Apache 2.0 / MIT

package proxy

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"
	"strings"
	"time"

	"github.com/zokastech/aegis/aegis-llm-proxy/internal/alert"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/analysis"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/config"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/deanon"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/engine"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/providers"
	"github.com/zokastech/aegis/aegis-llm-proxy/internal/stats"
)

// Handler reverse-proxy HTTP avec analyse AEGIS.
type Handler struct {
	Cfg      *config.Config
	Eng      *engine.Client
	Upstream *url.URL
	Stats    *stats.Registry
}

// New construit le handler (parse upstream_url).
func New(cfg *config.Config, eng *engine.Client, st *stats.Registry) (*Handler, error) {
	u, err := url.Parse(cfg.UpstreamURL)
	if err != nil {
		return nil, fmt.Errorf("upstream_url: %w", err)
	}
	if u.Scheme == "" || u.Host == "" {
		return nil, fmt.Errorf("upstream_url doit inclure scheme et host")
	}
	return &Handler{Cfg: cfg, Eng: eng, Upstream: u, Stats: st}, nil
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if h.Cfg.Dashboard.Enabled {
		p := strings.TrimSuffix(h.Cfg.Dashboard.Prefix, "/")
		if r.URL.Path == p+"/stats" && r.Method == http.MethodGet {
			h.Stats.WriteJSON(w)
			return
		}
		if r.URL.Path == p || r.URL.Path == p+"/" {
			if r.Method == http.MethodGet {
				h.serveDashboardIndex(w, p)
				return
			}
		}
	}
	if r.URL.Path == "/health" || r.URL.Path == "/ready" {
		w.Header().Set("Content-Type", "text/plain")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
		return
	}

	h.Stats.RequestsTotal.Add(1)
	body, err := io.ReadAll(r.Body)
	if err != nil {
		h.Stats.ErrorsTotal.Add(1)
		http.Error(w, "lecture corps", http.StatusBadRequest)
		return
	}
	_ = r.Body.Close()

	kind := providers.ClassifyPath(r.URL.Path)
	isStream := len(body) > 0 && providers.PeekStream(body)

	ctx, cancel := context.WithTimeout(r.Context(), time.Duration(h.Cfg.Engine.TimeoutSeconds)*time.Second)
	defer cancel()

	combined := collectForAnalysis(kind, body)
	var ar *analysis.Result
	if combined != "" && h.Cfg.Engine.Type == config.EngineHTTP {
		raw, aerr := h.Eng.Analyze(ctx, combined, analysis.AnalysisConfigJSON(h.Cfg))
		if aerr == nil {
			ar, _ = analysis.Parse(raw)
			if ar != nil && analysis.HasProtectedPII(h.Cfg, ar) {
				h.Stats.PIIDetected.Add(1)
			}
		}
	}

	if h.Cfg.Mode == config.ModeBlock && ar != nil && analysis.ShouldBlock(h.Cfg, ar) {
		h.Stats.BlockedTotal.Add(1)
		writeAegisBlock(w)
		return
	}

	if h.Cfg.Mode == config.ModeAlert && ar != nil && analysis.HasProtectedPII(h.Cfg, ar) {
		go alert.SendWebhook(context.Background(), h.Cfg.WebhookURL, alert.Payload{
			Path:    r.URL.Path,
			Summary: analysis.Summary(ar),
			Mode:    string(h.Cfg.Mode),
		})
	}

	if isStream {
		h.Stats.StreamPassthrough.Add(1)
		h.forwardStream(w, r, body)
		return
	}

	outBody := body
	var anon *deanon.AnonymizedPayload
	if h.Cfg.Mode == config.ModeAnonymize && kind != providers.KindUnknown && h.Cfg.Engine.Type == config.EngineHTTP {
		nb, pl, merr := h.mutatePrompt(ctx, kind, body)
		if merr != nil {
			h.Stats.ErrorsTotal.Add(1)
			http.Error(w, "anonymisation: "+merr.Error(), http.StatusBadGateway)
			return
		}
		outBody = nb
		anon = pl
		if anon != nil {
			h.Stats.AnonymizedTotal.Add(1)
		}
	}

	respBody, status, hdrs, ferr := h.doUpstream(ctx, r, outBody)
	if ferr != nil {
		h.Stats.ErrorsTotal.Add(1)
		http.Error(w, ferr.Error(), http.StatusBadGateway)
		return
	}
	defer respBody.Close()

	respBytes, err := io.ReadAll(respBody)
	if err != nil {
		h.Stats.ErrorsTotal.Add(1)
		http.Error(w, "lecture réponse amont", http.StatusBadGateway)
		return
	}

	ct := hdrs.Get("Content-Type")
	if anon != nil && strings.Contains(ct, "json") && status == http.StatusOK {
		respBytes = h.maybeRestoreResponse(kind, respBytes, anon)
	}

	for k, vv := range hdrs {
		if strings.EqualFold(k, "Content-Length") {
			continue
		}
		for _, v := range vv {
			w.Header().Add(k, v)
		}
	}
	w.Header().Set("Content-Length", fmt.Sprintf("%d", len(respBytes)))
	w.WriteHeader(status)
	_, _ = w.Write(respBytes)
}

func (h *Handler) serveDashboardIndex(w http.ResponseWriter, p string) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.WriteHeader(http.StatusOK)
	fmt.Fprintf(w, `<!DOCTYPE html><html><head><meta charset="utf-8"><title>AEGIS LLM Proxy</title></head><body>
<h1>AEGIS LLM Proxy</h1>
<p>zokastech.fr — stats pour le dashboard AEGIS principal.</p>
<p><a href="%s/stats">JSON stats</a></p>
</body></html>`, p)
}

func writeAegisBlock(w http.ResponseWriter) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusForbidden)
	_ = json.NewEncoder(w).Encode(map[string]interface{}{
		"error": map[string]string{
			"message": "Requête bloquée : données sensibles au-delà du seuil configuré.",
			"type":    "aegis_blocked",
			"code":    "pii_policy",
		},
	})
}

func collectForAnalysis(kind providers.Kind, body []byte) string {
	switch kind {
	case providers.KindOpenAIChat:
		return providers.CollectOpenAIChatText(body)
	case providers.KindOpenAICompletion:
		return providers.CollectOpenAICompletionText(body)
	case providers.KindAnthropicMessages:
		return providers.CollectAnthropicText(body)
	default:
		return ""
	}
}

func (h *Handler) mutatePrompt(ctx context.Context, kind providers.Kind, body []byte) ([]byte, *deanon.AnonymizedPayload, error) {
	var parts []*deanon.AnonymizedPayload
	mut := func(s string) (string, error) {
		if s == "" {
			return s, nil
		}
		raw, err := h.Eng.Anonymize(ctx, s, h.Cfg.AnonymizeConfigJSON)
		if err != nil {
			return "", err
		}
		pl, err := deanon.ParseAnonymized(raw)
		if err != nil {
			return "", err
		}
		parts = append(parts, pl)
		return pl.Text, nil
	}
	var out []byte
	var err error
	switch kind {
	case providers.KindOpenAIChat:
		out, err = providers.MutateOpenAIChat(body, mut)
	case providers.KindOpenAICompletion:
		out, err = providers.MutateOpenAICompletion(body, mut)
	case providers.KindAnthropicMessages:
		out, err = providers.MutateAnthropicMessages(body, mut)
	default:
		return body, nil, nil
	}
	if err != nil {
		return nil, nil, err
	}
	return out, deanon.Merge(parts), nil
}

func (h *Handler) maybeRestoreResponse(kind providers.Kind, body []byte, a *deanon.AnonymizedPayload) []byte {
	fn := func(s string) string { return deanon.Restore(s, a) }
	var err error
	switch kind {
	case providers.KindOpenAIChat:
		body, err = providers.RestoreOpenAIChatResponse(body, fn)
	case providers.KindOpenAICompletion:
		body, err = providers.RestoreOpenAICompletionResponse(body, fn)
	case providers.KindAnthropicMessages:
		body, err = providers.RestoreAnthropicResponse(body, fn)
	default:
		return body
	}
	if err != nil {
		return body
	}
	return body
}

func (h *Handler) doUpstream(ctx context.Context, r *http.Request, body []byte) (io.ReadCloser, int, http.Header, error) {
	target := h.Upstream.ResolveReference(&url.URL{
		Path:     r.URL.Path,
		RawQuery: r.URL.RawQuery,
	})
	req, err := http.NewRequestWithContext(ctx, r.Method, target.String(), bytes.NewReader(body))
	if err != nil {
		return nil, 0, nil, err
	}
	copyHeaders(req, r, body)
	if h.Cfg.InjectAPIKeyEnv != "" && req.Header.Get("Authorization") == "" {
		if k := os.Getenv(h.Cfg.InjectAPIKeyEnv); k != "" {
			req.Header.Set("Authorization", "Bearer "+k)
		}
	}
	cli := &http.Client{Timeout: time.Duration(h.Cfg.Engine.TimeoutSeconds) * time.Second}
	resp, err := cli.Do(req)
	if err != nil {
		return nil, 0, nil, err
	}
	return resp.Body, resp.StatusCode, resp.Header, nil
}

func copyHeaders(dst, src *http.Request, body []byte) {
	dst.Header = make(http.Header)
	for k, vv := range src.Header {
		if isHopHeader(k) {
			continue
		}
		dst.Header[k] = vv
	}
	dst.ContentLength = int64(len(body))
	dst.Header.Set("Content-Length", fmt.Sprintf("%d", len(body)))
	dst.Host = dst.URL.Host
}

func isHopHeader(k string) bool {
	switch strings.ToLower(k) {
	case "connection", "keep-alive", "proxy-authenticate", "proxy-authorization",
		"te", "trailers", "transfer-encoding", "upgrade":
		return true
	default:
		return false
	}
}

func (h *Handler) forwardStream(w http.ResponseWriter, r *http.Request, body []byte) {
	r.Body = io.NopCloser(bytes.NewReader(body))
	r.ContentLength = int64(len(body))
	r.GetBody = func() (io.ReadCloser, error) {
		return io.NopCloser(bytes.NewReader(body)), nil
	}

	rp := &httputil.ReverseProxy{
		Director: func(req *http.Request) {
			t := h.Upstream.ResolveReference(&url.URL{
				Path:     r.URL.Path,
				RawQuery: r.URL.RawQuery,
			})
			req.URL = t
			req.Host = t.Host
			if h.Cfg.InjectAPIKeyEnv != "" && req.Header.Get("Authorization") == "" {
				if k := os.Getenv(h.Cfg.InjectAPIKeyEnv); k != "" {
					req.Header.Set("Authorization", "Bearer "+k)
				}
			}
		},
		Transport:     http.DefaultTransport,
		FlushInterval: 100 * time.Millisecond,
	}
	rp.ServeHTTP(w, r)
}
