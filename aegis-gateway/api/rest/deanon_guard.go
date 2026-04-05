// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"fmt"
	"net/http"
	"sync"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/zokastech/aegis/aegis-gateway/config"
)

// DeanonGuard applies strict rate limiting, a circuit breaker, and alerts on /deanonymize.
type DeanonGuard struct {
	mu sync.Mutex

	cfg          config.SecurityDeanonymizeConfig
	rateHits     map[string][]time.Time
	circuitHits  []time.Time
	trippedUntil time.Time
}

// NewDeanonGuard builds a guard from config (circuit window in minutes).
func NewDeanonGuard(cfg config.SecurityDeanonymizeConfig) *DeanonGuard {
	if cfg.MaxPerHour <= 0 {
		cfg.MaxPerHour = 10
	}
	if cfg.CircuitMaxSuccess <= 0 {
		cfg.CircuitMaxSuccess = 5
	}
	if cfg.CircuitWindowMinutes <= 0 {
		cfg.CircuitWindowMinutes = 15
	}
	return &DeanonGuard{
		cfg:      cfg,
		rateHits: map[string][]time.Time{},
	}
}

func rateKey(id1, id2 string) string {
	a, b := id1, id2
	if a > b {
		a, b = b, a
	}
	return a + "|" + b
}

// Allow returns nil if the request may proceed, otherwise an Echo HTTP error.
func (g *DeanonGuard) Allow(fingerprintID1, fingerprintID2 string) error {
	g.mu.Lock()
	defer g.mu.Unlock()

	now := time.Now()
	if now.Before(g.trippedUntil) {
		return echo.NewHTTPError(http.StatusServiceUnavailable,
			fmt.Sprintf("circuit deanonymize ouvert jusqu’à %s", g.trippedUntil.UTC().Format(time.RFC3339)))
	}

	win := time.Duration(g.cfg.CircuitWindowMinutes) * time.Minute
	cut := now.Add(-win)
	g.circuitHits = pruneTimes(g.circuitHits, cut)
	if len(g.circuitHits) >= g.cfg.CircuitMaxSuccess {
		g.trippedUntil = now.Add(win)
		return echo.NewHTTPError(http.StatusServiceUnavailable, "seuil de dé-anonymisations atteint (circuit breaker)")
	}

	key := rateKey(fingerprintID1, fingerprintID2)
	hourAgo := now.Add(-time.Hour)
	g.rateHits[key] = pruneTimes(g.rateHits[key], hourAgo)
	if len(g.rateHits[key]) >= g.cfg.MaxPerHour {
		return echo.NewHTTPError(http.StatusTooManyRequests,
			fmt.Sprintf("maximum %d dé-anonymisations/heure pour cette paire d’approbateurs", g.cfg.MaxPerHour))
	}
	g.rateHits[key] = append(g.rateHits[key], now)
	return nil
}

// RecordSuccess records a successful deanonymization (circuit counter).
func (g *DeanonGuard) RecordSuccess() {
	g.mu.Lock()
	defer g.mu.Unlock()
	now := time.Now()
	win := time.Duration(g.cfg.CircuitWindowMinutes) * time.Minute
	g.circuitHits = append(pruneTimes(g.circuitHits, now.Add(-win)), now)
}

func pruneTimes(ts []time.Time, cut time.Time) []time.Time {
	i := 0
	for _, t := range ts {
		if !t.Before(cut) {
			ts[i] = t
			i++
		}
	}
	return ts[:i]
}
