// AEGIS — zokastech.fr — Apache 2.0 / MIT

package metrics

import (
	"strings"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

const namespace = "aegis"

var analyzeBuckets = []float64{.005, .01, .025, .05, .1, .25, .5, 1, 2.5, 5, 10, 30, 60}

// AEGIS Prometheus metrics (names aligned with observability contract).
var (
	AnalyzeRequestsTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: namespace,
			Name:      "analyze_requests_total",
			Help:      "Total analyze-related HTTP/gRPC requests by endpoint, method, status, language.",
		},
		[]string{"endpoint", "method", "status", "language"},
	)
	AnalyzeDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Namespace: namespace,
			Name:      "analyze_duration_seconds",
			Help:      "Analyze request duration in seconds by pipeline level and logical endpoint.",
			Buckets:   analyzeBuckets,
		},
		[]string{"pipeline_level", "endpoint"},
	)
	EntitiesDetectedTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: namespace,
			Name:      "entities_detected_total",
			Help:      "Entities detected by type and detection pipeline level.",
		},
		[]string{"entity_type", "detection_level"},
	)
	AnonymizeOpsTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: namespace,
			Name:      "anonymize_operations_total",
			Help:      "Anonymization operator applications by operator type.",
		},
		[]string{"operator_type"},
	)
	FalsePositiveReports = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: namespace,
			Name:      "false_positive_reports_total",
			Help:      "User false-positive feedback submissions.",
		},
		[]string{"channel"},
	)
	NERInferenceDuration = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Namespace: namespace,
			Name:      "ner_inference_duration_seconds",
			Help:      "NER model inference latency (subset of analyze when pipeline level is 3).",
			Buckets:   analyzeBuckets,
		},
	)
	BatchSize = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Namespace: namespace,
			Name:      "batch_size",
			Help:      "Number of texts per analyze batch request.",
			Buckets:   prometheus.LinearBuckets(1, 5, 20),
		},
	)
	ActiveConnections = promauto.NewGauge(
		prometheus.GaugeOpts{
			Namespace: namespace,
			Name:      "active_connections",
			Help:      "HTTP requests currently being processed (in-flight).",
		},
	)
	DeanonymizeOpsTotal = promauto.NewCounter(
		prometheus.CounterOpts{
			Namespace: namespace,
			Name:      "deanonymize_operations_total",
			Help:      "Successful deanonymize operations (sensitive — alert on increase).",
		},
	)
	ComponentReady = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: namespace,
			Name:      "component_ready",
			Help:      "1 if component passed last readiness check, 0 otherwise.",
		},
		[]string{"component"},
	)
)

// SanitizeLabel normalizes a value for Prometheus labels (avoids runaway cardinality).
func SanitizeLabel(s string) string {
	s = strings.TrimSpace(s)
	if s == "" {
		return "unknown"
	}
	b := make([]rune, 0, len(s))
	for _, r := range strings.ToLower(s) {
		switch {
		case r >= 'a' && r <= 'z', r >= '0' && r <= '9':
			b = append(b, r)
		default:
			b = append(b, '_')
		}
	}
	out := string(b)
	if out == "" {
		return "unknown"
	}
	return out
}

// SetComponentReady updates the component_ready{component} gauge.
func SetComponentReady(component string, ok bool) {
	v := 0.0
	if ok {
		v = 1
	}
	ComponentReady.WithLabelValues(SanitizeLabel(component)).Set(v)
}

// IncFalsePositive increments the reported false-positive counter.
func IncFalsePositive(channel string) {
	FalsePositiveReports.WithLabelValues(SanitizeLabel(channel)).Inc()
}

// IncDeanonymize increments after a successful deanonymization.
func IncDeanonymize() {
	DeanonymizeOpsTotal.Inc()
}
