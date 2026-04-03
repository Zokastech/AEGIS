// AEGIS — zokastech.fr — Apache 2.0 / MIT

package analysis

import (
	"encoding/json"
	"fmt"

	"github.com/zokastech/aegis/aegis-llm-proxy/internal/config"
)

// Entity détectée (JSON moteur).
type Entity struct {
	EntityType string  `json:"entity_type"`
	Score      float64 `json:"score"`
	Text       string  `json:"text"`
}

// Result racine analyse FFI.
type Result struct {
	Entities []Entity `json:"entities"`
}

// Parse extrait les entités depuis le JSON `result` analyze.
func Parse(resultJSON []byte) (*Result, error) {
	var r Result
	if err := json.Unmarshal(resultJSON, &r); err != nil {
		return nil, err
	}
	return &r, nil
}

// AnalysisConfigJSON construit le JSON passé à /v1/analyze.
func AnalysisConfigJSON(c *config.Config) string {
	b, _ := json.Marshal(map[string]float64{
		"score_threshold": c.ScoreThreshold,
	})
	return string(b)
}

// ShouldBlock retourne vrai si une entité protégée dépasse le seuil de blocage.
func ShouldBlock(c *config.Config, r *Result) bool {
	if r == nil {
		return false
	}
	protected := c.ProtectedEntityTypes
	for _, e := range r.Entities {
		if !entityMatchesFilter(e.EntityType, protected) {
			continue
		}
		if c.BlockMinScore <= 0 {
			return true
		}
		if e.Score >= c.BlockMinScore {
			return true
		}
	}
	return false
}

func entityMatchesFilter(entityType string, protected []string) bool {
	if len(protected) == 0 {
		return true
	}
	for _, p := range protected {
		if p == entityType {
			return true
		}
	}
	return false
}

// HasProtectedPII indique s’il existe au moins une entité filtrée (pour logs / alertes).
func HasProtectedPII(c *config.Config, r *Result) bool {
	if r == nil {
		return false
	}
	protected := c.ProtectedEntityTypes
	for _, e := range r.Entities {
		if entityMatchesFilter(e.EntityType, protected) {
			return true
		}
	}
	return false
}

// Summary court pour webhooks / logs.
func Summary(r *Result) string {
	if r == nil || len(r.Entities) == 0 {
		return "aucune entité"
	}
	return fmt.Sprintf("%d entité(s)", len(r.Entities))
}
