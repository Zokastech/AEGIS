// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"errors"
	"os"

	"github.com/zokastech/aegis/aegis-gateway/audit"
	"github.com/zokastech/aegis/aegis-gateway/auth"
	"github.com/zokastech/aegis/aegis-gateway/auth/apikey"
	"github.com/zokastech/aegis/aegis-gateway/config"
	"github.com/zokastech/aegis/aegis-gateway/rbac"
)

// GatewaySecurity bundles the HTTP gateway “secure by default” components.
type GatewaySecurity struct {
	Config       config.SecurityConfig
	APIKeyHeader string
	APIKeys      *apikey.FileStore
	RBAC         *rbac.YAMLStore
	Audit        *audit.FileLogger
	JWT          *auth.JWTValidator
	OIDC         *auth.OIDCVerifier
	Deanon       *DeanonGuard
	LegacyAdmin  []string
}

// BootstrapGatewaySecurity loads API keys, RBAC YAML, file audit, JWT/OIDC validators.
func BootstrapGatewaySecurity(cfg config.Config) (*GatewaySecurity, error) {
	sc := cfg.Security
	header := cfg.APIKeyHeader
	if header == "" {
		header = "X-API-Key"
	}

	pepper := sc.APIKeys.Pepper
	if pepper == "" {
		pepper = os.Getenv("AEGIS_API_KEY_PEPPER")
	}
	keys, err := apikey.LoadFileStore(sc.APIKeys.FilePath, pepper)
	if err != nil {
		return nil, err
	}

	var rb *rbac.YAMLStore
	if sc.RBAC.FilePath != "" {
		rb, err = rbac.LoadYAML(sc.RBAC.FilePath)
		if err != nil {
			if errors.Is(err, os.ErrNotExist) {
				rb = rbac.EmptyYAMLStore()
			} else {
				return nil, err
			}
		}
	} else {
		rb = rbac.EmptyYAMLStore()
	}

	var al *audit.FileLogger
	if (sc.Audit.Backend == "file" || sc.Audit.Backend == "") && sc.Audit.FilePath != "" {
		al = audit.NewFileLogger(sc.Audit.FilePath, sc.Audit.RotateMaxMB, sc.Audit.ArchiveDir)
	}

	g := &GatewaySecurity{
		Config:       sc,
		APIKeyHeader: header,
		APIKeys:      keys,
		RBAC:         rb,
		Audit:        al,
		JWT:          auth.NewJWTValidator(sc.JWT),
		OIDC:         auth.NewOIDCVerifier(sc.OIDC),
		Deanon:       NewDeanonGuard(sc.Deanon),
		LegacyAdmin:  cfg.AdminAPIKeys,
	}
	return g, nil
}
