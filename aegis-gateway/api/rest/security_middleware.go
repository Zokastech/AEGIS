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
	p, err := g.Resolve(r.Context(), Credentials{
		ClientCN:      auth.ClientCN(r),
		Authorization: r.Header.Get(echo.HeaderAuthorization),
		APIKey:        r.Header.Get(g.APIKeyHeader),
	})
	if err != nil {
		return nil, toEchoErr(err)
	}
	return p, nil
}

func toEchoErr(err error) error {
	if ae, ok := err.(*AuthError); ok {
		return echo.NewHTTPError(ae.Status, ae.Message)
	}
	return err
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
			if err := g.Authorize(auth.PrincipalFromEcho(c), perm); err != nil {
				return toEchoErr(err)
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
	h1, h2 := g.ApprovalHeaders()
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if g.Config.Development.DisableAuth {
				return legacyAdminDeanon(c, g.APIKeyHeader, g.LegacyAdmin, next)
			}
			p, err := g.ResolveDualApproval(c.Request().Header.Get(h1), c.Request().Header.Get(h2))
			if err != nil {
				return toEchoErr(err)
			}
			auth.SetPrincipal(c, p)
			err = next(c)
			if err == nil && c.Response().Status < 400 {
				g.Deanon.RecordSuccess()
				rid, _ := c.Get("request_id").(string)
				go g.sendDeanonAlert(rid, c.Request().URL.Path, p.Subject)
			}
			return err
		}
	}
}

func legacyAdminDeanon(c echo.Context, apiKeyHeader string, adminKeys []string, next echo.HandlerFunc) error {
	if apiKeyHeader == "" {
		apiKeyHeader = "X-API-Key"
	}
	key := strings.TrimSpace(c.Request().Header.Get(apiKeyHeader))
	if len(nonEmpty(adminKeys)) == 0 {
		if roleHeaderAllowed() && strings.EqualFold(c.Request().Header.Get("X-Aegis-Role"), "admin") {
			auth.SetPrincipal(c, &auth.Principal{Subject: "dev:header-admin", Role: rbac.RoleAdmin, AuthMethod: "disabled"})
			return next(c)
		}
		return echo.NewHTTPError(http.StatusForbidden, "admin: définir admin_api_keys (ou AEGIS_DEV_ALLOW_ROLE_HEADER=1 + X-Aegis-Role: admin en dev)")
	}
	if adminKeyMatch(adminKeys, key) {
		auth.SetPrincipal(c, &auth.Principal{Subject: "dev:admin-key", Role: rbac.RoleAdmin, AuthMethod: "api_key"})
		return next(c)
	}
	if key == "" {
		return echo.NewHTTPError(http.StatusForbidden, "clé admin requise")
	}
	return echo.NewHTTPError(http.StatusForbidden, "clé admin invalide")
}

// sendDeanonAlert posts a webhook alert (approvers = "deanon:<id1>+<id2>" subject, never secrets).
func (g *GatewaySecurity) sendDeanonAlert(rid, path, subject string) {
	url := strings.TrimSpace(g.Config.Alerts.WebhookURL)
	if url == "" {
		return
	}
	approvers := strings.Split(strings.TrimPrefix(subject, "deanon:"), "+")
	payload := map[string]interface{}{
		"event":      "aegis.deanonymize",
		"severity":   "high",
		"timestamp":  time.Now().UTC().Format(time.RFC3339Nano),
		"request_id": rid,
		"approvers":  approvers,
		"path":       path,
	}
	b, _ := json.Marshal(payload)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(b))
	if err != nil {
		return
	}
	req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	resp, err := alertHTTPClient.Do(req)
	if err == nil {
		_ = resp.Body.Close()
	}
}

// alertHTTPClient is used for outbound alert webhooks: bounded timeout, no redirects followed
// (a redirect could bounce the alert to an unintended/internal host).
var alertHTTPClient = &http.Client{
	Timeout: 5 * time.Second,
	CheckRedirect: func(*http.Request, []*http.Request) error {
		return http.ErrUseLastResponse
	},
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
			if err := g.Authorize(p, rbac.PermMetricsView); err != nil {
				return echo.NewHTTPError(http.StatusForbidden, "metrics: permission refusée")
			}
			return next(c)
		}
	}
}
