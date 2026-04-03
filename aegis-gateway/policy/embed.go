// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import "embed"

//go:embed policies/*.yaml
var embeddedPolicies embed.FS

// DefaultEngine loads embedded policies plus optional overlay from policyDir on disk.
func DefaultEngine(policyDir string, store MappingStore) (*PolicyEngine, error) {
	return LoadEmbedded(embeddedPolicies, "policies", policyDir, store)
}
