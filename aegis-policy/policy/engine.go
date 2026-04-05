// AEGIS — zokastech.fr — Apache 2.0 / MIT
// Policy engine for **AEGIS** — https://zokastech.fr

package policy

import (
	"fmt"
	"os"
	"strings"

	"gopkg.in/yaml.v3"
)

// DocumentPolicy décrit une politique de conformité (RGPD, etc.).
type DocumentPolicy struct {
	Regulation  string `yaml:"regulation"`
	Name        string `yaml:"name"`
	Description string `yaml:"description"`
	Retention   struct {
		PseudonymizationMappingDays int `yaml:"pseudonymization_mapping_days"`
		AuditLogDays                int `yaml:"audit_log_days"`
	} `yaml:"retention"`
	Entities []EntityRule `yaml:"entities"`
	Defaults struct {
		OnUnknownEntity           string `yaml:"on_unknown_entity"`
		BlockHighRiskCombinations bool   `yaml:"block_high_risk_combinations"`
	} `yaml:"defaults"`
	DataMinimization struct {
		Enabled                    bool `yaml:"enabled"`
		StripRequestBodiesFromLogs bool `yaml:"strip_request_bodies_from_logs"`
	} `yaml:"data_minimization"`
	Rights struct {
		ErasureEndpointEnabled bool `yaml:"erasure_endpoint_enabled"`
		DpiaAutoReport         bool `yaml:"dpia_auto_report"`
	} `yaml:"rights"`
}

// EntityRule règle par type d'entité détecté.
type EntityRule struct {
	Type     string `yaml:"type"`
	Action   string `yaml:"action"`
	Operator string `yaml:"operator"`
}

// Engine charge et interroge les politiques YAML.
type Engine struct {
	active *DocumentPolicy
	rules  map[string]EntityRule
}

// Load lit un fichier politique.
func Load(path string) (*Engine, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var p DocumentPolicy
	if err := yaml.Unmarshal(b, &p); err != nil {
		return nil, err
	}
	rules := make(map[string]EntityRule)
	for _, e := range p.Entities {
		key := strings.ToUpper(strings.TrimSpace(e.Type))
		rules[key] = e
	}
	return &Engine{active: &p, rules: rules}, nil
}

// Name retourne l'identifiant de la politique active.
func (e *Engine) Name() string {
	if e == nil || e.active == nil {
		return ""
	}
	return e.active.Name
}

// OperatorFor retourne l'opérateur à appliquer pour un type d'entité (ex: "EMAIL").
func (e *Engine) OperatorFor(entityType string) (string, bool) {
	if e == nil {
		return "", false
	}
	r, ok := e.rules[strings.ToUpper(entityType)]
	if !ok {
		return "", false
	}
	return r.Operator, true
}

// DataMinimizationEnabled reflète l'art. 25 RGPD (privacy by design / minimisation).
func (e *Engine) DataMinimizationEnabled() bool {
	return e != nil && e.active != nil && e.active.DataMinimization.Enabled
}

// DPIAAuto indique si un rapport DPIA automatique est activé.
func (e *Engine) DPIAAuto() bool {
	return e != nil && e.active != nil && e.active.Rights.DpiaAutoReport
}

// Summarize retourne un résumé pour en-têtes de réponse ou journaux (sans PII).
func (e *Engine) Summarize() string {
	if e == nil || e.active == nil {
		return "no-policy"
	}
	return fmt.Sprintf("%s/%s", e.active.Regulation, e.active.Name)
}
