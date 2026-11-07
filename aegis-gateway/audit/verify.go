// AEGIS — zokastech.fr — Apache 2.0 / MIT

package audit

import (
	"encoding/json"
	"fmt"
)

// VerifyChain checks the SHA-256 hash chain of JSONL audit lines (tamper evidence).
// prevHash is the hash preceding the first line ("" for a fresh log). It returns the index of the
// first broken line, or -1 when the whole chain is valid.
func VerifyChain(lines []string, prevHash string) (int, error) {
	for i, l := range lines {
		if l == "" {
			continue
		}
		var e Entry
		if err := json.Unmarshal([]byte(l), &e); err != nil {
			return i, fmt.Errorf("audit: ligne %d illisible: %w", i, err)
		}
		if e.PrevHashHex != prevHash {
			return i, fmt.Errorf("audit: ligne %d : chaînage rompu", i)
		}
		h, err := ChainHash(prevHash, e)
		if err != nil {
			return i, err
		}
		if h != e.EntryHashHex {
			return i, fmt.Errorf("audit: ligne %d : empreinte invalide (entrée modifiée)", i)
		}
		prevHash = h
	}
	return -1, nil
}
