// AEGIS — zokastech.fr — Apache 2.0 / MIT

package middleware

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"net"
	"net/http"
	"os"
	"strconv"
	"strings"
	"sync"
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
			if !validRequestID(rid) {
				rid = uuid.NewString()
			}
			c.Response().Header().Set(HeaderRequestID, rid)
			c.Set("request_id", rid)
			return next(c)
		}
	}
}

// validRequestID accepts client-supplied IDs only if short and made of safe characters
// (prevents log/audit injection and header abuse).
func validRequestID(s string) bool {
	if s == "" || len(s) > 128 {
		return false
	}
	for _, r := range s {
		switch {
		case r >= 'a' && r <= 'z', r >= 'A' && r <= 'Z', r >= '0' && r <= '9', r == '-', r == '_', r == '.', r == ':':
		default:
			return false
		}
	}
	return true
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
			c.Response().Header().Set("Content-Security-Policy", "default-src 'none'; style-src 'unsafe-inline'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'")
			c.Response().Header().Set("Referrer-Policy", "no-referrer")
			c.Response().Header().Set("Cross-Origin-Resource-Policy", "same-origin")
			return next(c)
		}
	}
}

// CORS configurable (legacy: empty origins → *).
func CORS(allowOrigins []string) echo.MiddlewareFunc {
	return corsWithOrigins(allowOrigins)
}

// corsWithOrigins never combines a wildcard origin with credentials.
func corsWithOrigins(allowOrigins []string) echo.MiddlewareFunc {
	orig := allowOrigins
	wildcard := len(orig) == 0
	for _, o := range orig {
		if strings.TrimSpace(o) == "*" {
			wildcard = true
		}
	}
	if len(orig) == 0 {
		orig = []string{"*"}
	}
	return middleware.CORSWithConfig(middleware.CORSConfig{
		AllowOrigins:     orig,
		AllowMethods:     []string{echo.GET, echo.POST, echo.PUT, echo.DELETE, echo.OPTIONS},
		AllowHeaders:     []string{echo.HeaderOrigin, echo.HeaderContentType, echo.HeaderAccept, HeaderRequestID, "X-API-Key", "Authorization", "X-Aegis-Role", "X-Aegis-Approval-1", "X-Aegis-Approval-2"},
		AllowCredentials: !wildcard,
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
	return corsWithOrigins(allowOrigins)
}

// limiterSet is a concurrency-safe set of token buckets with idle eviction and a hard cap,
// so an attacker rotating IPs/keys cannot exhaust memory.
type limiterSet struct {
	mu      sync.Mutex
	rps     rate.Limit
	burst   int
	maxSize int
	idle    time.Duration
	items   map[string]*limiterEntry
	lastGC  time.Time
	now     func() time.Time
}

type limiterEntry struct {
	lim      *rate.Limiter
	lastSeen time.Time
}

const (
	defaultLimiterMaxEntries = 100_000
	defaultLimiterIdle       = 10 * time.Minute
)

func newLimiterSet(rps float64, burst int) *limiterSet {
	return &limiterSet{
		rps:     rate.Limit(rps),
		burst:   burst,
		maxSize: defaultLimiterMaxEntries,
		idle:    defaultLimiterIdle,
		items:   make(map[string]*limiterEntry),
		now:     time.Now,
	}
}

// allow reports whether one more event is permitted for key.
func (s *limiterSet) allow(key string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	now := s.now()
	if now.Sub(s.lastGC) > time.Minute || len(s.items) >= s.maxSize {
		s.gcLocked(now)
	}
	e, ok := s.items[key]
	if !ok {
		if len(s.items) >= s.maxSize {
			// Table full of active clients: fail closed for unknown keys instead of growing.
			return false
		}
		e = &limiterEntry{lim: rate.NewLimiter(s.rps, s.burst)}
		s.items[key] = e
	}
	e.lastSeen = now
	return e.lim.AllowN(now, 1)
}

func (s *limiterSet) gcLocked(now time.Time) {
	for k, e := range s.items {
		if now.Sub(e.lastSeen) > s.idle {
			delete(s.items, k)
		}
	}
	s.lastGC = now
}

func (s *limiterSet) size() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.items)
}

// RateLimitIP rate-limits by client IP (Echo RealIP: configure e.IPExtractor via TrustedProxyIPExtractor).
func RateLimitIP(rps float64, burst int) echo.MiddlewareFunc {
	if rps <= 0 {
		rps = 50
	}
	if burst <= 0 {
		burst = int(rps * 2)
	}
	set := newLimiterSet(rps, burst)
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if !set.allow(c.RealIP()) {
				return echo.NewHTTPError(http.StatusTooManyRequests, "rate limit exceeded")
			}
			return next(c)
		}
	}
}

// RateLimitAPIKey rate-limits by API key when the header is present.
// Buckets are keyed by SHA-256 of the key so plaintext secrets are never retained in memory maps.
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
	set := newLimiterSet(rps, burst)
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			key := strings.TrimSpace(c.Request().Header.Get(header))
			if key == "" {
				return next(c)
			}
			sum := sha256.Sum256([]byte(key))
			if !set.allow(hex.EncodeToString(sum[:16])) {
				return echo.NewHTTPError(http.StatusTooManyRequests, "api key rate limit exceeded")
			}
			return next(c)
		}
	}
}

// TrustedProxyIPExtractor returns an Echo IP extractor.
// With no trusted proxies, the direct TCP peer is used and X-Forwarded-For / X-Real-IP are ignored
// (they are client-controlled and would allow rate-limit bypass and audit-log spoofing).
// With trusted CIDRs/IPs, X-Forwarded-For is honoured only when the peer is one of them.
func TrustedProxyIPExtractor(trusted []string) (echo.IPExtractor, error) {
	if len(trusted) == 0 {
		return echo.ExtractIPDirect(), nil
	}
	opts := []echo.TrustOption{
		echo.TrustLoopback(false),
		echo.TrustLinkLocal(false),
		echo.TrustPrivateNet(false),
	}
	for _, t := range trusted {
		t = strings.TrimSpace(t)
		if t == "" {
			continue
		}
		if !strings.Contains(t, "/") {
			if ip := net.ParseIP(t); ip != nil && ip.To4() != nil {
				t += "/32"
			} else {
				t += "/128"
			}
		}
		_, n, err := net.ParseCIDR(t)
		if err != nil {
			return nil, fmt.Errorf("trusted_proxies: %q invalide: %w", t, err)
		}
		opts = append(opts, echo.TrustIPRange(n))
	}
	return echo.ExtractIPFromXFFHeader(opts...), nil
}

// BodyLimit caps request bodies (bytes). 0 or negative = 10 MiB.
func BodyLimit(maxBytes int64) echo.MiddlewareFunc {
	if maxBytes <= 0 {
		maxBytes = 10 << 20
	}
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			req := c.Request()
			if req.ContentLength > maxBytes {
				return echo.NewHTTPError(http.StatusRequestEntityTooLarge, "corps de requête trop volumineux")
			}
			req.Body = http.MaxBytesReader(c.Response(), req.Body, maxBytes)
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
