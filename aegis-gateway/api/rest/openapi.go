// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	_ "embed"
	"net/http"

	"github.com/labstack/echo/v4"
)

//go:embed openapi.yaml
var openAPISpec []byte

// OpenAPIYAML serves the OpenAPI 3.0 spec (complements handler swag annotations).
func OpenAPIYAML(c echo.Context) error {
	return c.Blob(http.StatusOK, "application/yaml", openAPISpec)
}
