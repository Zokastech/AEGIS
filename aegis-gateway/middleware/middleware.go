// AEGIS — zokastech.fr — Apache 2.0 / MIT

package middleware

import (
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"golang.org/x/time/rate"
)

const HeaderRequestID = "X-Request-ID"

// RequestID injects a per-request UUID.
func RequestID() echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			rid := c.Request().Header.Get(HeaderRequestID)
			if rid == "" {
				rid = uuid.NewString()
			}
			c.Response().Header().Set(HeaderRequestID, rid)
			c.Set("request_id", rid)
			return next(c)
		}
	}
}

// ZerologLogger logs each request (structured JSON).
func ZerologLogger() echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			start := time.Now()
			err := next(c)
			rid, _ := c.Get("request_id").(string)
			log.Info().
				Str("request_id", rid).
				Str("method", c.Request().Method).
				Str("path", c.Path()).
				Int("status", c.Response().Status).
				Dur("latency", time.Since(start)).
				Str("remote_ip", c.RealIP()).
				Msg("http_request")
			return err
		}
	}
}

// RecoverPanic prevents the process from crashing on panics.
func RecoverPanic() echo.MiddlewareFunc {
	return middleware.Recover()
}

// Gzip compresses responses.
// /metrics is excluded: Prometheus exposition must stay plain text (scrapers expect text format;
// gzip caused "expected a valid start token, got \"\\x1f\"").
func Gzip() echo.MiddlewareFunc {
	return middleware.GzipWithConfig(middleware.GzipConfig{
		Level: 5,
		Skipper: func(c echo.Context) bool {
			return c.Request().URL.Path == "/metrics"
		},
	})
}

// SecurityHeaders sets baseline OWASP response headers (HSTS via AEGIS_HSTS_MAX_AGE).
// If tlsActive is false (cleartext HTTP), HSTS is not sent (Mozilla guidance).
func SecurityHeaders(tlsActive bool) echo.MiddlewareFunc {
	hstsMax := int64(31536000)
	if v := os.Getenv("AEGIS_HSTS_MAX_AGE"); v != "" {
		if n, err := strconv.ParseInt(v, 10, 64); err == nil {
			hstsMax = n
		}
	}
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if tlsActive {
				c.Response().Header().Set("Strict-Transport-Security", "max-age="+strconv.FormatInt(hstsMax, 10)+"; includeSubDomains")
			}
			c.Response().Header().Set("X-Content-Type-Options", "nosniff")
			c.Response().Header().Set("X-Frame-Options", "DENY")
			c.Response().Header().Set("Cache-Control", "no-store")
			c.Response().Header().Set("Content-Security-Policy", "frame-ancestors 'none'")
			return next(c)
		}
	}
}

// CORS configurable (legacy: empty origins → *).
func CORS(allowOrigins []string) echo.MiddlewareFunc {
	orig := allowOrigins
	if len(orig) == 0 {
		orig = []string{"*"}
	}
	return middleware.CORSWithConfig(middleware.CORSConfig{
		AllowOrigins:     orig,
		AllowMethods:     []string{echo.GET, echo.POST, echo.PUT, echo.OPTIONS},
		AllowHeaders:     []string{echo.HeaderOrigin, echo.HeaderContentType, echo.HeaderAccept, HeaderRequestID, "X-API-Key", "Authorization", "X-Aegis-Role", "X-Aegis-Approval-1", "X-Aegis-Approval-2"},
		AllowCredentials: true,
	})
}

// CORSFromSecurity: in production with no configured origins, skip CORS middleware (no ACAO headers).
func CORSFromSecurity(production bool, allowOrigins []string) echo.MiddlewareFunc {
	if production && len(allowOrigins) == 0 {
		return func(next echo.HandlerFunc) echo.HandlerFunc {
			return func(c echo.Context) error {
				return next(c)
			}
		}
	}
	orig := allowOrigins
	if len(orig) == 0 {
		orig = []string{"*"}
	}
	return middleware.CORSWithConfig(middleware.CORSConfig{
		AllowOrigins:     orig,
		AllowMethods:     []string{echo.GET, echo.POST, echo.PUT, echo.OPTIONS},
		AllowHeaders:     []string{echo.HeaderOrigin, echo.HeaderContentType, echo.HeaderAccept, HeaderRequestID, "X-API-Key", "Authorization", "X-Aegis-Role", "X-Aegis-Approval-1", "X-Aegis-Approval-2"},
		AllowCredentials: true,
	})
}

// RateLimitIP rate-limits by client IP (best-effort).
func RateLimitIP(rps float64, burst int) echo.MiddlewareFunc {
	if rps <= 0 {
		rps = 50
	}
	if burst <= 0 {
		burst = int(rps * 2)
	}
	limiters := map[string]*rate.Limiter{}
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			ip := c.RealIP()
			lim, ok := limiters[ip]
			if !ok {
				lim = rate.NewLimiter(rate.Limit(rps), burst)
				limiters[ip] = lim
			}
			if !lim.Allow() {
				return echo.NewHTTPError(429, "rate limit exceeded")
			}
			return next(c)
		}
	}
}

// RateLimitAPIKey rate-limits by API key value when the header is present.
func RateLimitAPIKey(header string, rps float64, burst int) echo.MiddlewareFunc {
	if rps <= 0 {
		rps = 100
	}
	if burst <= 0 {
		burst = int(rps * 2)
	}
	if header == "" {
		header = "X-API-Key"
	}
	limiters := map[string]*rate.Limiter{}
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			key := strings.TrimSpace(c.Request().Header.Get(header))
			if key == "" {
				return next(c)
			}
			lim, ok := limiters[key]
			if !ok {
				lim = rate.NewLimiter(rate.Limit(rps), burst)
				limiters[key] = lim
			}
			if !lim.Allow() {
				return echo.NewHTTPError(429, "api key rate limit exceeded")
			}
			return next(c)
		}
	}
}

// SetGlobalLogLevel configures zerolog (e.g. from env).
func SetGlobalLogLevel(level string) {
	l, err := zerolog.ParseLevel(strings.ToLower(level))
	if err != nil {
		l = zerolog.InfoLevel
	}
	zerolog.SetGlobalLevel(l)
}
