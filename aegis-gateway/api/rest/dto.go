// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import "encoding/json"

// AnalyzeRequest is the POST /v1/analyze body.
type AnalyzeRequest struct {
	Text         string `json:"text"`
	AnalysisJSON string `json:"analysis_config_json,omitempty"`
	Policy       string `json:"policy,omitempty"` // e.g. gdpr-strict (or ?policy=)
}

// AnalyzeResponse wraps raw engine JSON (serde Rust compatibility).
type AnalyzeResponse struct {
	Result json.RawMessage `json:"result"`
}

// AnalyzeBatchRequest is a paginated batch.
type AnalyzeBatchRequest struct {
	Texts    []string `json:"texts"`
	Page     int      `json:"page,omitempty"`
	PageSize int      `json:"page_size,omitempty"`
	Policy   string   `json:"policy,omitempty"`
}

// AnalyzeBatchResponse holds results plus pagination.
type AnalyzeBatchResponse struct {
	Items    []json.RawMessage `json:"items"`
	Total    int               `json:"total"`
	Page     int               `json:"page"`
	PageSize int               `json:"page_size"`
	HasMore  bool              `json:"has_more"`
}

// AnonymizeRequest is analyze plus anonymization.
type AnonymizeRequest struct {
	Text       string `json:"text"`
	ConfigJSON string `json:"config_json,omitempty"`
	Policy     string `json:"policy,omitempty"`
	SubjectID  string `json:"subject_id,omitempty"` // pseudonymization / erasure tracking
}

// AnonymizeResponse is engine JSON (anonymized + analysis).
type AnonymizeResponse struct {
	Result json.RawMessage `json:"result"`
}

// DeanonymizeRequest is admin-only (future FFI).
type DeanonymizeRequest struct {
	AnonymizedResultJSON string `json:"anonymized_result_json"`
	KeyMaterialJSON      string `json:"key_material_json,omitempty"`
}

// DeanonymizeResponse is restored text or an error payload.
type DeanonymizeResponse struct {
	Text    string `json:"text,omitempty"`
	Code    string `json:"code,omitempty"`
	Message string `json:"message,omitempty"`
}

// RecognizersResponse lists L1/L2 recognizers.
type RecognizersResponse struct {
	Recognizers []RecognizerDTO `json:"recognizers"`
}

// RecognizerDTO is one recognizer entry.
type RecognizerDTO struct {
	Name    string `json:"name"`
	Kind    string `json:"kind"`
	Enabled bool   `json:"enabled"`
}

// EntityTypesResponse lists supported entity types.
type EntityTypesResponse struct {
	EntityTypes []string `json:"entity_types"`
}

// UpdateConfigRequest is full or partial YAML.
type UpdateConfigRequest struct {
	YAML string `json:"yaml"`
}

// UpdateConfigResponse carries merge status.
type UpdateConfigResponse struct {
	Status string `json:"status"`
}

// HealthResponse is gateway + engine health.
type HealthResponse struct {
	Status      string `json:"status"`
	RustVersion string `json:"rust_version,omitempty"`
}

// ErrorBody is a uniform JSON error envelope.
type ErrorBody struct {
	Code      string `json:"code"`
	Message   string `json:"message"`
	RequestID string `json:"request_id,omitempty"`
}
