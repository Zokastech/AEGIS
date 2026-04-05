// AEGIS — zokastech.fr — Apache 2.0 / MIT

package health

import (
	"context"
	"net/http"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/zokastech/aegis/aegis-gateway/metrics"
)

// Opts configures probes (injected from cmd to avoid an import cycle with rest).
type Opts struct {
	// PingEngine checks the engine (FFI / mock) — e.g. Pool.With + Version().
	PingEngine func(ctx context.Context) error
	// RedisPing is optional; if nil, Redis is treated as ready (no Redis configured).
	RedisPing func(ctx context.Context) error
}

// Attach registers /health/live, /health/ready, /health/startup (no auth, for Kubernetes).
func Attach(e *echo.Echo, o Opts) {
	e.GET("/health/live", liveHandler)
	e.GET("/health/ready", readyHandler(o))
	e.GET("/health/startup", startupHandler(o))
}

func liveHandler(c echo.Context) error {
	return c.JSON(http.StatusOK, map[string]string{"status": "alive"})
}

func readyHandler(o Opts) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx, cancel := context.WithTimeout(c.Request().Context(), 3*time.Second)
		defer cancel()

		if err := o.PingEngine(ctx); err != nil {
			metrics.SetComponentReady("ner", false)
			return c.JSON(http.StatusServiceUnavailable, map[string]string{
				"status": "not_ready",
				"reason": "engine_unavailable",
			})
		}
		metrics.SetComponentReady("ner", true)

		if o.RedisPing != nil {
			if err := o.RedisPing(ctx); err != nil {
				metrics.SetComponentReady("redis", false)
				return c.JSON(http.StatusServiceUnavailable, map[string]string{
					"status": "not_ready",
					"reason": "redis_unavailable",
				})
			}
			metrics.SetComponentReady("redis", true)
		} else {
			metrics.SetComponentReady("redis", true)
		}

		return c.JSON(http.StatusOK, map[string]string{"status": "ready"})
	}
}

func startupHandler(_ Opts) echo.HandlerFunc {
	return func(c echo.Context) error {
		if !StartupComplete() {
			return c.JSON(http.StatusServiceUnavailable, map[string]string{
				"status": "starting",
				"reason": "startup_not_complete",
			})
		}
		return c.JSON(http.StatusOK, map[string]string{"status": "started"})
	}
}

