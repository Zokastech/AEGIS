// AEGIS — zokastech.fr — Apache 2.0 / MIT

package auth

import (
	"context"
	"errors"
	"strings"
	"sync"

	"github.com/coreos/go-oidc/v3/oidc"
	"github.com/zokastech/aegis/aegis-gateway/config"
)

// OIDCVerifier validates OIDC access tokens (Keycloak, Auth0, Azure AD, etc.).
type OIDCVerifier struct {
	cfg      config.SecurityOIDCConfig
	mu       sync.Mutex
	verifier *oidc.IDTokenVerifier
}

func NewOIDCVerifier(cfg config.SecurityOIDCConfig) *OIDCVerifier {
	return &OIDCVerifier{cfg: cfg}
}

func (o *OIDCVerifier) lazy(ctx context.Context) error {
	if !o.cfg.Enabled || o.cfg.IssuerURL == "" || o.cfg.ClientID == "" {
		return errors.New("oidc: non configuré")
	}
	o.mu.Lock()
	defer o.mu.Unlock()
	if o.verifier != nil {
		return nil
	}
	p, err := oidc.NewProvider(ctx, strings.TrimSuffix(o.cfg.IssuerURL, "/"))
	if err != nil {
		return err
	}
	o.verifier = p.Verifier(&oidc.Config{ClientID: o.cfg.ClientID})
	return nil
}

// Verify returns the OIDC token subject.
func (o *OIDCVerifier) Verify(ctx context.Context, rawToken string) (sub string, err error) {
	if err := o.lazy(ctx); err != nil {
		return "", err
	}
	idt, err := o.verifier.Verify(ctx, rawToken)
	if err != nil {
		return "", err
	}
	var claims struct {
		Sub string `json:"sub"`
	}
	if err := idt.Claims(&claims); err != nil {
		return "", err
	}
	if claims.Sub == "" {
		return "", errors.New("oidc: sub vide")
	}
	return claims.Sub, nil
}
