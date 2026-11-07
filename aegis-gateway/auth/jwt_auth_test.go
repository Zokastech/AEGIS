// AEGIS — zokastech.fr — Apache 2.0 / MIT

package auth

import (
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/zokastech/aegis/aegis-gateway/config"
)

const secret = "0123456789abcdef0123456789abcdef"

func sign(t *testing.T, method jwt.SigningMethod, key interface{}, claims jwt.MapClaims) string {
	t.Helper()
	s, err := jwt.NewWithClaims(method, claims).SignedString(key)
	if err != nil {
		t.Fatal(err)
	}
	return s
}

func validator(iss, aud string) *JWTValidator {
	return NewJWTValidator(config.SecurityJWTConfig{Enabled: true, HMACSecret: secret, Issuer: iss, Audience: aud})
}

func base() jwt.MapClaims {
	return jwt.MapClaims{"sub": "alice", "iss": "aegis", "aud": "gateway", "exp": time.Now().Add(time.Hour).Unix(), "iat": time.Now().Unix()}
}

func TestJWTValid(t *testing.T) {
	sub, err := validator("aegis", "gateway").ParseAndValidate(sign(t, jwt.SigningMethodHS256, []byte(secret), base()))
	if err != nil || sub != "alice" {
		t.Fatalf("sub=%q err=%v", sub, err)
	}
}

func TestJWTAudienceArray(t *testing.T) {
	c := base()
	c["aud"] = []string{"other", "gateway"}
	if _, err := validator("aegis", "gateway").ParseAndValidate(sign(t, jwt.SigningMethodHS256, []byte(secret), c)); err != nil {
		t.Fatalf("aud tableau valide refusé: %v", err)
	}
	// Regression: an array audience used to skip the check entirely.
	c["aud"] = []string{"other", "evil"}
	if _, err := validator("aegis", "gateway").ParseAndValidate(sign(t, jwt.SigningMethodHS256, []byte(secret), c)); err == nil {
		t.Fatal("aud tableau sans la bonne audience accepté")
	}
}

func TestJWTRejections(t *testing.T) {
	v := validator("aegis", "gateway")
	cases := map[string]jwt.MapClaims{}
	c := base()
	delete(c, "exp")
	cases["sans exp"] = c
	c = base()
	c["exp"] = time.Now().Add(-time.Hour).Unix()
	cases["expiré"] = c
	c = base()
	c["iss"] = "evil"
	cases["mauvais iss"] = c
	c = base()
	delete(c, "aud")
	cases["aud manquant"] = c
	c = base()
	delete(c, "sub")
	cases["sub manquant"] = c
	c = base()
	c["nbf"] = time.Now().Add(time.Hour).Unix()
	cases["nbf futur"] = c
	for name, cl := range cases {
		if _, err := v.ParseAndValidate(sign(t, jwt.SigningMethodHS256, []byte(secret), cl)); err == nil {
			t.Errorf("%s: accepté", name)
		}
	}
	if _, err := v.ParseAndValidate(sign(t, jwt.SigningMethodHS256, []byte("wrong-secret-wrong-secret-wrong!!"), base())); err == nil {
		t.Error("mauvaise signature acceptée")
	}
	none := sign(t, jwt.SigningMethodNone, jwt.UnsafeAllowNoneSignatureType, base())
	if _, err := v.ParseAndValidate(none); err == nil {
		t.Error("alg=none accepté")
	}
	if _, err := v.ParseAndValidate("not.a.jwt"); err == nil {
		t.Error("jeton malformé accepté")
	}
}

func TestJWTDisabled(t *testing.T) {
	v := NewJWTValidator(config.SecurityJWTConfig{Enabled: false, HMACSecret: secret})
	if _, err := v.ParseAndValidate(sign(t, jwt.SigningMethodHS256, []byte(secret), base())); err == nil {
		t.Fatal("JWT désactivé mais accepté")
	}
}
