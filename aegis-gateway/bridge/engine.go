// AEGIS — zokastech.fr — Apache 2.0 / MIT

package bridge

import (
	"context"
	"errors"
	"sync"
	"time"
)

// ErrNotImplemented is returned when a capability is not yet exposed by aegis-ffi.
var ErrNotImplemented = errors.New("aegis-gateway: non implémenté côté moteur")

// Engine abstracts the AEGIS engine (Rust/FFI or mock).
type Engine interface {
	Analyze(ctx context.Context, text string, analysisConfigJSON string) (json string, err error)
	AnalyzeBatch(ctx context.Context, texts []string) (json string, err error)
	Anonymize(ctx context.Context, text string, configJSON string) (json string, err error)
	Deanonymize(ctx context.Context, bodyJSON string) (json string, err error)
	LastError() string
	Version() string
}

// Pool manages multiple Rust handles (thread-safe reuse).
type Pool struct {
	mu       sync.Mutex
	factory  func() (Engine, error)
	max      int
	idle     []Engine
	all      int
	acquireT time.Duration
}

// NewPool creates a pool; factory must return one Engine per call (e.g. FFI init).
func NewPool(max int, acquireTimeout time.Duration, factory func() (Engine, error)) *Pool {
	if max < 1 {
		max = 4
	}
	return &Pool{
		max:      max,
		factory:  factory,
		idle:     nil,
		acquireT: acquireTimeout,
	}
}

// With runs fn with an engine borrowed from the pool.
func (p *Pool) With(ctx context.Context, fn func(context.Context, Engine) error) error {
	e, err := p.acquire(ctx)
	if err != nil {
		return err
	}
	defer p.release(e)
	return fn(ctx, e)
}

func (p *Pool) acquire(ctx context.Context) (Engine, error) {
	deadline := time.Now().Add(p.acquireT)
	for {
		p.mu.Lock()
		if n := len(p.idle); n > 0 {
			e := p.idle[n-1]
			p.idle = p.idle[:n-1]
			p.mu.Unlock()
			return e, nil
		}
		if p.all < p.max {
			p.all++
			p.mu.Unlock()
			e, ferr := p.factory()
			if ferr != nil {
				p.mu.Lock()
				p.all--
				p.mu.Unlock()
				return nil, ferr
			}
			return e, nil
		}
		p.mu.Unlock()
		if time.Now().After(deadline) {
			return nil, context.DeadlineExceeded
		}
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-time.After(10 * time.Millisecond):
		}
	}
}

func (p *Pool) release(e Engine) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.idle = append(p.idle, e)
}
