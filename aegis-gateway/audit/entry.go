// AEGIS — zokastech.fr — Apache 2.0 / MIT

package audit

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"time"
)

// Entry is one immutable audit log line (SHA-256 chaining).
type Entry struct {
	TimestampRFC3339 string `json:"ts"`
	Actor            string `json:"actor"`
	AuthMethod       string `json:"auth_method,omitempty"`
	Action           string `json:"action"`
	Endpoint         string `json:"endpoint"`
	Method           string `json:"method,omitempty"`
	RequestID        string `json:"request_id,omitempty"`
	Success          bool   `json:"success"`
	StatusCode       int    `json:"status_code,omitempty"`
	PrevHashHex      string `json:"prev_hash_sha256"`
	EntryHashHex     string `json:"entry_hash_sha256"`
}

// ChainHash computes SHA-256(prevHashHex + "|" + payload without entry hash).
func ChainHash(prevHashHex string, e Entry) (string, error) {
	e.EntryHashHex = ""
	payload, err := json.Marshal(e)
	if err != nil {
		return "", err
	}
	h := sha256.New()
	if _, err := fmt.Fprintf(h, "%s|", prevHashHex); err != nil {
		return "", err
	}
	if _, err := h.Write(payload); err != nil {
		return "", err
	}
	return hex.EncodeToString(h.Sum(nil)), nil
}

// NewEntry builds an entry with a UTC timestamp.
func NewEntry(actor, authMethod, action, endpoint, method, requestID string, success bool, status int, prevHashHex string) (Entry, string, error) {
	e := Entry{
		TimestampRFC3339: time.Now().UTC().Format(time.RFC3339Nano),
		Actor:            actor,
		AuthMethod:       authMethod,
		Action:           action,
		Endpoint:         endpoint,
		Method:           method,
		RequestID:        requestID,
		Success:          success,
		StatusCode:       status,
		PrevHashHex:      prevHashHex,
	}
	eh, err := ChainHash(prevHashHex, e)
	if err != nil {
		return Entry{}, "", err
	}
	e.EntryHashHex = eh
	return e, eh, nil
}
