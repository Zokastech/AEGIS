// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"strings"
	"testing"
)

func TestBuildDPIARiskScore(t *testing.T) {
	e, _ := DefaultEngine("", NewMemoryMappingStore())
	pol, err := e.Policy("hipaa")
	if err != nil {
		t.Fatal(err)
	}
	types := []string{"MEDICAL_RECORD", "EMAIL", "SSN"}
	rep := BuildDPIA(pol, types)
	if rep.RiskScore < 40 {
		t.Fatalf("score trop bas pour PHI: %d", rep.RiskScore)
	}
	if rep.RiskBand == "faible" {
		t.Fatal("bande incohérente")
	}
	md, ct := WriteDPIA(rep, DPIAFormatMarkdown)
	if ct == "" || !strings.Contains(md, "HIPAA") {
		t.Fatalf("markdown invalide")
	}
	html, ct2 := WriteDPIA(rep, DPIAFormatHTML)
	if ct2 == "" || !strings.Contains(html, "<html") {
		t.Fatalf("html invalide")
	}
}

func TestDataMinimizationLowersRisk(t *testing.T) {
	pol := &PolicyDocument{
		Name:             "x",
		Regulation:       RegGDPR,
		Description:      "test",
		DataMinimization: DataMinimizationConfig{Enabled: true},
		Defaults:         PolicyDefaults{BlockHighRiskCombinations: false},
	}
	scoreOn, _, _ := computeRiskScore(pol, []string{"EMAIL"})
	pol.DataMinimization.Enabled = false
	scoreOff, _, _ := computeRiskScore(pol, []string{"EMAIL"})
	if scoreOn >= scoreOff {
		t.Fatalf("minimisation devrait réduire le score: %d >= %d", scoreOn, scoreOff)
	}
}
