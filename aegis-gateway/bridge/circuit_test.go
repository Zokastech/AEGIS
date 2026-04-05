// AEGIS — zokastech.fr — Apache 2.0 / MIT

package bridge

import (
	"errors"
	"testing"
)

func TestWithCircuitBreaker_NilPassesThrough(t *testing.T) {
	calls := 0
	err := WithCircuitBreaker(nil, func() error {
		calls++
		return nil
	})
	if err != nil || calls != 1 {
		t.Fatalf("err=%v calls=%d", err, calls)
	}
}

func TestWithCircuitBreaker_PropagatesError(t *testing.T) {
	e := errors.New("boom")
	err := WithCircuitBreaker(nil, func() error { return e })
	if !errors.Is(err, e) {
		t.Fatalf("got %v", err)
	}
}

func TestNewDefaultBreaker_NotNil(t *testing.T) {
	cb := NewDefaultBreaker()
	if cb == nil {
		t.Fatal("expected circuit breaker")
	}
}
