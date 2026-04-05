// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	gwconfig "github.com/zokastech/aegis/aegis-gateway/config"
	"github.com/zokastech/aegis/aegis-gateway/rbac"
)

// AttachRoutes registers all /v1 + /metrics routes (+ livez/readyz when the security layer is active).
func AttachRoutes(e *echo.Echo, s *Services, cfg gwconfig.Config) {
	if s.Gateway != nil {
		attachSecuredRoutes(e, s, cfg)
		return
	}
	attachLegacyRoutes(e, s, cfg)
}

func attachLegacyRoutes(e *echo.Echo, s *Services, cfg gwconfig.Config) {
	e.GET("/metrics", echo.WrapHandler(promhttp.Handler()))

	v1 := e.Group("/v1")
	v1.POST("/analyze", Analyze(s))
	v1.POST("/analyze/batch", AnalyzeBatch(s))
	v1.POST("/anonymize", Anonymize(s))
	v1.POST("/feedback/false-positive", FalsePositiveFeedback())

	admin := RequireAdmin(cfg.AdminAPIKeys, cfg.APIKeyHeader)
	v1.POST("/deanonymize", Deanonymize(s), admin)
	v1.PUT("/config", PutConfig(s), admin)

	v1.GET("/recognizers", ListRecognizers())
	v1.GET("/entities", ListEntityTypes(s))
	v1.GET("/health", Health(s))
	v1.GET("/openapi.yaml", OpenAPIYAML)

	if s.Policy != nil {
		v1.GET("/policies", ListPolicies(s))
		v1.GET("/policy/dpia", GetDPIA(s))
		adminPol := RequireAdmin(cfg.AdminAPIKeys, cfg.APIKeyHeader)
		v1.DELETE("/subjects/:id", DeleteSubject(s), adminPol)
	}
}

func attachSecuredRoutes(e *echo.Echo, s *Services, cfg gwconfig.Config) {
	g := s.Gateway

	e.GET("/livez", func(c echo.Context) error { return c.NoContent(http.StatusNoContent) })
	e.GET("/readyz", func(c echo.Context) error { return c.NoContent(http.StatusNoContent) })
	e.GET("/metrics", echo.WrapHandler(promhttp.Handler()), g.MetricsAccess())

	v1 := e.Group("/v1", g.AuthMiddleware(), g.AuditMiddleware())
	v1.POST("/analyze", Analyze(s), g.RequirePermission(rbac.PermAnalyzeExecute))
	v1.POST("/analyze/batch", AnalyzeBatch(s), g.RequirePermission(rbac.PermAnalyzeBatch))
	v1.POST("/anonymize", Anonymize(s), g.RequirePermission(rbac.PermAnonymizeExecute))
	v1.POST("/feedback/false-positive", FalsePositiveFeedback(), g.RequirePermission(rbac.PermAnalyzeExecute))

	v1.POST("/deanonymize", Deanonymize(s), g.DeanonymizeAccess())
	v1.PUT("/config", PutConfig(s), g.RequirePermission(rbac.PermConfigWrite))

	v1.GET("/recognizers", ListRecognizers(), g.RequirePermission(rbac.PermMetaRead))
	v1.GET("/entities", ListEntityTypes(s), g.RequirePermission(rbac.PermMetaRead))
	v1.GET("/health", Health(s), g.RequirePermission(rbac.PermMetaRead))
	v1.GET("/openapi.yaml", OpenAPIYAML, g.RequirePermission(rbac.PermMetaRead))
	v1.GET("/audit/export", AuditExport(s), g.RequirePermission(rbac.PermAuditExport))

	if s.Policy != nil {
		v1.GET("/policies", ListPolicies(s), g.RequirePermission(rbac.PermPolicyList))
		v1.GET("/policy/dpia", GetDPIA(s), g.RequirePermission(rbac.PermPolicyDPIA))
		v1.DELETE("/subjects/:id", DeleteSubject(s), g.RequirePermission(rbac.PermSubjectsErase))
	}
}
