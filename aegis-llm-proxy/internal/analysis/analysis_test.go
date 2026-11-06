// AEGIS — zokastech.fr — Apache 2.0 / MIT

package analysis

import (
	"testing"

	"github.com/zokastech/aegis/aegis-llm-proxy/internal/config"
)

func TestShouldBlockAndProtected(t *testing.T) {
	r, err := Parse([]byte(`{"entities":[{"entity_type":"EMAIL","score":0.7},{"entity_type":"PERSON","score":0.4}]}`))
	if err != nil {
		t.Fatal(err)
	}
	c := &config.Config{}
	if !ShouldBlock(c, r) || !HasProtectedPII(c, r) {
		t.Fatal("aucun filtre : toute entité doit bloquer")
	}
	c.ProtectedEntityTypes = []string{"IBAN"}
	if ShouldBlock(c, r) || HasProtectedPII(c, r) {
		t.Fatal("filtre IBAN ne doit pas bloquer EMAIL")
	}
	c.ProtectedEntityTypes = []string{"EMAIL"}
	c.BlockMinScore = 0.8
	if ShouldBlock(c, r) {
		t.Fatal("score 0.7 < 0.8 ne doit pas bloquer")
	}
	c.BlockMinScore = 0.5
	if !ShouldBlock(c, r) {
		t.Fatal("score 0.7 >= 0.5 doit bloquer")
	}
	if ShouldBlock(c, nil) || HasProtectedPII(c, nil) {
		t.Fatal("nil")
	}
	if Summary(nil) == "" || Summary(r) == "" {
		t.Fatal("summary")
	}
	if _, err := Parse([]byte(`x`)); err == nil {
		t.Fatal("JSON invalide accepté")
	}
}
