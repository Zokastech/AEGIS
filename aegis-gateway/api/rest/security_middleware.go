// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/zokastech/aegis/aegis-gateway/audit"
	"github.com/zokastech/aegis/aegis-gateway/auth"
	"github.com/zokastech/aegis/aegis-gateway/rbac"
)

func (g *GatewaySecurity) effectiveAPIKeyRole(id, fileRole string) string {
	if r, ok := g.RBAC.RoleForAPIKey(id); ok && r != "" {
		return r
	}
	return fileRole
}

func (g *GatewaySecurity) isAdminEffectiveRole(id, fileRole string) bool {
	r := g.effectiveAPIKeyRole(id, fileRole)
	return r == rbac.RoleAdmin
}

// ResolvePrincipal authenticates the request (mTLS CN > Bearer OIDC/JWT > API key).
func (g *GatewaySecurity) ResolvePrincipal(c echo.Context) (*auth.Principal, error) {
	r := c.Request()

	if cn := auth.ClientCN(r); cn != "" {
		role, ok := g.RBAC.RoleForMTLSCN(cn)
		if !ok || role == "" {
			return nil, echo.NewHTTPError(http.StatusForbidden, "mTLS: CN non mappé dans rbac")
		}
		return &auth.Principal{
			Subject:    "mtls:" + cn,
			Role:       role,
			AuthMethod: "mtls",
		}, nil
	}

	authz := strings.TrimSpace(r.Header.Get(echo.HeaderAuthorization))
	if len(authz) > 7 && strings.EqualFold(authz[:7], "Bearer ") {
		raw := strings.TrimSpace(authz[7:])
		if g.Config.OIDC.Enabled {
			if sub, err := g.OIDC.Verify(r.Context(), raw); err == nil && sub != "" {
				role, ok := g.RBAC.RoleForJWTSubject(sub)
				if !ok || role == "" {
					return nil, echo.NewHTTPError(http.StatusForbidden, "OIDC: sujet sans rôle RBAC")
				}
				return &auth.Principal{Subject: "oidc:" + sub, Role: role, AuthMethod: "oidc"}, nil
			}
		}
		if g.Config.JWT.Enabled {
			sub, err := g.JWT.ParseAndValidate(raw)
			if err == nil && sub != "" {
				role, ok := g.RBAC.RoleForJWTSubject(sub)
				if !ok || role == "" {
					return nil, echo.NewHTTPError(http.StatusForbidden, "JWT: sujet sans rôle RBAC")
				}
				return &auth.Principal{Subject: "jwt:" + sub, Role: role, AuthMethod: "jwt"}, nil
			}
		}
	}

	key := strings.TrimSpace(r.Header.Get(g.APIKeyHeader))
	if key != "" {
		id, roleFile, ok := g.APIKeys.Validate(key)
		if !ok {
			return nil, echo.NewHTTPError(http.StatusUnauthorized, "clé API invalide ou révoquée")
		}
		role := g.effectiveAPIKeyRole(id, roleFile)
		if role == "" {
			return nil, echo.NewHTTPError(http.StatusForbidden, "clé API sans rôle")
		}
		return &auth.Principal{
			Subject:    "api_key:" + id,
			KeyID:      id,
			Role:       role,
			AuthMethod: "api_key",
		}, nil
	}

	return nil, echo.NewHTTPError(http.StatusUnauthorized, "authentification requise")
}

func isPublicPath(path string, list []string) bool {
	for _, p := range list {
		if p == path {
			return true
		}
	}
	return false
}

// AuthMiddleware enforces auth except public paths and POST /v1/deanonymize (handled by DeanonymizeAccess).
func (g *GatewaySecurity) AuthMiddleware() echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if g.Config.Development.DisableAuth {
				auth.SetPrincipal(c, &auth.Principal{
					Subject:    "dev:bypass",
					Role:       rbac.RoleAdmin,
					AuthMethod: "disabled",
				})
				return next(c)
			}
			path := c.Path()
			if path == "" {
				path = c.Request().URL.Path
			}
			if isPublicPath(path, g.Config.PublicPaths) {
				return next(c)
			}
			if path == "/v1/deanonymize" && c.Request().Method == http.MethodPost {
				return next(c)
			}
			p, err := g.ResolvePrincipal(c)
			if err != nil {
				return err
			}
			auth.SetPrincipal(c, p)
			return next(c)
		}
	}
}

// RequirePermission enforces RBAC for a business permission.
func (g *GatewaySecurity) RequirePermission(perm string) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if g.Config.Development.DisableAuth {
				return next(c)
			}
			p := auth.PrincipalFromEcho(c)
			if p == nil {
				return echo.NewHTTPError(http.StatusForbidden, "principal manquant")
			}
			perms := g.RBAC.PermissionsForRole(p.Role)
			if !rbac.Can(perms, perm) {
				return echo.NewHTTPError(http.StatusForbidden, "permission refusée")
			}
			return next(c)
		}
	}
}

// AuditMiddleware logs each request after handling.
func (g *GatewaySecurity) AuditMiddleware() echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if g.Audit == nil {
				return next(c)
			}
			err := next(c)
			p := auth.PrincipalFromEcho(c)
			actor := "anonymous"
			authM := ""
			if p != nil {
				actor = p.Subject
				authM = p.AuthMethod
			}
			rid, _ := c.Get("request_id").(string)
			status := c.Response().Status
			ok := status < 400
			if err != nil {
				if he, y := err.(*echo.HTTPError); y {
					if he.Code > 0 {
						status = he.Code
					}
					ok = he.Code < 400
				} else {
					ok = false
				}
			}
			path := c.Path()
			if path == "" {
				path = c.Request().URL.Path
			}
			ent := audit.Entry{
				TimestampRFC3339: time.Now().UTC().Format(time.RFC3339Nano),
				Actor:            actor,
				AuthMethod:       authM,
				Action:           c.Request().Method + " " + path,
				Endpoint:         c.Request().URL.Path,
				Method:           c.Request().Method,
				RequestID:        rid,
				Success:          ok,
				StatusCode:       status,
			}
			_ = g.Audit.Append(ent)
			return err
		}
	}
}

// DeanonymizeAccess double approbation admin + rate limit + circuit + alerte.
func (g *GatewaySecurity) DeanonymizeAccess() echo.MiddlewareFunc {
	h1 := g.Config.Deanon.ApprovalHeader1
	h2 := g.Config.Deanon.ApprovalHeader2
	if h1 == "" {
		h1 = "X-Aegis-Approval-1"
	}
	if h2 == "" {
		h2 = "X-Aegis-Approval-2"
	}
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if g.Config.Development.DisableAuth {
				return legacyAdminDeanon(c, g.APIKeyHeader, g.LegacyAdmin, next)
			}
			s1 := strings.TrimSpace(c.Request().Header.Get(h1))
			s2 := strings.TrimSpace(c.Request().Header.Get(h2))
			id1, r1, ok1 := g.APIKeys.Validate(s1)
			id2, r2, ok2 := g.APIKeys.Validate(s2)
			if !ok1 || !ok2 {
				return echo.NewHTTPError(http.StatusForbidden, "deanonymize: deux clés API admin valides requises (en-têtes d’approbation)")
			}
			if id1 == id2 {
				return echo.NewHTTPError(http.StatusForbidden, "deanonymize: les deux approbateurs doivent être des clés distinctes")
			}
			if !g.isAdminEffectiveRole(id1, r1) || !g.isAdminEffectiveRole(id2, r2) {
				return echo.NewHTTPError(http.StatusForbidden, "deanonymize: rôle admin requis pour les deux clés")
			}
			if err := g.Deanon.Allow(id1, id2); err != nil {
				return err
			}
			auth.SetPrincipal(c, &auth.Principal{
				Subject:    "deanon:" + id1 + "+" + id2,
				KeyID:      id1,
				Role:       rbac.RoleAdmin,
				AuthMethod: "dual_api_key",
			})
			err := next(c)
			if err == nil && c.Response().Status < 400 {
				g.Deanon.RecordSuccess()
				go g.sendDeanonAlert(c, id1, id2)
			}
			return err
		}
	}
}

func legacyAdminDeanon(c echo.Context, apiKeyHeader string, adminKeys []string, next echo.HandlerFunc) error {
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
	role := strings.EqualFold(c.Request().Header.Get("X-Aegis-Role"), "admin")
	key := strings.TrimSpace(c.Request().Header.Get(apiKeyHeader))
	if len(set) == 0 {
		if role {
			auth.SetPrincipal(c, &auth.Principal{Subject: "dev:header-admin", Role: rbac.RoleAdmin, AuthMethod: "disabled"})
			return next(c)
		}
		return echo.NewHTTPError(http.StatusForbidden, "admin: définir admin_api_keys ou X-Aegis-Role: admin (dev)")
	}
	if _, ok := set[key]; ok {
		auth.SetPrincipal(c, &auth.Principal{Subject: "dev:admin-key", Role: rbac.RoleAdmin, AuthMethod: "api_key"})
		return next(c)
	}
	if role && key == "" {
		return echo.NewHTTPError(http.StatusForbidden, "clé admin requise")
	}
	return echo.NewHTTPError(http.StatusForbidden, "clé admin invalide")
}

func (g *GatewaySecurity) sendDeanonAlert(c echo.Context, id1, id2 string) {
	url := strings.TrimSpace(g.Config.Alerts.WebhookURL)
	if url == "" {
		return
	}
	rid, _ := c.Get("request_id").(string)
	payload := map[string]interface{}{
		"event":      "aegis.deanonymize",
		"severity":   "high",
		"timestamp":  time.Now().UTC().Format(time.RFC3339Nano),
		"request_id": rid,
		"approvers":  []string{id1, id2},
		"path":       c.Request().URL.Path,
	}
	b, _ := json.Marshal(payload)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(b))
	if err != nil {
		return
	}
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	_, _ = http.DefaultClient.Do(req)
}

// MetricsAccess protects /metrics in production.
func (g *GatewaySecurity) MetricsAccess() echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if !g.Config.Production || g.Config.Development.DisableAuth {
				return next(c)
			}
			p, err := g.ResolvePrincipal(c)
			if err != nil {
				return err
			}
			perms := g.RBAC.PermissionsForRole(p.Role)
			if !rbac.Can(perms, rbac.PermMetricsView) {
				return echo.NewHTTPError(http.StatusForbidden, "metrics: permission refusée")
			}
			return next(c)
		}
	}
}
