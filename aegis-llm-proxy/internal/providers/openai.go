// AEGIS — zokastech.fr — Apache 2.0 / MIT

package providers

import (
	"encoding/json"
	"strings"
)

// MutateOpenAIChat parcourt messages[].content (string ou blocs) et prompt système éventuel.
func MutateOpenAIChat(body []byte, mut func(string) (string, error)) ([]byte, error) {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return nil, err
	}
	if msgs, ok := root["messages"].([]interface{}); ok {
		for _, m := range msgs {
			msg, ok := m.(map[string]interface{})
			if !ok {
				continue
			}
			if err := mutateContentField(msg, "content", mut); err != nil {
				return nil, err
			}
		}
	}
	out, err := json.Marshal(root)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// MutateOpenAICompletion legacy : champ "prompt" (string ou liste).
func MutateOpenAICompletion(body []byte, mut func(string) (string, error)) ([]byte, error) {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return nil, err
	}
	switch p := root["prompt"].(type) {
	case string:
		n, err := mut(p)
		if err != nil {
			return nil, err
		}
		root["prompt"] = n
	case []interface{}:
		for i, x := range p {
			if s, ok := x.(string); ok {
				n, err := mut(s)
				if err != nil {
					return nil, err
				}
				p[i] = n
			}
		}
		root["prompt"] = p
	}
	return json.Marshal(root)
}

func mutateContentField(msg map[string]interface{}, key string, mut func(string) (string, error)) error {
	c, ok := msg[key]
	if !ok {
		return nil
	}
	switch v := c.(type) {
	case string:
		n, err := mut(v)
		if err != nil {
			return err
		}
		msg[key] = n
	case []interface{}:
		for _, item := range v {
			bl, ok := item.(map[string]interface{})
			if !ok {
				continue
			}
			if bl["type"] == "text" {
				if s, ok := bl["text"].(string); ok {
					n, err := mut(s)
					if err != nil {
						return err
					}
					bl["text"] = n
				}
			}
		}
		msg[key] = v
	}
	return nil
}

// RestoreOpenAIChatResponse applique fn sur chaque fragment de texte dans choices[].message.content.
func RestoreOpenAIChatResponse(body []byte, fn func(string) string) ([]byte, error) {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return nil, err
	}
	choices, ok := root["choices"].([]interface{})
	if !ok {
		return body, nil
	}
	for _, ch := range choices {
		cm, ok := ch.(map[string]interface{})
		if !ok {
			continue
		}
		msg, ok := cm["message"].(map[string]interface{})
		if !ok {
			continue
		}
		_ = mutateContentField(msg, "content", func(s string) (string, error) {
			return fn(s), nil
		})
	}
	return json.Marshal(root)
}

// RestoreOpenAICompletionResponse restaure choices[].text.
func RestoreOpenAICompletionResponse(body []byte, fn func(string) string) ([]byte, error) {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return nil, err
	}
	choices, ok := root["choices"].([]interface{})
	if !ok {
		return body, nil
	}
	for _, ch := range choices {
		cm, ok := ch.(map[string]interface{})
		if !ok {
			continue
		}
		if s, ok := cm["text"].(string); ok {
			cm["text"] = fn(s)
		}
	}
	return json.Marshal(root)
}

// CollectOpenAIChatText concatène le texte utilisateur pour une analyse globale (block / log).
func CollectOpenAIChatText(body []byte) string {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return ""
	}
	var b stringsBuilder
	msgs, ok := root["messages"].([]interface{})
	if !ok {
		return ""
	}
	for _, m := range msgs {
		msg, ok := m.(map[string]interface{})
		if !ok {
			continue
		}
		appendContentText(msg, "content", &b)
	}
	return b.String()
}

// CollectOpenAICompletionText extrait le prompt pour analyse.
func CollectOpenAICompletionText(body []byte) string {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return ""
	}
	var b strings.Builder
	switch p := root["prompt"].(type) {
	case string:
		b.WriteString(p)
	case []interface{}:
		for _, x := range p {
			if s, ok := x.(string); ok {
				b.WriteString(s)
				b.WriteByte('\n')
			}
		}
	}
	return b.String()
}

func appendContentText(msg map[string]interface{}, key string, b *strings.Builder) {
	c, ok := msg[key]
	if !ok {
		return
	}
	switch v := c.(type) {
	case string:
		b.WriteString(v)
		b.WriteByte('\n')
	case []interface{}:
		for _, item := range v {
			bl, ok := item.(map[string]interface{})
			if !ok {
				continue
			}
			if bl["type"] == "text" {
				if s, ok := bl["text"].(string); ok {
					b.WriteString(s)
					b.WriteByte('\n')
				}
			}
		}
	}
}
