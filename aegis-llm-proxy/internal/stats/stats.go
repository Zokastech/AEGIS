// AEGIS — zokastech.fr — Apache 2.0 / MIT

package stats

import (
	"encoding/json"
	"net/http"
	"sync/atomic"
)

// Registry compteurs atomiques pour le dashboard AEGIS.
type Registry struct {
	RequestsTotal    atomic.Uint64
	PIIDetected      atomic.Uint64
	BlockedTotal     atomic.Uint64
	AnonymizedTotal  atomic.Uint64
	StreamPassthrough atomic.Uint64
	ErrorsTotal      atomic.Uint64
}

// Snapshot lecture cohérente (approximative).
func (r *Registry) Snapshot() map[string]uint64 {
	return map[string]uint64{
		"requests_total":     r.RequestsTotal.Load(),
		"pii_detected_total": r.PIIDetected.Load(),
		"blocked_total":      r.BlockedTotal.Load(),
		"anonymized_total":   r.AnonymizedTotal.Load(),
		"stream_passthrough_total": r.StreamPassthrough.Load(),
		"errors_total":       r.ErrorsTotal.Load(),
	}
}

// WriteJSON écrit application/json.
func (r *Registry) WriteJSON(w http.ResponseWriter) {
	w.Header().Set("Content-Type", "application/json")
	enc := json.NewEncoder(w)
	enc.SetIndent("", "  ")
	_ = enc.Encode(r.Snapshot())
}
