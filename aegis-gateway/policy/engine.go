// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"strings"
	"sync"
)

// Policy engine sentinel errors.
var (
	ErrPolicyNotFound  = errors.New("policy: politique inconnue")
	ErrPolicyBlocked   = errors.New("policy: traitement bloqué par la politique")
	ErrErasureDisabled = errors.New("policy: effacement désactivé pour cette politique")
)

// PolicyEngine loads and applies YAML policies.
type PolicyEngine struct {
	mu        sync.RWMutex
	policies  map[string]*PolicyDocument
	mappings  MappingStore
	sourceDir string
}

// NewEngine creates an empty engine (call LoadEmbedded / LoadDir next).
func NewEngine(store MappingStore) *PolicyEngine {
	if store == nil {
		store = NewMemoryMappingStore()
	}
	return &PolicyEngine{
		policies: make(map[string]*PolicyDocument),
		mappings: store,
	}
}

// LoadEmbedded loads embedded policies then overlays from disk when policyDir exists.
func LoadEmbedded(embedded fs.FS, embedRoot string, policyDir string, store MappingStore) (*PolicyEngine, error) {
	e := NewEngine(store)
	m, err := LoadFS(embedded, embedRoot)
	if err != nil {
		return nil, err
	}
	e.mu.Lock()
	e.policies = m
	e.mu.Unlock()
	if policyDir != "" {
		if st, err := os.Stat(policyDir); err == nil && st.IsDir() {
			over, err := LoadDir(policyDir)
			if err != nil {
				return nil, err
			}
			e.mu.Lock()
			for k, v := range over {
				e.policies[k] = v
			}
			e.sourceDir = policyDir
			e.mu.Unlock()
		}
	}
	return e, nil
}

// Policy returns a policy by logical name.
func (e *PolicyEngine) Policy(name string) (*PolicyDocument, error) {
	if name == "" {
		return nil, ErrPolicyNotFound
	}
	e.mu.RLock()
	defer e.mu.RUnlock()
	p, ok := e.policies[name]
	if !ok {
		return nil, ErrPolicyNotFound
	}
	return p, nil
}

// ListNames returns names of loaded policies.
func (e *PolicyEngine) ListNames() []string {
	e.mu.RLock()
	defer e.mu.RUnlock()
	out := make([]string, 0, len(e.policies))
	for n := range e.policies {
		out = append(out, n)
	}
	return out
}

// MappingStore exposes the pseudonym mapping store.
func (e *PolicyEngine) MappingStore() MappingStore { return e.mappings }

// BeforeAnalyze applies minimization (Art. 25) and logical on_detect actions.
func (e *PolicyEngine) BeforeAnalyze(_ context.Context, pol *PolicyDocument, text string) (string, *MinimizationReport, error) {
	if pol == nil {
		return text, nil, nil
	}
	report := NewMinimizationReport(pol.Name)
	out, err := MinimizeAtIngestion(text, pol, report)
	if err != nil {
		return "", report, err
	}
	for _, a := range pol.AutomaticActions.OnDetect {
		report.RecordAction(string(a), "on_detect", "pré-analyse")
	}
	return out, report, nil
}

// AfterAnalyze post-processes analyze JSON (PII surface reduction, blocks).
func (e *PolicyEngine) AfterAnalyze(_ context.Context, pol *PolicyDocument, analyzeJSON []byte) ([]byte, *PolicyReport, error) {
	if pol == nil {
		return analyzeJSON, nil, nil
	}
	pr := &PolicyReport{Policy: pol.Name, Regulation: string(pol.Regulation)}
	var root map[string]interface{}
	if err := json.Unmarshal(analyzeJSON, &root); err != nil {
		return analyzeJSON, pr, nil
	}
	entities, ok := root["entities"].([]interface{})
	if !ok {
		return analyzeJSON, pr, nil
	}
	found := make(map[string]bool)
	for _, ent := range entities {
		em, ok := ent.(map[string]interface{})
		if !ok {
			continue
		}
		typ, _ := em["entity_type"].(string)
		if typ != "" {
			found[typ] = true
		}
	}
	if violationHighRisk(found, pol) {
		pr.Blocked = true
		pr.Reason = "combinaison à haut risque (policy.defaults.block_high_risk_combinations)"
		shouldBlock := false
		for _, a := range pol.AutomaticActions.OnViolation {
			if a == ActBlock {
				shouldBlock = true
				break
			}
		}
		if shouldBlock {
			return nil, pr, fmt.Errorf("%w: %s", ErrPolicyBlocked, pr.Reason)
		}
	}
	strip := pol.Defaults.StripResponseEntityText
	idx := pol.EntityRuleByType()
	var outEnts []interface{}
	for _, ent := range entities {
		em, ok := ent.(map[string]interface{})
		if !ok {
			continue
		}
		typ, _ := em["entity_type"].(string)
		rule, hasRule := idx[typ]
		if strip || (hasRule && rule.ResponseRedact) {
			em["text"] = "[REDACTED]"
			pr.RedactedEntityCount++
		}
		outEnts = append(outEnts, em)
	}
	root["entities"] = outEnts
	root["policy_applied"] = pol.Name
	root["policy_regulation"] = string(pol.Regulation)
	b, err := json.Marshal(root)
	if err != nil {
		return analyzeJSON, pr, err
	}
	return b, pr, nil
}

func violationHighRisk(found map[string]bool, pol *PolicyDocument) bool {
	if !pol.Defaults.BlockHighRiskCombinations {
		return false
	}
	cats := 0
	if found["MEDICAL_RECORD"] {
		cats++
	}
	if found["CREDIT_CARD"] || found["IBAN"] || found["BANK_ACCOUNT"] {
		cats++
	}
	if found["SSN"] || found["PASSPORT"] || found["NATIONAL_ID"] || found["DRIVER_LICENSE"] || found["TAX_ID"] {
		cats++
	}
	return cats >= 2
}

// MergeAnonymizeConfig merges policy into engine config JSON.
func MergeAnonymizeConfig(userJSON string, pol *PolicyDocument) (string, error) {
	u := map[string]interface{}{}
	if strings.TrimSpace(userJSON) != "" {
		if err := json.Unmarshal([]byte(userJSON), &u); err != nil {
			return userJSON, err
		}
	}
	ops := map[string]string{}
	ret := map[string]int{}
	for _, e := range pol.Entities {
		if e.Operator != "" {
			ops[e.Type] = string(e.Operator)
		}
		d := e.RetentionDays
		if d == 0 && pol.Retention.PseudonymizationMappingDays > 0 {
			d = pol.Retention.PseudonymizationMappingDays
		}
		if d > 0 {
			ret[e.Type] = d
		}
	}
	u["policy_name"] = pol.Name
	u["policy_regulation"] = string(pol.Regulation)
	u["entity_operators"] = ops
	u["entity_retention_days"] = ret
	u["pseudonymization_mapping_days"] = pol.Retention.PseudonymizationMappingDays
	b, err := json.Marshal(u)
	if err != nil {
		return userJSON, err
	}
	return string(b), nil
}

// PolicyReport carries post-analyze metadata.
type PolicyReport struct {
	Policy              string `json:"policy"`
	Regulation          string `json:"regulation"`
	Blocked             bool   `json:"blocked"`
	Reason              string `json:"reason,omitempty"`
	RedactedEntityCount int    `json:"redacted_entity_count"`
}
