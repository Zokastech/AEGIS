// AEGIS — zokastech.fr — Apache 2.0 / MIT

package bridge

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
)

// MockEngine simulates engine JSON for integration tests without CGO.
type MockEngine struct{}

func NewMockEngine() *MockEngine { return &MockEngine{} }

type analysisResult struct {
	Entities           []map[string]interface{} `json:"entities"`
	ProcessingTimeMs   uint64                 `json:"processing_time_ms"`
	LanguageDetected   *string                `json:"language_detected,omitempty"`
	TextLength         int                    `json:"text_length"`
}

func (m *MockEngine) Analyze(_ context.Context, text string, _ string) (string, error) {
	start, end := 0, 0
	email := "mock@example.com"
	if i := strings.Index(text, "@"); i >= 0 {
		j := i + 1
		for j < len(text) && text[j] != ' ' && text[j] != '\t' && text[j] != '\n' {
			j++
		}
		start = i
		end = j
		email = text[start:end]
	}
	r := analysisResult{
		Entities: []map[string]interface{}{
			{
				"entity_type":      "EMAIL",
				"start":            start,
				"end":              end,
				"text":             email,
				"score":            0.99,
				"recognizer_name":  "mock",
				"metadata":         map[string]string{},
			},
		},
		ProcessingTimeMs: 1,
		TextLength:       len(text),
	}
	b, err := json.Marshal(r)
	return string(b), err
}

func (m *MockEngine) AnalyzeBatch(_ context.Context, texts []string) (string, error) {
	out := make([]analysisResult, 0, len(texts))
	for _, t := range texts {
		j, _ := m.Analyze(context.Background(), t, "")
		var one analysisResult
		_ = json.Unmarshal([]byte(j), &one)
		out = append(out, one)
	}
	b, err := json.Marshal(out)
	return string(b), err
}

func (m *MockEngine) Anonymize(_ context.Context, text string, _ string) (string, error) {
	aj, _ := m.Analyze(context.Background(), text, "")
	type wrap struct {
		Anonymized map[string]interface{} `json:"anonymized"`
		Analysis   json.RawMessage        `json:"analysis"`
	}
	w := wrap{
		Anonymized: map[string]interface{}{
			"text":   "[REDACTED]",
			"spans":  []interface{}{},
			"tokens": []interface{}{},
		},
		Analysis: json.RawMessage(aj),
	}
	b, err := json.Marshal(w)
	return string(b), err
}

func (m *MockEngine) Deanonymize(_ context.Context, _ string) (string, error) {
	return "", fmt.Errorf("%w", ErrNotImplemented)
}

func (m *MockEngine) LastError() string { return "" }

func (m *MockEngine) Version() string { return "mock-0.0.0" }
