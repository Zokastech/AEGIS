// AEGIS — zokastech.fr — Apache 2.0 / MIT

package metrics

import (
	"encoding/json"
	"strconv"
)

// LanguageAndPipeline extracts language and pipeline_level from analysis_config_json.
func LanguageAndPipeline(analysisJSON string) (lang, pipeline string) {
	lang, pipeline = "unknown", "unknown"
	if analysisJSON == "" {
		return
	}
	var m map[string]interface{}
	if err := json.Unmarshal([]byte(analysisJSON), &m); err != nil {
		return
	}
	if v, ok := m["language"].(string); ok && v != "" {
		lang = SanitizeLabel(v)
	}
	switch v := m["pipeline_level"].(type) {
	case float64:
		pipeline = strconv.Itoa(int(v))
	case int:
		pipeline = strconv.Itoa(v)
	case string:
		if v != "" {
			pipeline = SanitizeLabel(v)
		}
	default:
	}
	if pipeline == "unknown" || pipeline == "" {
		pipeline = "unknown"
	}
	return lang, pipeline
}

// CountEntitiesByType counts entities in engine JSON output (root or .result).
func CountEntitiesByType(raw []byte, _ string) map[string]int {
	out := make(map[string]int)
	if len(raw) == 0 {
		return out
	}
	var root map[string]interface{}
	if err := json.Unmarshal(raw, &root); err != nil {
		return out
	}
	entities := extractEntitiesArray(root)
	for _, e := range entities {
		em, ok := e.(map[string]interface{})
		if !ok {
			continue
		}
		typ, _ := em["entity_type"].(string)
		if typ == "" {
			typ = "unknown"
		}
		typ = SanitizeLabel(typ)
		out[typ]++
	}
	return out
}

func extractEntitiesArray(root map[string]interface{}) []interface{} {
	if ent, ok := root["entities"].([]interface{}); ok {
		return ent
	}
	if res, ok := root["result"].(map[string]interface{}); ok {
		if ent, ok := res["entities"].([]interface{}); ok {
			return ent
		}
	}
	return nil
}

// ObserveAnonymizeOperators parses anonymized JSON and increments anonymize_operations_total.
func ObserveAnonymizeOperators(resultJSON []byte) {
	if len(resultJSON) == 0 {
		AnonymizeOpsTotal.WithLabelValues("unknown").Inc()
		return
	}
	var root map[string]interface{}
	if err := json.Unmarshal(resultJSON, &root); err != nil {
		AnonymizeOpsTotal.WithLabelValues("unknown").Inc()
		return
	}
	anon, _ := root["anonymized"].(map[string]interface{})
	if anon != nil {
		if ops, ok := anon["operators"].([]interface{}); ok && len(ops) > 0 {
			for _, o := range ops {
				label := "unknown"
				switch t := o.(type) {
				case string:
					label = SanitizeLabel(t)
				case map[string]interface{}:
					if n, ok := t["type"].(string); ok && n != "" {
						label = SanitizeLabel(n)
					} else if n, ok := t["name"].(string); ok && n != "" {
						label = SanitizeLabel(n)
					}
				}
				AnonymizeOpsTotal.WithLabelValues(label).Inc()
			}
			return
		}
		if op, ok := anon["operator"].(string); ok && op != "" {
			AnonymizeOpsTotal.WithLabelValues(SanitizeLabel(op)).Inc()
			return
		}
	}
	// Minimal response (e.g. mock): count one generic operation
	AnonymizeOpsTotal.WithLabelValues("mask_or_replace").Inc()
}
