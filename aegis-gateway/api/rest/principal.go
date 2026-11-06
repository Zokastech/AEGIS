// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"context"
	"net/http"
	"strings"

	"github.com/zokastech/aegis/aegis-gateway/auth"
	"github.com/zokastech/aegis/aegis-gateway/rbac"
)

// Credentials are the transport-agnostic authentication inputs (REST headers or gRPC metadata).
type Credentials struct {
	// ClientCN is the CN of a verified mTLS client certificate ("" if none).
	ClientCN string
	// Authorization is the raw Authorization header / metadata value.
	Authorization string
	// APIKey is the raw API key header / metadata value.
	APIKey string
}

// AuthError carries an HTTP status so each transport can map it (REST → HTTP, gRPC → codes).
type AuthError struct {
	Status  int
	Message string
}

func (e *AuthError) Error() string { return e.Message }

func authErr(status int, msg string) *AuthError { return &AuthError{Status: status, Message: msg} }

// Resolve authenticates credentials (mTLS CN > Bearer OIDC/JWT > API key).
func (g *GatewaySecurity) Resolve(ctx context.Context, cr Credentials) (*auth.Principal, error) {
	if cn := strings.TrimSpace(cr.ClientCN); cn != "" {
		role, ok := g.RBAC.RoleForMTLSCN(cn)
		if !ok || role == "" {
			return nil, authErr(http.StatusForbidden, "mTLS: CN non mappé dans rbac")
		}
		return &auth.Principal{Subject: "mtls:" + cn, Role: role, AuthMethod: "mtls"}, nil
	}

	authz := strings.TrimSpace(cr.Authorization)
	if len(authz) > 7 && strings.EqualFold(authz[:7], "Bearer ") {
		raw := strings.TrimSpace(authz[7:])
		if g.Config.OIDC.Enabled && g.OIDC != nil {
			if sub, err := g.OIDC.Verify(ctx, raw); err == nil && sub != "" {
				role, ok := g.RBAC.RoleForJWTSubject(sub)
				if !ok || role == "" {
					return nil, authErr(http.StatusForbidden, "OIDC: sujet sans rôle RBAC")
				}
				return &auth.Principal{Subject: "oidc:" + sub, Role: role, AuthMethod: "oidc"}, nil
			}
		}
		if g.Config.JWT.Enabled && g.JWT != nil {
			if sub, err := g.JWT.ParseAndValidate(raw); err == nil && sub != "" {
				role, ok := g.RBAC.RoleForJWTSubject(sub)
				if !ok || role == "" {
					return nil, authErr(http.StatusForbidden, "JWT: sujet sans rôle RBAC")
				}
				return &auth.Principal{Subject: "jwt:" + sub, Role: role, AuthMethod: "jwt"}, nil
			}
		}
		if strings.TrimSpace(cr.APIKey) == "" {
			return nil, authErr(http.StatusUnauthorized, "jeton Bearer invalide ou expiré")
		}
	}

	if key := strings.TrimSpace(cr.APIKey); key != "" {
		id, roleFile, ok := g.APIKeys.Validate(key)
		if !ok {
			return nil, authErr(http.StatusUnauthorized, "clé API invalide ou révoquée")
		}
		role := g.effectiveAPIKeyRole(id, roleFile)
		if role == "" {
			return nil, authErr(http.StatusForbidden, "clé API sans rôle")
		}
		return &auth.Principal{Subject: "api_key:" + id, KeyID: id, Role: role, AuthMethod: "api_key"}, nil
	}

	return nil, authErr(http.StatusUnauthorized, "authentification requise")
}

// Authorize checks that the principal's role grants perm.
func (g *GatewaySecurity) Authorize(p *auth.Principal, perm string) error {
	if p == nil {
		return authErr(http.StatusForbidden, "principal manquant")
	}
	if !rbac.Can(g.RBAC.PermissionsForRole(p.Role), perm) {
		return authErr(http.StatusForbidden, "permission refusée")
	}
	return nil
}

// ResolveDualApproval validates two distinct admin API keys and applies the deanonymize guard
// (rate limit per approver pair + circuit breaker).
func (g *GatewaySecurity) ResolveDualApproval(secret1, secret2 string) (*auth.Principal, error) {
	s1 := strings.TrimSpace(secret1)
	s2 := strings.TrimSpace(secret2)
	id1, r1, ok1 := g.APIKeys.Validate(s1)
	id2, r2, ok2 := g.APIKeys.Validate(s2)
	if !ok1 || !ok2 {
		return nil, authErr(http.StatusForbidden, "deanonymize: deux clés API admin valides requises (en-têtes d’approbation)")
	}
	if id1 == id2 {
		return nil, authErr(http.StatusForbidden, "deanonymize: les deux approbateurs doivent être des clés distinctes")
	}
	if !g.isAdminEffectiveRole(id1, r1) || !g.isAdminEffectiveRole(id2, r2) {
		return nil, authErr(http.StatusForbidden, "deanonymize: rôle admin requis pour les deux clés")
	}
	if err := g.Deanon.Allow(id1, id2); err != nil {
		return nil, err
	}
	return &auth.Principal{
		Subject:    "deanon:" + id1 + "+" + id2,
		KeyID:      id1,
		Role:       rbac.RoleAdmin,
		AuthMethod: "dual_api_key",
	}, nil
}

// ApprovalHeaders returns the configured dual-approval header names.
func (g *GatewaySecurity) ApprovalHeaders() (string, string) {
	h1 := g.Config.Deanon.ApprovalHeader1
	h2 := g.Config.Deanon.ApprovalHeader2
	if h1 == "" {
		h1 = "X-Aegis-Approval-1"
	}
	if h2 == "" {
		h2 = "X-Aegis-Approval-2"
	}
	return h1, h2
}
