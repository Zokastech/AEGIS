// AEGIS — zokastech.fr — Apache 2.0 / MIT

package providers

import (
	"encoding/json"
	"strings"
)

// MutateAnthropicMessages parcourt system (string ou blocs) et messages[].content.
func MutateAnthropicMessages(body []byte, mut func(string) (string, error)) ([]byte, error) {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return nil, err
	}
	if err := mutateAnthropicSystem(root, mut); err != nil {
		return nil, err
	}
	if msgs, ok := root["messages"].([]interface{}); ok {
		for _, m := range msgs {
			msg, ok := m.(map[string]interface{})
			if !ok {
				continue
			}
			if err := mutateAnthropicMessageContent(msg, mut); err != nil {
				return nil, err
			}
		}
	}
	return json.Marshal(root)
}

func mutateAnthropicSystem(root map[string]interface{}, mut func(string) (string, error)) error {
	sys, ok := root["system"]
	if !ok {
		return nil
	}
	switch v := sys.(type) {
	case string:
		n, err := mut(v)
		if err != nil {
			return err
		}
		root["system"] = n
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
		root["system"] = v
	}
	return nil
}

func mutateAnthropicMessageContent(msg map[string]interface{}, mut func(string) (string, error)) error {
	c, ok := msg["content"]
	if !ok {
		return nil
	}
	switch v := c.(type) {
	case string:
		n, err := mut(v)
		if err != nil {
			return err
		}
		msg["content"] = n
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
		msg["content"] = v
	}
	return nil
}

// RestoreAnthropicResponse applique fn sur chaque bloc texte de content[].
func RestoreAnthropicResponse(body []byte, fn func(string) string) ([]byte, error) {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return nil, err
	}
	content, ok := root["content"].([]interface{})
	if !ok {
		return body, nil
	}
	for _, item := range content {
		bl, ok := item.(map[string]interface{})
		if !ok {
			continue
		}
		if bl["type"] == "text" {
			if s, ok := bl["text"].(string); ok {
				bl["text"] = fn(s)
			}
		}
	}
	root["content"] = content
	return json.Marshal(root)
}

// CollectAnthropicText pour analyse globale.
func CollectAnthropicText(body []byte) string {
	var root map[string]interface{}
	if err := json.Unmarshal(body, &root); err != nil {
		return ""
	}
	var b strings.Builder
	_ = mutateAnthropicSystemCollect(root, &b)
	if msgs, ok := root["messages"].([]interface{}); ok {
		for _, m := range msgs {
			msg, ok := m.(map[string]interface{})
			if !ok {
				continue
			}
			appendAnthropicMsgContent(msg, &b)
		}
	}
	return b.String()
}

func mutateAnthropicSystemCollect(root map[string]interface{}, b *strings.Builder) error {
	sys, ok := root["system"]
	if !ok {
		return nil
	}
	switch v := sys.(type) {
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
	return nil
}

func appendAnthropicMsgContent(msg map[string]interface{}, b *strings.Builder) {
	c, ok := msg["content"]
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
