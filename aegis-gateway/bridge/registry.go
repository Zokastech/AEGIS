// AEGIS — zokastech.fr — Apache 2.0 / MIT

package bridge

// RecognizerInfo describes a recognizer exposed by the gateway (static list until FFI exposes one).
type RecognizerInfo struct {
	Name    string `json:"name"`
	Kind    string `json:"kind"`
	Enabled bool   `json:"enabled"`
}

// DefaultRecognizers lists typical AEGIS regex recognizers (aligned with product docs).
func DefaultRecognizers() []RecognizerInfo {
	return []RecognizerInfo{
		{Name: "email", Kind: "regex", Enabled: true},
		{Name: "phone_eu", Kind: "regex", Enabled: true},
		{Name: "iban", Kind: "regex", Enabled: true},
		{Name: "credit_card", Kind: "regex", Enabled: true},
		{Name: "ip_address", Kind: "regex", Enabled: true},
		{Name: "national_id_fr", Kind: "regex", Enabled: true},
		{Name: "pipeline_l2_context", Kind: "context", Enabled: true},
		{Name: "ner_onnx", Kind: "ml", Enabled: false},
	}
}

// SupportedEntityTypes are AEGIS config keys / serde SCREAMING_SNAKE_CASE.
func SupportedEntityTypes() []string {
	return []string{
		"PERSON", "EMAIL", "PHONE", "CREDIT_CARD", "IBAN", "SSN", "PASSPORT",
		"DRIVER_LICENSE", "IP_ADDRESS", "URL", "DATE", "ADDRESS", "ORGANIZATION",
		"LOCATION", "MEDICAL_RECORD", "NATIONAL_ID", "TAX_ID", "BANK_ACCOUNT",
		"CRYPTO_WALLET", "VEHICLE_PLATE",
	}
}
