// AEGIS — zokastech.fr — Apache 2.0 / MIT

// Package app is the application layer (use cases). It depends on the domain and on ports
// (interfaces); implementations live in api/, bridge/, policy/, etc.
package app

import (
	"context"
)

// EnginePool provides access to the AEGIS engine (infrastructure adapter: bridge.Pool).
type EnginePool interface {
	With(ctx context.Context, fn func(ctx context.Context, eng AnalysisEngine) error) error
}

// AnalysisEngine is the subset of the engine required for single-document analysis.
type AnalysisEngine interface {
	Analyze(ctx context.Context, text, analysisConfigJSON string) (json string, err error)
}

// Circuit protects engine calls (adapter: gobreaker).
type Circuit interface {
	Run(fn func() error) error
}
