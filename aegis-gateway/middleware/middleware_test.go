// AEGIS — zokastech.fr — Apache 2.0 / MIT

package middleware

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/labstack/echo/v4"
)

func okHandler(c echo.Context) error { return c.String(http.StatusOK, c.RealIP()) }

func serve(e *echo.Echo, r *http.Request) *httptest.ResponseRecorder {
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, r)
	return rec
}

// Regression: limiter maps were written concurrently without a lock (fatal "concurrent map writes").
func TestRateLimitConcurrentSafe(t *testing.T) {
	e := echo.New()
	e.Use(RateLimitIP(1e6, 1e6), RateLimitAPIKey("X-API-Key", 1e6, 1e6))
	e.GET("/", okHandler)
	var wg sync.WaitGroup
	for i := 0; i < 64; i++ {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			for j := 0; j < 50; j++ {
				r := httptest.NewRequest(http.MethodGet, "/", nil)
				r.RemoteAddr = "10.0." + string(rune('0'+i%10)) + ".1:1234"
				r.Header.Set("X-API-Key", strings.Repeat("k", i%7+1))
				_ = serve(e, r)
			}
		}(i)
	}
	wg.Wait()
}

func TestRateLimitIPBlocks(t *testing.T) {
	e := echo.New()
	e.IPExtractor = echo.ExtractIPDirect()
	e.Use(RateLimitIP(1, 2))
	e.GET("/", okHandler)
	codes := []int{}
	for i := 0; i < 3; i++ {
		r := httptest.NewRequest(http.MethodGet, "/", nil)
		r.RemoteAddr = "192.0.2.10:5555"
		codes = append(codes, serve(e, r).Code)
	}
	if codes[2] != http.StatusTooManyRequests {
		t.Fatalf("codes %v", codes)
	}
}

// Regression: X-Forwarded-For was trusted by default, letting clients rotate fake IPs to evade limits.
func TestSpoofedXFFIgnoredWithoutTrustedProxies(t *testing.T) {
	ipx, err := TrustedProxyIPExtractor(nil)
	if err != nil {
		t.Fatal(err)
	}
	e := echo.New()
	e.IPExtractor = ipx
	e.Use(RateLimitIP(1, 1))
	e.GET("/", okHandler)
	var last int
	for i := 0; i < 3; i++ {
		r := httptest.NewRequest(http.MethodGet, "/", nil)
		r.RemoteAddr = "198.51.100.7:4444"
		r.Header.Set("X-Forwarded-For", "203.0.113."+string(rune('1'+i)))
		last = serve(e, r).Code
	}
	if last != http.StatusTooManyRequests {
		t.Fatalf("XFF usurpé contourne la limite: %d", last)
	}
}

func TestTrustedProxyHonoursXFF(t *testing.T) {
	ipx, err := TrustedProxyIPExtractor([]string{"10.1.0.0/16", "192.0.2.1"})
	if err != nil {
		t.Fatal(err)
	}
	e := echo.New()
	e.IPExtractor = ipx
	e.GET("/", okHandler)
	r := httptest.NewRequest(http.MethodGet, "/", nil)
	r.RemoteAddr = "10.1.2.3:80"
	r.Header.Set("X-Forwarded-For", "203.0.113.9")
	if got := serve(e, r).Body.String(); got != "203.0.113.9" {
		t.Fatalf("IP client via proxy de confiance: %q", got)
	}
	r = httptest.NewRequest(http.MethodGet, "/", nil)
	r.RemoteAddr = "172.16.0.5:80"
	r.Header.Set("X-Forwarded-For", "203.0.113.9")
	if got := serve(e, r).Body.String(); got == "203.0.113.9" {
		t.Fatal("XFF honoré depuis un pair non fiable")
	}
	if _, err := TrustedProxyIPExtractor([]string{"not-an-ip"}); err == nil {
		t.Fatal("CIDR invalide accepté")
	}
}

func TestLimiterSetEvictionAndCap(t *testing.T) {
	s := newLimiterSet(10, 10)
	now := time.Unix(1_700_000_000, 0)
	s.now = func() time.Time { return now }
	s.maxSize = 3
	for _, k := range []string{"a", "b", "c"} {
		if !s.allow(k) {
			t.Fatal("refus inattendu")
		}
	}
	if s.allow("d") {
		t.Fatal("table pleine : une nouvelle clé doit être refusée (pas de croissance)")
	}
	now = now.Add(11 * time.Minute)
	if !s.allow("d") {
		t.Fatal("entrées inactives non évincées")
	}
	if s.size() != 1 {
		t.Fatalf("taille après éviction: %d", s.size())
	}
}

func TestAPIKeyLimiterPerKey(t *testing.T) {
	e := echo.New()
	mw := RateLimitAPIKey("X-API-Key", 1, 1)
	e.Use(mw)
	e.GET("/", okHandler)
	r := httptest.NewRequest(http.MethodGet, "/", nil)
	r.Header.Set("X-API-Key", "super-secret")
	if serve(e, r).Code != http.StatusOK {
		t.Fatal()
	}
	r = httptest.NewRequest(http.MethodGet, "/", nil)
	r.Header.Set("X-API-Key", "super-secret")
	if serve(e, r).Code != http.StatusTooManyRequests {
		t.Fatal("limite par clé non appliquée")
	}
}

func TestBodyLimit(t *testing.T) {
	e := echo.New()
	e.Use(BodyLimit(16))
	e.POST("/", func(c echo.Context) error {
		var m map[string]interface{}
		if err := c.Bind(&m); err != nil {
			return err
		}
		return c.NoContent(http.StatusOK)
	})
	r := httptest.NewRequest(http.MethodPost, "/", strings.NewReader(`{"a":"`+strings.Repeat("x", 64)+`"}`))
	r.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	if code := serve(e, r).Code; code != http.StatusRequestEntityTooLarge {
		t.Fatalf("want 413 got %d", code)
	}
	r = httptest.NewRequest(http.MethodPost, "/", strings.NewReader(`{"a":1}`))
	r.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
	if code := serve(e, r).Code; code != http.StatusOK {
		t.Fatalf("petit corps refusé: %d", code)
	}
}

func TestRequestIDSanitized(t *testing.T) {
	e := echo.New()
	e.Use(RequestID())
	e.GET("/", func(c echo.Context) error { return c.String(http.StatusOK, c.Get("request_id").(string)) })
	for in, keep := range map[string]bool{
		"abc-123_X.y:z":            true,
		"evil\"}\n{\"admin\":true": false,
		strings.Repeat("a", 200):   false,
		"":                         false,
	} {
		r := httptest.NewRequest(http.MethodGet, "/", nil)
		if in != "" {
			r.Header.Set(HeaderRequestID, in)
		}
		got := serve(e, r).Body.String()
		if keep && got != in {
			t.Errorf("ID valide remplacé: %q", in)
		}
		if !keep && got == in {
			t.Errorf("ID dangereux conservé: %q", in)
		}
	}
}

func TestSecurityHeaders(t *testing.T) {
	e := echo.New()
	e.Use(SecurityHeaders(true))
	e.GET("/", okHandler)
	h := serve(e, httptest.NewRequest(http.MethodGet, "/", nil)).Header()
	for _, k := range []string{"Strict-Transport-Security", "X-Content-Type-Options", "X-Frame-Options", "Content-Security-Policy", "Referrer-Policy", "Cache-Control"} {
		if h.Get(k) == "" {
			t.Errorf("en-tête %s absent", k)
		}
	}
	e2 := echo.New()
	e2.Use(SecurityHeaders(false))
	e2.GET("/", okHandler)
	if serve(e2, httptest.NewRequest(http.MethodGet, "/", nil)).Header().Get("Strict-Transport-Security") != "" {
		t.Error("HSTS envoyé en HTTP clair")
	}
}

func TestCORSWildcardNeverWithCredentials(t *testing.T) {
	e := echo.New()
	e.Use(CORSFromSecurity(false, nil))
	e.GET("/", okHandler)
	r := httptest.NewRequest(http.MethodGet, "/", nil)
	r.Header.Set("Origin", "https://evil.example")
	h := serve(e, r).Header()
	if h.Get("Access-Control-Allow-Credentials") == "true" {
		t.Fatal("wildcard + credentials")
	}
	if got := h.Get("Access-Control-Allow-Origin"); got == "https://evil.example" {
		t.Fatal("origine reflétée")
	}

	e2 := echo.New()
	e2.Use(CORSFromSecurity(true, nil))
	e2.GET("/", okHandler)
	if serve(e2, r).Header().Get("Access-Control-Allow-Origin") != "" {
		t.Fatal("production sans origines : aucun en-tête CORS attendu")
	}

	e3 := echo.New()
	e3.Use(CORSFromSecurity(true, []string{"https://app.example"}))
	e3.GET("/", okHandler)
	r3 := httptest.NewRequest(http.MethodGet, "/", nil)
	r3.Header.Set("Origin", "https://app.example")
	h3 := serve(e3, r3).Header()
	if h3.Get("Access-Control-Allow-Origin") != "https://app.example" || h3.Get("Access-Control-Allow-Credentials") != "true" {
		t.Fatalf("liste blanche: %v", h3)
	}
}
