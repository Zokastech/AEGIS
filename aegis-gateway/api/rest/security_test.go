// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/zokastech/aegis/aegis-gateway/audit"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/config"
)

type securedEnv struct {
	e       *echo.Echo
	gw      *GatewaySecurity
	secrets map[string]string // key id -> plaintext secret
}

// newSecuredEnv builds the production route stack (auth + RBAC + audit) with real API keys.
func newSecuredEnv(t *testing.T, mutate func(*config.SecurityConfig)) *securedEnv {
	t.Helper()
	dir := t.TempDir()
	sc := config.DefaultSecurity()
	sc.APIKeys.FilePath = filepath.Join(dir, "api_keys.yaml")
	sc.APIKeys.Pepper = "test-pepper-0123456789"
	sc.RBAC.FilePath = ""
	sc.Audit.FilePath = filepath.Join(dir, "audit.jsonl")
	sc.Audit.ArchiveDir = filepath.Join(dir, "archive")
	if mutate != nil {
		mutate(&sc)
	}
	cfg := config.Config{APIKeyHeader: "X-API-Key", Security: sc}
	gw, err := BootstrapGatewaySecurity(cfg)
	if err != nil {
		t.Fatal(err)
	}
	secrets := map[string]string{}
	for id, role := range map[string]string{
		"admin-1": "admin", "admin-2": "admin", "viewer-1": "viewer", "operator-1": "operator", "auditor-1": "auditor",
	} {
		s := "secret-" + id + "-xxxxxxxxxxxxxxxx"
		if err := gw.APIKeys.Generate(id, role, s); err != nil {
			t.Fatal(err)
		}
		secrets[id] = s
	}
	pool := bridge.NewPool(2, 2*time.Second, func() (bridge.Engine, error) { return bridge.NewMockEngine(), nil })
	ld := config.NewLoader()
	svc := &Services{Pool: pool, Breaker: bridge.NewDefaultBreaker(), EngineTimeout: 5 * time.Second, Loader: ld, Gateway: gw}
	svc.AnalyzeUC = NewAnalyzeUseCase(svc)
	e := echo.New()
	AttachRoutes(e, svc, cfg)
	return &securedEnv{e: e, gw: gw, secrets: secrets}
}

func (s *securedEnv) req(method, path, body string, hdr map[string]string) *httptest.ResponseRecorder {
	r := httptest.NewRequest(method, path, strings.NewReader(body))
	r.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	for k, v := range hdr {
		r.Header.Set(k, v)
	}
	rec := httptest.NewRecorder()
	s.e.ServeHTTP(rec, r)
	return rec
}

func analyzeBody() string {
	b, _ := json.Marshal(AnalyzeRequest{Text: "mail a@b.co"})
	return string(b)
}

func TestSecured_Unauthenticated401(t *testing.T) {
	s := newSecuredEnv(t, nil)
	if rec := s.req(http.MethodPost, "/v1/analyze", analyzeBody(), nil); rec.Code != http.StatusUnauthorized {
		t.Fatalf("want 401 got %d", rec.Code)
	}
	if rec := s.req(http.MethodPost, "/v1/analyze", analyzeBody(), map[string]string{"X-API-Key": "nope"}); rec.Code != http.StatusUnauthorized {
		t.Fatalf("clé invalide: want 401 got %d", rec.Code)
	}
}

func TestSecured_PublicProbes(t *testing.T) {
	s := newSecuredEnv(t, nil)
	if rec := s.req(http.MethodGet, "/livez", "", nil); rec.Code != http.StatusNoContent {
		t.Fatalf("livez: %d", rec.Code)
	}
}

func TestSecured_RBACMatrix(t *testing.T) {
	s := newSecuredEnv(t, nil)
	cases := []struct {
		key    string
		method string
		path   string
		body   string
		want   int
	}{
		{"viewer-1", http.MethodPost, "/v1/analyze", analyzeBody(), http.StatusOK},
		{"viewer-1", http.MethodPost, "/v1/anonymize", `{"text":"a@b.co"}`, http.StatusForbidden},
		{"operator-1", http.MethodPost, "/v1/anonymize", `{"text":"a@b.co"}`, http.StatusOK},
		{"operator-1", http.MethodPut, "/v1/config", `{"yaml":"pool_size: 3\n"}`, http.StatusForbidden},
		{"admin-1", http.MethodPut, "/v1/config", `{"yaml":"pool_size: 3\n"}`, http.StatusOK},
		{"auditor-1", http.MethodPost, "/v1/analyze", analyzeBody(), http.StatusForbidden},
		{"viewer-1", http.MethodGet, "/v1/audit/export", "", http.StatusForbidden},
		{"viewer-1", http.MethodGet, "/metrics", "", http.StatusForbidden},
		{"auditor-1", http.MethodGet, "/metrics", "", http.StatusOK},
	}
	for _, c := range cases {
		rec := s.req(c.method, c.path, c.body, map[string]string{"X-API-Key": s.secrets[c.key]})
		if rec.Code != c.want {
			t.Errorf("%s %s %s: want %d got %d (%s)", c.key, c.method, c.path, c.want, rec.Code, rec.Body.String())
		}
	}
}

// Runtime config must not be able to touch security (e.g. disable auth).
func TestSecured_PutConfigRejectsSecurityKeys(t *testing.T) {
	s := newSecuredEnv(t, nil)
	for _, y := range []string{
		"security:\n  development:\n    disable_auth: true\n",
		"admin_api_keys: [\"x\"]\n",
		"grpc_listen: \":1\"\n",
	} {
		b, _ := json.Marshal(UpdateConfigRequest{YAML: y})
		rec := s.req(http.MethodPut, "/v1/config", string(b), map[string]string{"X-API-Key": s.secrets["admin-1"]})
		if rec.Code != http.StatusBadRequest {
			t.Errorf("%q: want 400 got %d", y, rec.Code)
		}
	}
}

func TestSecured_RevokedKeyRejected(t *testing.T) {
	s := newSecuredEnv(t, nil)
	if err := s.gw.APIKeys.Revoke("viewer-1"); err != nil {
		t.Fatal(err)
	}
	rec := s.req(http.MethodPost, "/v1/analyze", analyzeBody(), map[string]string{"X-API-Key": s.secrets["viewer-1"]})
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("clé révoquée encore acceptée: %d", rec.Code)
	}
}

func dual(s *securedEnv, a, b string) map[string]string {
	return map[string]string{"X-Aegis-Approval-1": s.secrets[a], "X-Aegis-Approval-2": s.secrets[b]}
}

func TestSecured_DeanonymizeDualApproval(t *testing.T) {
	s := newSecuredEnv(t, nil)
	body := `{"anonymized_result_json":"{}"}`
	if rec := s.req(http.MethodPost, "/v1/deanonymize", body, nil); rec.Code != http.StatusForbidden {
		t.Fatalf("sans approbation: %d", rec.Code)
	}
	if rec := s.req(http.MethodPost, "/v1/deanonymize", body, dual(s, "admin-1", "admin-1")); rec.Code != http.StatusForbidden {
		t.Fatalf("même clé deux fois: %d", rec.Code)
	}
	if rec := s.req(http.MethodPost, "/v1/deanonymize", body, dual(s, "admin-1", "viewer-1")); rec.Code != http.StatusForbidden {
		t.Fatalf("non-admin: %d", rec.Code)
	}
	// Single admin key in X-API-Key is NOT enough.
	if rec := s.req(http.MethodPost, "/v1/deanonymize", body, map[string]string{"X-API-Key": s.secrets["admin-1"]}); rec.Code != http.StatusForbidden {
		t.Fatalf("clé unique: %d", rec.Code)
	}
	rec := s.req(http.MethodPost, "/v1/deanonymize", body, dual(s, "admin-1", "admin-2"))
	if rec.Code == http.StatusForbidden || rec.Code == http.StatusUnauthorized {
		t.Fatalf("double approbation valide refusée: %d %s", rec.Code, rec.Body.String())
	}
}

func TestSecured_DeanonymizeRateLimited(t *testing.T) {
	s := newSecuredEnv(t, func(sc *config.SecurityConfig) { sc.Deanon.MaxPerHour = 2 })
	body := `{"anonymized_result_json":"{}"}`
	var last int
	for i := 0; i < 3; i++ {
		last = s.req(http.MethodPost, "/v1/deanonymize", body, dual(s, "admin-1", "admin-2")).Code
	}
	if last != http.StatusTooManyRequests {
		t.Fatalf("3e appel: want 429 got %d", last)
	}
}

func TestSecured_AuditLogWritten(t *testing.T) {
	s := newSecuredEnv(t, nil)
	_ = s.req(http.MethodPost, "/v1/analyze", analyzeBody(), map[string]string{"X-API-Key": s.secrets["viewer-1"]})
	lines, err := auditLines(s.gw.Config.Audit.FilePath)
	if err != nil || len(lines) == 0 {
		t.Fatalf("audit vide: %v", err)
	}
	if strings.Contains(strings.Join(lines, "\n"), s.secrets["viewer-1"]) {
		t.Fatal("secret de clé API présent dans le journal d’audit")
	}
	if !strings.Contains(lines[len(lines)-1], "api_key:viewer-1") {
		t.Fatalf("acteur absent: %s", lines[len(lines)-1])
	}
}

func TestLegacyRoleHeaderRequiresExplicitOptIn(t *testing.T) {
	e := echo.New()
	h := RequireAdmin(nil, "X-API-Key")(func(c echo.Context) error { return c.NoContent(http.StatusOK) })
	e.POST("/x", h)
	call := func() int {
		r := httptest.NewRequest(http.MethodPost, "/x", nil)
		r.Header.Set("X-Aegis-Role", "admin")
		rec := httptest.NewRecorder()
		e.ServeHTTP(rec, r)
		return rec.Code
	}
	if code := call(); code != http.StatusForbidden {
		t.Fatalf("X-Aegis-Role honoré sans opt-in: %d", code)
	}
	t.Setenv("AEGIS_DEV_ALLOW_ROLE_HEADER", "1")
	if code := call(); code != http.StatusOK {
		t.Fatalf("opt-in dev: %d", code)
	}
}

func TestAdminKeyMatch(t *testing.T) {
	keys := []string{"alpha", " beta ", ""}
	if !adminKeyMatch(keys, "alpha") || !adminKeyMatch(keys, "beta") {
		t.Fatal("clé valide refusée")
	}
	if adminKeyMatch(keys, "") || adminKeyMatch(keys, "alph") || adminKeyMatch(nil, "alpha") {
		t.Fatal("clé invalide acceptée")
	}
}

func TestFilterAnalysisForPrincipalNil(t *testing.T) {
	raw := []byte(`{"entities":[]}`)
	if !bytes.Equal(FilterAnalysisForPrincipal(raw, nil, nil), raw) {
		t.Fatal("nil services doit renvoyer l’entrée")
	}
}

func auditLines(path string) ([]string, error) {
	return audit.ScanLinesForward(path, 100)
}
