// AEGIS — zokastech.fr — Apache 2.0 / MIT

package auth

import (
	"github.com/labstack/echo/v4"
)

// Principal is an authenticated identity (API key, JWT, mTLS, OIDC).
type Principal struct {
	Subject   string   // e.g. api_key:uuid, jwt:sub, mtls:CN
	Role      string   // viewer | operator | admin | auditor
	KeyID     string   // set for API key auth
	AuthMethod string  // mtls | api_key | jwt | oidc
}

const echoPrincipalKey = "aegis_principal"

// SetPrincipal attaches the principal to the Echo context.
func SetPrincipal(c echo.Context, p *Principal) {
	c.Set(echoPrincipalKey, p)
}

// PrincipalFromEcho reads the principal (nil if missing).
func PrincipalFromEcho(c echo.Context) *Principal {
	v := c.Get(echoPrincipalKey)
	if v == nil {
		return nil
	}
	p, _ := v.(*Principal)
	return p
}
