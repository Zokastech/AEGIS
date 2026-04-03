// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
)

func TestEmbeddedPoliciesLoad(t *testing.T) {
	e, err := DefaultEngine("", NewMemoryMappingStore())
	if err != nil {
		t.Fatal(err)
	}
	names := e.ListNames()
	if len(names) < 5 {
		t.Fatalf("attendu ≥5 politiques embarquées, obtenu %v", names)
	}
	seen := map[string]bool{}
	for _, n := range names {
		seen[n] = true
	}
	for _, req := range []string{"gdpr-strict", "gdpr-analytics", "hipaa", "pci-dss", "gdpr-article-17", "ccpa", "lgpd"} {
		if !seen[req] {
			t.Errorf("politique manquante: %s", req)
		}
	}
}

func TestAfterAnalyzeRedactsStrict(t *testing.T) {
	e, err := DefaultEngine("", NewMemoryMappingStore())
	if err != nil {
		t.Fatal(err)
	}
	pol, err := e.Policy("gdpr-strict")
	if err != nil {
		t.Fatal(err)
	}
	raw := []byte(`{"entities":[{"entity_type":"EMAIL","text":"a@b.co","start":0,"end":6}],"text_length":10}`)
	out, pr, err := e.AfterAnalyze(context.Background(), pol, raw)
	if err != nil {
		t.Fatal(err)
	}
	var m map[string]interface{}
	if err := json.Unmarshal(out, &m); err != nil {
		t.Fatal(err)
	}
	ent := m["entities"].([]interface{})[0].(map[string]interface{})
	if ent["text"] != "[REDACTED]" {
		t.Fatalf("texte non masqué: %v", ent["text"])
	}
	if pr.RedactedEntityCount < 1 {
		t.Fatalf("compteur redaction: %+v", pr)
	}
}

func TestHighRiskCombinationBlocks(t *testing.T) {
	e, err := DefaultEngine("", NewMemoryMappingStore())
	if err != nil {
		t.Fatal(err)
	}
	pol, err := e.Policy("gdpr-strict")
	if err != nil {
		t.Fatal(err)
	}
	raw := []byte(`{"entities":[
		{"entity_type":"MEDICAL_RECORD","text":"x"},
		{"entity_type":"CREDIT_CARD","text":"4111"}
	]}`)
	_, _, err = e.AfterAnalyze(context.Background(), pol, raw)
	if err == nil || !errors.Is(err, ErrPolicyBlocked) {
		t.Fatalf("attendu ErrPolicyBlocked, obtenu %v", err)
	}
}

func TestMergeAnonymizeConfig(t *testing.T) {
	e, _ := DefaultEngine("", NewMemoryMappingStore())
	pol, _ := e.Policy("pci-dss")
	out, err := MergeAnonymizeConfig(`{"foo":1}`, pol)
	if err != nil {
		t.Fatal(err)
	}
	var m map[string]interface{}
	_ = json.Unmarshal([]byte(out), &m)
	ops, ok := m["entity_operators"].(map[string]interface{})
	if !ok {
		t.Fatalf("entity_operators manquant: %s", out)
	}
	if ops["CREDIT_CARD"] != "fpe" {
		t.Fatalf("opérateur FPE carte: %v", ops["CREDIT_CARD"])
	}
}
