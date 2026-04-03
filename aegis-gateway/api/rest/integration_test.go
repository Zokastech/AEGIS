// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/config"
	"github.com/zokastech/aegis/aegis-gateway/health"
)

func testEcho(t *testing.T) (*echo.Echo, *Services, config.Config) {
	t.Helper()
	pool := bridge.NewPool(2, 2*time.Second, func() (bridge.Engine, error) {
		return bridge.NewMockEngine(), nil
	})
	ld := config.NewLoader()
	if err := ld.MergeYAML([]byte("http_listen: :0\n")); err != nil {
		t.Fatal(err)
	}
	svc := &Services{
		Pool:          pool,
		Breaker:       bridge.NewDefaultBreaker(),
		EngineTimeout: 5 * time.Second,
		Loader:        ld,
	}
	svc.AnalyzeUC = NewAnalyzeUseCase(svc)
	cfg := config.Config{
		AdminAPIKeys: []string{"admin-secret"},
		APIKeyHeader: "X-API-Key",
	}
	e := echo.New()
	health.Attach(e, health.Opts{PingEngine: func(ctx context.Context) error {
		return pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
			_ = eng.Version()
			return nil
		})
	}})
	health.MarkStartupOK()
	AttachRoutes(e, svc, cfg)
	return e, svc, cfg
}

func TestPostAnalyze(t *testing.T) {
	e, _, _ := testEcho(t)
	b, _ := json.Marshal(AnalyzeRequest{Text: "contact a@b.co please"})
	req := httptest.NewRequest(http.MethodPost, "/v1/analyze", bytes.NewReader(b))
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("status %d body %s", rec.Code, rec.Body.String())
	}
	var out AnalyzeResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &out); err != nil {
		t.Fatal(err)
	}
	if !json.Valid(out.Result) {
		t.Fatalf("invalid json: %s", out.Result)
	}
}

func TestPostAnalyzeBatchPagination(t *testing.T) {
	e, _, _ := testEcho(t)
	body := AnalyzeBatchRequest{
		Texts:    []string{"a", "b", "c", "d"},
		Page:     2,
		PageSize: 2,
	}
	b, _ := json.Marshal(body)
	req := httptest.NewRequest(http.MethodPost, "/v1/analyze/batch", bytes.NewReader(b))
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("status %d %s", rec.Code, rec.Body.String())
	}
	var out AnalyzeBatchResponse
	_ = json.Unmarshal(rec.Body.Bytes(), &out)
	if out.Total != 4 || out.Page != 2 || len(out.Items) != 2 {
		t.Fatalf("unexpected %+v", out)
	}
}

func TestPostAnonymize(t *testing.T) {
	e, _, _ := testEcho(t)
	b, _ := json.Marshal(AnonymizeRequest{Text: "mail x@y.co"})
	req := httptest.NewRequest(http.MethodPost, "/v1/anonymize", bytes.NewReader(b))
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatal(rec.Code, rec.Body.String())
	}
}

func TestDeanonymizeAdmin501(t *testing.T) {
	e, _, _ := testEcho(t)
	b, _ := json.Marshal(DeanonymizeRequest{AnonymizedResultJSON: "{}"})
	req := httptest.NewRequest(http.MethodPost, "/v1/deanonymize", bytes.NewReader(b))
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	req.Header.Set("X-API-Key", "admin-secret")
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	if rec.Code != http.StatusNotImplemented {
		t.Fatalf("want 501 got %d %s", rec.Code, rec.Body.String())
	}
}

func TestDeanonymizeForbiddenWithoutKey(t *testing.T) {
	e, _, _ := testEcho(t)
	b, _ := json.Marshal(DeanonymizeRequest{AnonymizedResultJSON: "{}"})
	req := httptest.NewRequest(http.MethodPost, "/v1/deanonymize", bytes.NewReader(b))
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("want 403 got %d", rec.Code)
	}
}

func TestGetRecognizersEntitiesHealthMetricsOpenAPI(t *testing.T) {
	e, _, _ := testEcho(t)
	for _, p := range []string{
		"/health/live", "/health/ready", "/health/startup",
		"/v1/recognizers", "/v1/entities", "/v1/health", "/metrics", "/v1/openapi.yaml",
	} {
		req := httptest.NewRequest(http.MethodGet, p, nil)
		rec := httptest.NewRecorder()
		e.ServeHTTP(rec, req)
		if rec.Code != http.StatusOK {
			t.Fatalf("%s -> %d %s", p, rec.Code, rec.Body.String())
		}
	}
}

func TestFalsePositiveFeedback(t *testing.T) {
	e, _, _ := testEcho(t)
	req := httptest.NewRequest(http.MethodPost, "/v1/feedback/false-positive", nil)
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("want 204 got %d %s", rec.Code, rec.Body.String())
	}
}

func TestPutConfigAdmin(t *testing.T) {
	e, _, _ := testEcho(t)
	b, _ := json.Marshal(UpdateConfigRequest{YAML: "rate_limit_rps: 9\n"})
	req := httptest.NewRequest(http.MethodPut, "/v1/config", bytes.NewReader(b))
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	req.Header.Set("X-API-Key", "admin-secret")
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatal(rec.Code, rec.Body.String())
	}
}
