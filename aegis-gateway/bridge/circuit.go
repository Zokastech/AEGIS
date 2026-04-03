// AEGIS — zokastech.fr — Apache 2.0 / MIT

package bridge

import (
	"time"

	"github.com/sony/gobreaker"
)

// WithCircuitBreaker wraps an engine call with gobreaker (timeouts via context).
func WithCircuitBreaker(cb *gobreaker.CircuitBreaker, fn func() error) error {
	if cb == nil {
		return fn()
	}
	_, err := cb.Execute(func() (interface{}, error) {
		return nil, fn()
	})
	return err
}

// NewDefaultBreaker creates a reasonable circuit breaker for FFI calls.
func NewDefaultBreaker() *gobreaker.CircuitBreaker {
	st := gobreaker.Settings{
		Name:        "aegis-ffi",
		MaxRequests: 3,
		Interval:    30 * time.Second,
		Timeout:     12 * time.Second,
		ReadyToTrip: func(c gobreaker.Counts) bool {
			return c.ConsecutiveFailures >= 5
		},
	}
	return gobreaker.NewCircuitBreaker(st)
}
