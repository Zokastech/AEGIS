// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/sony/gobreaker"
	"github.com/zokastech/aegis/aegis-gateway/audit"
	"github.com/zokastech/aegis/aegis-gateway/auth"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	gwconfig "github.com/zokastech/aegis/aegis-gateway/config"
	"github.com/zokastech/aegis/aegis-gateway/internal/app"
	"github.com/zokastech/aegis/aegis-gateway/internal/domain"
	"github.com/zokastech/aegis/aegis-gateway/metrics"
	"github.com/zokastech/aegis/aegis-gateway/policy"
)

// Services holds HTTP-layer dependencies.
type Services struct {
	Pool          *bridge.Pool
	Breaker       *gobreaker.CircuitBreaker
	EngineTimeout time.Duration
	Loader        *gwconfig.Loader
	Gateway       *GatewaySecurity // nil → routes legacy (tests / compat)
	Policy        *policy.PolicyEngine
	// AnalyzeUC is the “analyze a text” use case (application layer).
	AnalyzeUC *app.Analyze
}

// Analyze POST /v1/analyze
// @Summary Analyze text
// @Description Returns detected PII entities (AEGIS engine JSON).
// @Tags analyze
// @Accept json
// @Produce json
// @Param request body AnalyzeRequest true "Text plus optional analysis config (JSON string)"
// @Success 200 {object} AnalyzeResponse
// @Failure 400 {object} ErrorBody
// @Failure 500 {object} ErrorBody
// @Router /v1/analyze [post]
func Analyze(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		started := time.Now()
		st := http.StatusOK
		analysisJSON := ""
		var metricsRaw []byte
		defer func() {
			metrics.ObserveAnalyze("/v1/analyze", http.MethodPost, st, analysisJSON, started, metricsRaw)
		}()

		var req AnalyzeRequest
		if err := c.Bind(&req); err != nil {
			st = http.StatusBadRequest
			return errJSON(c, http.StatusBadRequest, "INVALID_JSON", err.Error())
		}
		analysisJSON = req.AnalysisJSON
		if s.AnalyzeUC == nil {
			st = http.StatusInternalServerError
			return errJSON(c, http.StatusInternalServerError, "CONFIG", "AnalyzeUC non configuré")
		}
		res, err := s.AnalyzeUC.Execute(c.Request().Context(), app.AnalyzeCommand{
			Text:         req.Text,
			AnalysisJSON: req.AnalysisJSON,
			PolicyName:   policyNameFromRequest(c, req.Policy),
			EntityFilter: func(raw []byte) []byte {
				return filterAnalysisEntities(raw, s, c)
			},
		})
		if err != nil {
			switch {
			case errors.Is(err, domain.ErrEmptyText):
				st = http.StatusBadRequest
				return errJSON(c, http.StatusBadRequest, "VALIDATION", "text requis")
			case errors.Is(err, app.ErrPolicyPhaseBefore):
				if errors.Is(err, policy.ErrPolicyBlocked) {
					st = http.StatusUnprocessableEntity
					return errJSON(c, http.StatusUnprocessableEntity, "POLICY_BLOCKED", err.Error())
				}
				st = http.StatusBadRequest
				return errJSON(c, http.StatusBadRequest, "POLICY", err.Error())
			case errors.Is(err, app.ErrPolicyPhaseAfter):
				if errors.Is(err, policy.ErrPolicyBlocked) {
					st = http.StatusUnprocessableEntity
					return errJSON(c, http.StatusUnprocessableEntity, "POLICY_BLOCKED", err.Error())
				}
				st = http.StatusInternalServerError
				return errJSON(c, http.StatusInternalServerError, "POLICY", err.Error())
			default:
				st = http.StatusInternalServerError
				return errJSON(c, http.StatusInternalServerError, "ENGINE", err.Error())
			}
		}
		metricsRaw = res.RawJSON
		raw := res.RawJSON
		if res.Minimization != nil || res.PolicyReport != nil {
			return c.JSON(http.StatusOK, analyzePolicyResponse{
				Result:       json.RawMessage(raw),
				Minimization: res.Minimization,
				PolicyReport: res.PolicyReport,
			})
		}
		return c.JSON(http.StatusOK, AnalyzeResponse{Result: json.RawMessage(raw)})
	}
}

// AnalyzeBatch POST /v1/analyze/batch
// @Summary Batch analysis (pagination)
// @Tags analyze
// @Accept json
// @Produce json
// @Param request body AnalyzeBatchRequest true "List of texts"
// @Success 200 {object} AnalyzeBatchResponse
// @Router /v1/analyze/batch [post]
func AnalyzeBatch(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		started := time.Now()
		st := http.StatusOK
		analysisJSON := ""
		batchLen := 0
		var metricItems [][]byte
		defer func() {
			metrics.ObserveAnalyzeBatch("/v1/analyze/batch", http.MethodPost, st, batchLen, analysisJSON, started, metricItems)
		}()

		var req AnalyzeBatchRequest
		if err := c.Bind(&req); err != nil {
			st = http.StatusBadRequest
			return errJSON(c, http.StatusBadRequest, "INVALID_JSON", err.Error())
		}
		if len(req.Texts) == 0 {
			st = http.StatusBadRequest
			return errJSON(c, http.StatusBadRequest, "VALIDATION", "texts non vide requis")
		}
		page := req.Page
		if page < 1 {
			page = 1
		}
		ps := req.PageSize
		if ps < 1 {
			ps = 20
		}
		if ps > 100 {
			ps = 100
		}
		total := len(req.Texts)
		start := (page - 1) * ps
		if start > total {
			start = total
		}
		end := start + ps
		if end > total {
			end = total
		}
		slice := req.Texts[start:end]
		batchLen = len(slice)
		ctx, cancel := context.WithTimeout(c.Request().Context(), s.EngineTimeout*time.Duration(1+len(slice)/10))
		defer cancel()
		pname := policyNameFromRequest(c, req.Policy)
		var arr []json.RawMessage
		var err error
		if pname != "" && s.Policy != nil {
			err = bridge.WithCircuitBreaker(s.Breaker, func() error {
				return s.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
					for _, t := range slice {
						text := t
						var minErr error
						text, _, minErr = applyPolicyBeforeAnalyze(s, ctx, pname, text)
						if minErr != nil {
							return minErr
						}
						one, aerr := eng.Analyze(ctx, text, "")
						if aerr != nil {
							return aerr
						}
						raw := []byte(one)
						raw, _, aerr = applyPolicyAfterAnalyze(s, ctx, pname, raw)
						if aerr != nil {
							return aerr
						}
						metricItems = append(metricItems, append([]byte(nil), raw...))
						raw = filterAnalysisEntities(raw, s, c)
						arr = append(arr, json.RawMessage(raw))
					}
					return nil
				})
			})
			if err != nil {
				if errors.Is(err, policy.ErrPolicyBlocked) {
					st = http.StatusUnprocessableEntity
					return errJSON(c, http.StatusUnprocessableEntity, "POLICY_BLOCKED", err.Error())
				}
				st = http.StatusInternalServerError
				return errJSON(c, http.StatusInternalServerError, "ENGINE", err.Error())
			}
		} else {
			var raw string
			err = bridge.WithCircuitBreaker(s.Breaker, func() error {
				return s.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
					var e error
					raw, e = eng.AnalyzeBatch(ctx, slice)
					return e
				})
			})
			if err != nil {
				st = http.StatusInternalServerError
				return errJSON(c, http.StatusInternalServerError, "ENGINE", err.Error())
			}
			if err := json.Unmarshal([]byte(raw), &arr); err != nil {
				st = http.StatusInternalServerError
				return errJSON(c, http.StatusInternalServerError, "ENGINE", "batch JSON invalide")
			}
			for i := range arr {
				b, merr := arr[i].MarshalJSON()
				if merr != nil {
					continue
				}
				metricItems = append(metricItems, append([]byte(nil), b...))
				b = filterAnalysisEntities(b, s, c)
				arr[i] = json.RawMessage(b)
			}
		}
		return c.JSON(http.StatusOK, AnalyzeBatchResponse{
			Items:    arr,
			Total:    total,
			Page:     page,
			PageSize: ps,
			HasMore:  end < total,
		})
	}
}

// Anonymize POST /v1/anonymize
// @Summary Analyze and anonymize
// @Tags anonymize
// @Accept json
// @Produce json
// @Param request body AnonymizeRequest true "Text plus operator config JSON"
// @Success 200 {object} AnonymizeResponse
// @Router /v1/anonymize [post]
func Anonymize(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		st := http.StatusOK
		var resultRaw []byte
		defer func() {
			metrics.ObserveAnonymizeRequest("/v1/anonymize", http.MethodPost, st, resultRaw)
		}()

		var req AnonymizeRequest
		if err := c.Bind(&req); err != nil {
			st = http.StatusBadRequest
			return errJSON(c, http.StatusBadRequest, "INVALID_JSON", err.Error())
		}
		if strings.TrimSpace(req.Text) == "" {
			st = http.StatusBadRequest
			return errJSON(c, http.StatusBadRequest, "VALIDATION", "text requis")
		}
		pname := policyNameFromRequest(c, req.Policy)
		cfgJSON := req.ConfigJSON
		if pname != "" {
			merged, merr := mergePolicyAnonymizeConfig(s, pname, cfgJSON)
			if merr != nil {
				st = http.StatusBadRequest
				return errJSON(c, http.StatusBadRequest, "POLICY", merr.Error())
			}
			cfgJSON = merged
		}
		text := req.Text
		if pname != "" {
			ctx0, cancel0 := context.WithTimeout(c.Request().Context(), s.EngineTimeout)
			var minErr error
			text, _, minErr = applyPolicyBeforeAnalyze(s, ctx0, pname, text)
			cancel0()
			if minErr != nil {
				if errors.Is(minErr, policy.ErrPolicyBlocked) {
					st = http.StatusUnprocessableEntity
					return errJSON(c, http.StatusUnprocessableEntity, "POLICY_BLOCKED", minErr.Error())
				}
				st = http.StatusBadRequest
				return errJSON(c, http.StatusBadRequest, "POLICY", minErr.Error())
			}
		}
		ctx, cancel := context.WithTimeout(c.Request().Context(), s.EngineTimeout)
		defer cancel()
		var out string
		err := bridge.WithCircuitBreaker(s.Breaker, func() error {
			return s.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
				var e error
				out, e = eng.Anonymize(ctx, text, cfgJSON)
				return e
			})
		})
		if err != nil {
			st = http.StatusInternalServerError
			return errJSON(c, http.StatusInternalServerError, "ENGINE", err.Error())
		}
		registerPseudonymMapping(s, ctx, pname, req.SubjectID)
		resultRaw = []byte(out)
		return c.JSON(http.StatusOK, AnonymizeResponse{Result: json.RawMessage(out)})
	}
}

// Deanonymize POST /v1/deanonymize (admin)
// @Summary Deanonymization (admin only — extend FFI)
// @Tags admin
// @Security ApiKeyAuth
// @Accept json
// @Produce json
// @Param request body DeanonymizeRequest true "Payload"
// @Success 501 {object} DeanonymizeResponse
// @Router /v1/deanonymize [post]
func Deanonymize(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		var req DeanonymizeRequest
		if err := c.Bind(&req); err != nil {
			return errJSON(c, http.StatusBadRequest, "INVALID_JSON", err.Error())
		}
		ctx, cancel := context.WithTimeout(c.Request().Context(), s.EngineTimeout)
		defer cancel()
		body, _ := json.Marshal(req)
		var out string
		err := s.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
			var e error
			out, e = eng.Deanonymize(ctx, string(body))
			return e
		})
		if err != nil && out == "" {
			if errors.Is(err, bridge.ErrNotImplemented) {
				return c.JSON(http.StatusNotImplemented, DeanonymizeResponse{
					Code:    "NOT_IMPLEMENTED",
					Message: "aegis-ffi n’expose pas encore deanonymize ; étendre le crate aegis-ffi.",
				})
			}
			return errJSON(c, http.StatusInternalServerError, "ENGINE", err.Error())
		}
		if strings.TrimSpace(out) != "" {
			metrics.IncDeanonymize()
		}
		return c.JSON(http.StatusOK, DeanonymizeResponse{Text: out})
	}
}

// FalsePositiveFeedback POST /v1/feedback/false-positive — increments aegis_false_positive_reports_total.
func FalsePositiveFeedback() echo.HandlerFunc {
	return func(c echo.Context) error {
		metrics.IncFalsePositive("api")
		return c.NoContent(http.StatusNoContent)
	}
}

// ListRecognizers GET /v1/recognizers
// @Summary Active recognizers (gateway catalog)
// @Tags meta
// @Produce json
// @Success 200 {object} RecognizersResponse
// @Router /v1/recognizers [get]
func ListRecognizers() echo.HandlerFunc {
	return func(c echo.Context) error {
		rs := bridge.DefaultRecognizers()
		dto := make([]RecognizerDTO, len(rs))
		for i, r := range rs {
			dto[i] = RecognizerDTO{Name: r.Name, Kind: r.Kind, Enabled: r.Enabled}
		}
		return c.JSON(http.StatusOK, RecognizersResponse{Recognizers: dto})
	}
}

// ListEntityTypes GET /v1/entities
// @Summary Supported entity types
// @Tags meta
// @Produce json
// @Success 200 {object} EntityTypesResponse
// @Router /v1/entities [get]
func ListEntityTypes(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		all := bridge.SupportedEntityTypes()
		if s.Gateway != nil {
			p := auth.PrincipalFromEcho(c)
			if p != nil {
				var out []string
				for _, t := range all {
					if s.Gateway.RBAC.EntityTypeAllowed(p.Role, t) {
						out = append(out, t)
					}
				}
				all = out
			}
		}
		return c.JSON(http.StatusOK, EntityTypesResponse{EntityTypes: all})
	}
}

// PutConfig PUT /v1/config (admin)
// @Summary Hot-reload configuration (YAML merged into viper)
// @Tags admin
// @Security ApiKeyAuth
// @Accept json
// @Produce json
// @Param request body UpdateConfigRequest true "YAML fragment"
// @Success 200 {object} UpdateConfigResponse
// @Router /v1/config [put]
func PutConfig(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		var req UpdateConfigRequest
		if err := c.Bind(&req); err != nil {
			return errJSON(c, http.StatusBadRequest, "INVALID_JSON", err.Error())
		}
		if strings.TrimSpace(req.YAML) == "" {
			return errJSON(c, http.StatusBadRequest, "VALIDATION", "yaml requis")
		}
		if s.Loader == nil {
			return errJSON(c, http.StatusInternalServerError, "CONFIG", "loader absent")
		}
		if err := s.Loader.MergeYAML([]byte(req.YAML)); err != nil {
			return errJSON(c, http.StatusBadRequest, "CONFIG", err.Error())
		}
		return c.JSON(http.StatusOK, UpdateConfigResponse{Status: "merged"})
	}
}

// Health GET /v1/health
// @Summary Health check
// @Tags health
// @Produce json
// @Success 200 {object} HealthResponse
// @Router /v1/health [get]
func Health(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx, cancel := context.WithTimeout(c.Request().Context(), 2*time.Second)
		defer cancel()
		ver := ""
		_ = s.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
			ver = eng.Version()
			return nil
		})
		return c.JSON(http.StatusOK, HealthResponse{Status: "ok", RustVersion: ver})
	}
}

// AuditExport GET /v1/audit/export — formats jsonl (default), cef, leef.
func AuditExport(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		if s.Gateway == nil || s.Gateway.Audit == nil {
			return errJSON(c, http.StatusNotFound, "AUDIT", "export indisponible")
		}
		path := s.Gateway.Config.Audit.FilePath
		if path == "" {
			return errJSON(c, http.StatusNotFound, "AUDIT", "chemin fichier absent")
		}
		lines, err := audit.ScanLinesForward(path, 2000)
		if err != nil {
			return errJSON(c, http.StatusInternalServerError, "AUDIT", err.Error())
		}
		format := strings.ToLower(strings.TrimSpace(c.QueryParam("format")))
		c.Response().Header().Set(echo.HeaderContentType, echo.MIMETextPlainCharsetUTF8)
		w := c.Response().Writer
		for _, line := range lines {
			if format == "cef" || format == "leef" {
				var ent audit.Entry
				if json.Unmarshal([]byte(line), &ent) != nil {
					continue
				}
				if format == "cef" {
					_, _ = fmt.Fprintln(w, audit.ToCEF(ent))
				} else {
					_, _ = fmt.Fprintln(w, audit.ToLEEF(ent))
				}
				continue
			}
			_, _ = fmt.Fprintln(w, line)
		}
		return nil
	}
}

func errJSON(c echo.Context, status int, code, msg string) error {
	rid, _ := c.Get("request_id").(string)
	return c.JSON(status, ErrorBody{Code: code, Message: msg, RequestID: rid})
}

func filterAnalysisEntities(raw []byte, s *Services, c echo.Context) []byte {
	if s.Gateway == nil {
		return raw
	}
	p := auth.PrincipalFromEcho(c)
	if p == nil {
		return raw
	}
	allow := func(entityType string) bool {
		return s.Gateway.RBAC.EntityTypeAllowed(p.Role, entityType)
	}
	return filterAnalysisJSON(raw, allow)
}

func filterAnalysisJSON(raw []byte, allow func(string) bool) []byte {
	var m map[string]interface{}
	if err := json.Unmarshal(raw, &m); err != nil {
		return raw
	}
	ent, ok := m["entities"].([]interface{})
	if !ok {
		return raw
	}
	var kept []interface{}
	for _, e := range ent {
		em, ok := e.(map[string]interface{})
		if !ok {
			continue
		}
		typ, _ := em["entity_type"].(string)
		if allow(typ) {
			kept = append(kept, e)
		}
	}
	m["entities"] = kept
	b, err := json.Marshal(m)
	if err != nil {
		return raw
	}
	return b
}

// RequireAdmin checks X-API-Key ∈ adminKeys (or X-Aegis-Role: admin with no key when no keys are configured — dev mode).
func RequireAdmin(adminKeys []string, apiKeyHeader string) echo.MiddlewareFunc {
	set := map[string]struct{}{}
	for _, k := range adminKeys {
		k = strings.TrimSpace(k)
		if k != "" {
			set[k] = struct{}{}
		}
	}
	if apiKeyHeader == "" {
		apiKeyHeader = "X-API-Key"
	}
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			role := strings.EqualFold(c.Request().Header.Get("X-Aegis-Role"), "admin")
			key := strings.TrimSpace(c.Request().Header.Get(apiKeyHeader))
			if len(set) == 0 {
				if role {
					return next(c)
				}
				return errJSON(c, http.StatusForbidden, "FORBIDDEN", "admin: définir admin_api_keys ou X-Aegis-Role: admin (dev)")
			}
			if _, ok := set[key]; ok {
				return next(c)
			}
			if role && key == "" {
				return errJSON(c, http.StatusForbidden, "FORBIDDEN", "clé admin requise")
			}
			return errJSON(c, http.StatusForbidden, "FORBIDDEN", "clé admin invalide")
		}
	}
}
