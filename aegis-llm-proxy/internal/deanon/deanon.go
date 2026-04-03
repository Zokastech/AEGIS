// AEGIS — zokastech.fr — Apache 2.0 / MIT

package deanon

import (
	"encoding/json"
	"errors"
	"sort"
	"strings"
)

// Transformation correspond au JSON moteur (snake_case).
type Transformation struct {
	Replacement   string `json:"replacement"`
	OriginalText  string `json:"original_text"`
	EntityStart   int    `json:"entity_start"`
	EntityEnd     int    `json:"entity_end"`
	Operator      string `json:"operator"`
	EntityType    string `json:"entity_type"`
}

// AnonymizedPayload extrait de la réponse FFI anonymize.
type AnonymizedPayload struct {
	Text            string            `json:"text"`
	Transformations []Transformation  `json:"transformations"`
	MappingHints    map[string]string `json:"mapping_hints"`
}

// FfiAnonymizeResult racine `result` gateway : anonymized + analysis.
type FfiAnonymizeResult struct {
	Anonymized json.RawMessage `json:"anonymized"`
	Analysis   json.RawMessage `json:"analysis"`
}

// ParseAnonymized extrait le bloc anonymized depuis le JSON `result`.
func ParseAnonymized(resultJSON []byte) (*AnonymizedPayload, error) {
	var top FfiAnonymizeResult
	if err := json.Unmarshal(resultJSON, &top); err == nil && len(top.Anonymized) > 0 {
		var a AnonymizedPayload
		if err := json.Unmarshal(top.Anonymized, &a); err != nil {
			return nil, err
		}
		return &a, nil
	}
	var direct AnonymizedPayload
	if err := json.Unmarshal(resultJSON, &direct); err != nil {
		return nil, err
	}
	if direct.Text != "" || len(direct.Transformations) > 0 {
		return &direct, nil
	}
	return nil, errors.New("résultat anonymize sans texte ni transformations")
}

// Merge fusionne plusieurs charges utiles d’anonymisation (requête multi-segments).
func Merge(parts []*AnonymizedPayload) *AnonymizedPayload {
	if len(parts) == 0 {
		return nil
	}
	out := &AnonymizedPayload{MappingHints: map[string]string{}}
	for _, p := range parts {
		if p == nil {
			continue
		}
		out.Transformations = append(out.Transformations, p.Transformations...)
		for k, v := range p.MappingHints {
			if k != "" && v != "" {
				out.MappingHints[k] = v
			}
		}
	}
	if len(out.Transformations) == 0 && len(out.MappingHints) == 0 {
		return nil
	}
	return out
}

// Restore remplace les pseudonymes / masques présents dans la réponse LLM par les valeurs d’origine.
func Restore(llmResponse string, a *AnonymizedPayload) string {
	if a == nil {
		return llmResponse
	}
	type pair struct {
		from string
		to   string
	}
	var ps []pair
	seen := map[string]bool{}
	for _, t := range a.Transformations {
		if t.Replacement == "" || t.OriginalText == "" {
			continue
		}
		k := t.Replacement + "\x00" + t.OriginalText
		if seen[k] {
			continue
		}
		seen[k] = true
		ps = append(ps, pair{from: t.Replacement, to: t.OriginalText})
	}
	for k, v := range a.MappingHints {
		if k != "" && v != "" {
			ps = append(ps, pair{from: k, to: v})
		}
	}
	sort.Slice(ps, func(i, j int) bool {
		return len(ps[i].from) > len(ps[j].from)
	})
	out := llmResponse
	for _, p := range ps {
		out = strings.ReplaceAll(out, p.from, p.to)
	}
	return out
}
