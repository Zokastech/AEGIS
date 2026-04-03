// AEGIS — zokastech.fr — Apache 2.0 / MIT

package apikey

import (
	"crypto/sha256"
	"crypto/subtle"
	"encoding/hex"
)

// HashSecret computes SHA-256(pepper + ":" + secret) for at-rest storage.
func HashSecret(pepper, plain string) string {
	h := sha256.Sum256([]byte(pepper + ":" + plain))
	return hex.EncodeToString(h[:])
}

// ConstantTimeEquals compares two hex strings encoding hashes of equal length.
func ConstantTimeEquals(a, b string) bool {
	ab, err1 := hex.DecodeString(a)
	bb, err2 := hex.DecodeString(b)
	if err1 != nil || err2 != nil || len(ab) != len(bb) {
		return false
	}
	return subtle.ConstantTimeCompare(ab, bb) == 1
}
