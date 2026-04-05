// AEGIS — zokastech.fr — Apache 2.0 / MIT

package providers

import (
	"encoding/json"
	"net/url"
	"strings"
)

// Kind identifie le format de corps attendu.
type Kind int

const (
	KindUnknown Kind = iota
	KindOpenAIChat
	KindOpenAICompletion
	KindAnthropicMessages
)

// ClassifyPath déduit le provider depuis le chemin HTTP.
func ClassifyPath(path string) Kind {
	p := strings.TrimSuffix(path, "/")
	switch {
	case strings.Contains(p, "/chat/completions"):
		return KindOpenAIChat
	case strings.HasSuffix(p, "/completions") && !strings.Contains(p, "/chat/completions"):
		return KindOpenAICompletion
	case strings.HasSuffix(p, "/messages") && strings.Contains(p, "/v1/messages"):
		return KindAnthropicMessages
	default:
		if u, err := url.Parse(path); err == nil {
			return ClassifyPath(u.Path)
		}
		return KindUnknown
	}
}

// PeekStream détecte `"stream":true` dans un JSON (corps requête OpenAI / compatible).
func PeekStream(body []byte) bool {
	var v struct {
		Stream bool `json:"stream"`
	}
	if json.Unmarshal(body, &v) == nil && v.Stream {
		return true
	}
	return false
}
