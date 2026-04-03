// AEGIS — zokastech.fr — Apache 2.0 / MIT

package metrics

import (
	"strconv"
	"time"
)

// ObserveAnalyze records counters, histogram, and entity counts for successful or failed analyze calls.
func ObserveAnalyze(endpoint, method string, status int, analysisJSON string, started time.Time, resultJSON []byte) {
	lang, pl := LanguageAndPipeline(analysisJSON)
	st := strconv.Itoa(status)
	AnalyzeRequestsTotal.WithLabelValues(endpoint, method, st, lang).Inc()
	if status != 200 {
		return
	}
	d := time.Since(started).Seconds()
	AnalyzeDuration.WithLabelValues(pl, endpoint).Observe(d)
	if pl == "3" {
		NERInferenceDuration.Observe(d)
	}
	for et, n := range CountEntitiesByType(resultJSON, pl) {
		if n <= 0 {
			continue
		}
		EntitiesDetectedTotal.WithLabelValues(et, pl).Add(float64(n))
	}
}

// ObserveAnalyzeBatch records a batch request, batch size, and aggregates entity counts.
func ObserveAnalyzeBatch(endpoint, method string, status int, batchLen int, analysisJSON string, started time.Time, items [][]byte) {
	lang, pl := LanguageAndPipeline(analysisJSON)
	st := strconv.Itoa(status)
	AnalyzeRequestsTotal.WithLabelValues(endpoint, method, st, lang).Inc()
	if batchLen > 0 {
		BatchSize.Observe(float64(batchLen))
	}
	if status != 200 {
		return
	}
	d := time.Since(started).Seconds()
	AnalyzeDuration.WithLabelValues(pl, endpoint).Observe(d)
	if pl == "3" {
		NERInferenceDuration.Observe(d)
	}
	for _, raw := range items {
		for et, n := range CountEntitiesByType(raw, pl) {
			if n <= 0 {
				continue
			}
			EntitiesDetectedTotal.WithLabelValues(et, pl).Add(float64(n))
		}
	}
}

// ObserveAnonymizeRequest counts anonymize requests and operators on success.
func ObserveAnonymizeRequest(endpoint, method string, status int, resultJSON []byte) {
	AnalyzeRequestsTotal.WithLabelValues(endpoint, method, strconv.Itoa(status), "n_a").Inc()
	if status == 200 {
		ObserveAnonymizeOperators(resultJSON)
	}
}
