// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLoad_MinimalPolicy(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "p.yaml")
	content := `regulation: GDPR
name: test-policy
description: unit test
retention:
  pseudonymization_mapping_days: 0
  audit_log_days: 7
entities:
  - type: EMAIL
    action: anonymize
    operator: mask
defaults:
  on_unknown_entity: log
  block_high_risk_combinations: false
data_minimization:
  enabled: true
  strip_request_bodies_from_logs: true
rights:
  erasure_endpoint_enabled: false
  dpia_auto_report: true
`
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatal(err)
	}

	eng, err := Load(path)
	if err != nil {
		t.Fatal(err)
	}
	if eng.Name() != "test-policy" {
		t.Fatalf("Name: got %q", eng.Name())
	}
	if !eng.DataMinimizationEnabled() {
		t.Fatal("data minimization should be enabled")
	}
	if !eng.DPIAAuto() {
		t.Fatal("dpia auto should be true")
	}
	if s := eng.Summarize(); s != "GDPR/test-policy" {
		t.Fatalf("Summarize: got %q", s)
	}
	op, ok := eng.OperatorFor("email")
	if !ok || op != "mask" {
		t.Fatalf("OperatorFor EMAIL: ok=%v op=%q", ok, op)
	}
	_, ok = eng.OperatorFor("UNKNOWN")
	if ok {
		t.Fatal("unknown entity should not match")
	}
}

func TestEngine_NilReceivers(t *testing.T) {
	var e *Engine
	if e.Name() != "" {
		t.Fatal("nil engine Name should be empty")
	}
	if _, ok := e.OperatorFor("X"); ok {
		t.Fatal("nil OperatorFor")
	}
	if e.DataMinimizationEnabled() {
		t.Fatal("nil DataMinimizationEnabled")
	}
	if e.DPIAAuto() {
		t.Fatal("nil DPIAAuto")
	}
	if e.Summarize() != "no-policy" {
		t.Fatalf("Summarize: %q", e.Summarize())
	}
}
