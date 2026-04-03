// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"context"

	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/internal/app"
	"github.com/zokastech/aegis/aegis-gateway/internal/infra"
)

// bridgeEnginePool adapts *bridge.Pool to app.EnginePool: the FFI pool invokes a closure
// with bridge.Engine, while the application layer expects app.AnalysisEngine (distinct Go
// parameter types even though Engine satisfies AnalysisEngine).
type bridgeEnginePool struct{ p *bridge.Pool }

func (a bridgeEnginePool) With(ctx context.Context, fn func(context.Context, app.AnalysisEngine) error) error {
	return a.p.With(ctx, func(ctx context.Context, e bridge.Engine) error {
		return fn(ctx, e)
	})
}

// NewAnalyzeUseCase builds the Analyze use case (application layer DDD).
func NewAnalyzeUseCase(s *Services) *app.Analyze {
	if s == nil {
		return nil
	}
	return app.NewAnalyze(
		bridgeEnginePool{p: s.Pool},
		infra.GoBreakerCircuit{B: s.Breaker},
		app.PolicyEngineGate{Eng: s.Policy},
		s.EngineTimeout,
	)
}
