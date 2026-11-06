// AEGIS — zokastech.fr — Apache 2.0 / MIT

package auth

import (
	"errors"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/zokastech/aegis/aegis-gateway/config"
)

// JWTValidator validates HMAC-signed JWTs (HS256/384/512).
type JWTValidator struct {
	cfg config.SecurityJWTConfig
}

// NewJWTValidator builds a validator from security config.
func NewJWTValidator(cfg config.SecurityJWTConfig) *JWTValidator {
	return &JWTValidator{cfg: cfg}
}

// ParseAndValidate verifies signature, algorithm, exp (required), nbf/iat, iss and aud.
// aud may be a string or an array (RFC 7519 §4.1.3); when an audience is configured it must be present.
func (v *JWTValidator) ParseAndValidate(raw string) (sub string, err error) {
	if !v.cfg.Enabled || strings.TrimSpace(v.cfg.HMACSecret) == "" {
		return "", errors.New("jwt: désactivé ou hmac_secret vide")
	}
	opts := []jwt.ParserOption{
		jwt.WithValidMethods([]string{"HS256", "HS384", "HS512"}),
		jwt.WithExpirationRequired(),
		jwt.WithIssuedAt(),
		jwt.WithLeeway(30 * time.Second),
	}
	if v.cfg.Issuer != "" {
		opts = append(opts, jwt.WithIssuer(v.cfg.Issuer))
	}
	if v.cfg.Audience != "" {
		opts = append(opts, jwt.WithAudience(v.cfg.Audience))
	}
	claims := jwt.MapClaims{}
	token, err := jwt.ParseWithClaims(raw, claims, func(t *jwt.Token) (interface{}, error) {
		return []byte(v.cfg.HMACSecret), nil
	}, opts...)
	if err != nil {
		return "", err
	}
	if !token.Valid {
		return "", errors.New("jwt: jeton invalide")
	}
	sub, err = claims.GetSubject()
	if err != nil || sub == "" {
		return "", errors.New("jwt: sub manquant")
	}
	return sub, nil
}
