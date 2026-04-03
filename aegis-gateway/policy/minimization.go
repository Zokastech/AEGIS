// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"strings"
	"time"
	"unicode"
	"unicode/utf8"
)

// MinimizationEvent records one minimization step (Art. 25 GDPR).
type MinimizationEvent struct {
	At       time.Time `json:"at"`
	Policy   string    `json:"policy"`
	Reason   string    `json:"reason"`
	Action   string    `json:"action"`
	Detail   string    `json:"detail,omitempty"`
}

// MinimizationReport aggregates events for audit / DPIA.
type MinimizationReport struct {
	Policy   string              `json:"policy"`
	Events   []MinimizationEvent `json:"events"`
	Actions  []string            `json:"policy_actions,omitempty"`
	InputLen int                 `json:"input_runes_before"`
	OutputLen int                `json:"output_runes_after"`
}

// NewMinimizationReport returns an empty, timestamped report shell.
func NewMinimizationReport(policyName string) *MinimizationReport {
	return &MinimizationReport{Policy: policyName, Events: make([]MinimizationEvent, 0, 4)}
}

// Record appends an event.
func (r *MinimizationReport) Record(reason, action, detail string) {
	r.Events = append(r.Events, MinimizationEvent{
		At:     time.Now().UTC(),
		Policy: r.Policy,
		Reason: reason,
		Action: action,
		Detail: detail,
	})
}

// RecordAction records an automatic policy action.
func (r *MinimizationReport) RecordAction(action, phase, detail string) {
	r.Actions = append(r.Actions, phase+":"+action)
	r.Record(phase, action, detail)
}

// MinimizeAtIngestion applies minimization before engine input.
func MinimizeAtIngestion(text string, pol *PolicyDocument, report *MinimizationReport) (string, error) {
	if pol == nil || !pol.DataMinimization.Enabled {
		if report != nil {
			report.InputLen = utf8.RuneCountInString(text)
			report.OutputLen = report.InputLen
		}
		return text, nil
	}
	cfg := pol.DataMinimization
	if report != nil {
		report.InputLen = utf8.RuneCountInString(text)
	}
	out := text
	if cfg.StripControlCharacters {
		out = stripControlRunes(out)
		if out != text {
			report.Record("strip_control_chars", "removed_controls", "")
		}
	}
	if cfg.MaxInputRunes > 0 {
		if n := utf8.RuneCountInString(out); n > cfg.MaxInputRunes {
			out = truncateRunes(out, cfg.MaxInputRunes)
			report.Record("max_input_runes", "truncated", "")
		}
	}
	out = strings.TrimSpace(out)
	if report != nil {
		report.OutputLen = utf8.RuneCountInString(out)
	}
	return out, nil
}

func stripControlRunes(s string) string {
	var b strings.Builder
	b.Grow(len(s))
	for _, r := range s {
		if r == '\n' || r == '\r' || r == '\t' {
			b.WriteRune(r)
			continue
		}
		if unicode.IsControl(r) {
			continue
		}
		b.WriteRune(r)
	}
	return b.String()
}

func truncateRunes(s string, max int) string {
	if max <= 0 {
		return ""
	}
	var b strings.Builder
	n := 0
	for _, r := range s {
		if n >= max {
			break
		}
		b.WriteRune(r)
		n++
	}
	return b.String()
}

// PurgeMinimizedDataAfter is a documented hook for scheduled jobs (purge minimized data after N days).
// Here: no-op — wire to your datastore (S3, DB); policy carries MinimizedDataDays.
func PurgeMinimizedDataAfter(days int) error {
	if days <= 0 {
		return nil
	}
	// Storage integration: out of gateway scope.
	return nil
}
