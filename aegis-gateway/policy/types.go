// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

// Regulation is the primary legal framework for a policy.
type Regulation string

const (
	RegGDPR   Regulation = "GDPR"
	RegHIPAA  Regulation = "HIPAA"
	RegCCPA   Regulation = "CCPA"
	RegPCIDSS Regulation = "PCI_DSS"
	RegLGPD   Regulation = "LGPD"
)

// ActionOnEntity is the required handling for an entity type.
type ActionOnEntity string

const (
	ActionAnonymize ActionOnEntity = "anonymize"
	ActionLog       ActionOnEntity = "log"
	ActionBlock     ActionOnEntity = "block"
)

// OperatorType is an anonymization operator (passed to the engine).
type OperatorType string

const (
	OpMask         OperatorType = "mask"
	OpRedact       OperatorType = "redact"
	OpReplace      OperatorType = "replace"
	OpHash         OperatorType = "hash"
	OpPseudonymize OperatorType = "pseudonymize"
	OpFPE          OperatorType = "fpe"
	OpTokenize     OperatorType = "tokenize"
)

// AutoAction enumerates automatic policy actions.
type AutoAction string

const (
	ActLog         AutoAction = "log"
	ActLogMinimal  AutoAction = "log_minimal"
	ActAlert       AutoAction = "alert"
	ActBlock       AutoAction = "block"
	ActAuditRetain AutoAction = "audit_retain"
)

// PolicyDocument is the YAML policy root.
type PolicyDocument struct {
	Regulation  Regulation `yaml:"regulation"`
	Name        string     `yaml:"name"`
	Description string     `yaml:"description"`
	Version     string     `yaml:"version"`

	Retention RetentionPolicy `yaml:"retention"`

	Entities []EntityRule `yaml:"entities"`

	Defaults PolicyDefaults `yaml:"defaults"`

	AutomaticActions AutomaticActions `yaml:"automatic_actions"`

	DataMinimization DataMinimizationConfig `yaml:"data_minimization"`

	Rights RightsConfig `yaml:"rights"`
}

// RetentionPolicy holds retention durations.
type RetentionPolicy struct {
	PseudonymizationMappingDays int `yaml:"pseudonymization_mapping_days"`
	AuditLogDays                int `yaml:"audit_log_days"`
	MinimizedDataDays           int `yaml:"minimized_data_days"`
}

// EntityRule is per AEGIS entity type (SCREAMING_SNAKE_CASE).
type EntityRule struct {
	Type           string         `yaml:"type"`
	Action         ActionOnEntity `yaml:"action"`
	Operator       OperatorType   `yaml:"operator"`
	RetentionDays  int            `yaml:"retention_days"`
	Required       bool           `yaml:"required"`
	ResponseRedact bool           `yaml:"response_redact"`
}

// PolicyDefaults applies when an entity type is unknown.
type PolicyDefaults struct {
	OnUnknownEntity           string `yaml:"on_unknown_entity"` // log | anonymize | block
	BlockHighRiskCombinations bool   `yaml:"block_high_risk_combinations"`
	StripResponseEntityText   bool   `yaml:"strip_response_entity_text"`
}

// AutomaticActions run on detection / violation.
type AutomaticActions struct {
	OnDetect    []AutoAction `yaml:"on_detect"`
	OnViolation []AutoAction `yaml:"on_violation"`
}

// DataMinimizationConfig is Art. 25 GDPR-style minimization.
type DataMinimizationConfig struct {
	Enabled                    bool `yaml:"enabled"`
	StripRequestBodiesFromLogs bool `yaml:"strip_request_bodies_from_logs"`
	MaxInputRunes              int  `yaml:"max_input_runes"`
	StripControlCharacters     bool `yaml:"strip_control_chars"`
}

// RightsConfig configures data-subject rights endpoints.
type RightsConfig struct {
	ErasureEndpointEnabled bool `yaml:"erasure_endpoint_enabled"`
	DpiaAutoReport         bool `yaml:"dpia_auto_report"`
}

// Normalize fills in default fields where empty.
func (p *PolicyDocument) Normalize() {
	for i := range p.Entities {
		e := &p.Entities[i]
		if e.Action == "" {
			e.Action = ActionAnonymize
		}
		if p.Defaults.StripResponseEntityText && e.Type != "" {
			e.ResponseRedact = true
		}
	}
	if p.Version == "" {
		p.Version = "1"
	}
}

// EntityRuleByType builds a map of rules by entity type.
func (p *PolicyDocument) EntityRuleByType() map[string]EntityRule {
	m := make(map[string]EntityRule, len(p.Entities))
	for _, e := range p.Entities {
		m[e.Type] = e
	}
	return m
}
