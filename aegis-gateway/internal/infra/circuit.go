// AEGIS — zokastech.fr — Apache 2.0 / MIT

// Package infra provides outbound adapters (implementations of app ports).
package infra

import (
	"github.com/sony/gobreaker"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/internal/app"
)

// GoBreakerCircuit adapte *gobreaker.CircuitBreaker au port app.Circuit.
type GoBreakerCircuit struct {
	B *gobreaker.CircuitBreaker
}

func (g GoBreakerCircuit) Run(fn func() error) error {
	if g.B == nil {
		return fn()
	}
	return bridge.WithCircuitBreaker(g.B, fn)
}

var _ app.Circuit = GoBreakerCircuit{}
