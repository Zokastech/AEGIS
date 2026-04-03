// AEGIS — zokastech.fr — Apache 2.0 / MIT

package auth

import (
	"errors"
	"fmt"
	"strings"

	"github.com/golang-jwt/jwt/v5"
	"github.com/zokastech/aegis/aegis-gateway/config"
)

// JWTValidator verifies a Bearer JWT (HS256 or common claims parsing).
type JWTValidator struct {
	cfg config.SecurityJWTConfig
}

func NewJWTValidator(cfg config.SecurityJWTConfig) *JWTValidator {
	return &JWTValidator{cfg: cfg}
}

// ParseAndValidate returns the subject (sub) when the token is valid.
func (v *JWTValidator) ParseAndValidate(raw string) (sub string, err error) {
	if !v.cfg.Enabled || strings.TrimSpace(v.cfg.HMACSecret) == "" {
		return "", errors.New("jwt: désactivé ou hmac_secret vide")
	}
	token, err := jwt.Parse(raw, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("jwt: algo inattendu %v", t.Header["alg"])
		}
		return []byte(v.cfg.HMACSecret), nil
	})
	if err != nil || !token.Valid {
		return "", err
	}
	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok {
		return "", errors.New("jwt: claims invalides")
	}
	if iss, _ := claims["iss"].(string); v.cfg.Issuer != "" && iss != v.cfg.Issuer {
		return "", errors.New("jwt: iss invalide")
	}
	if aud, ok := claims["aud"].(string); v.cfg.Audience != "" && ok && aud != v.cfg.Audience {
		return "", errors.New("jwt: aud invalide")
	}
	sub, _ = claims["sub"].(string)
	if sub == "" {
		return "", errors.New("jwt: sub manquant")
	}
	return sub, nil
}
