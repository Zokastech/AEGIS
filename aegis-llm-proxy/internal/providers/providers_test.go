// AEGIS — zokastech.fr — Apache 2.0 / MIT

package providers

import (
	"encoding/json"
	"errors"
	"strings"
	"testing"
)

func upper(s string) (string, error) { return strings.ToUpper(s), nil }

func TestClassifyPath(t *testing.T) {
	cases := map[string]Kind{
		"/v1/chat/completions":          KindOpenAIChat,
		"/v1/chat/completions/":         KindOpenAIChat,
		"/openai/v1/chat/completions?x": KindOpenAIChat,
		"/v1/completions":               KindOpenAICompletion,
		"/v1/messages":                  KindAnthropicMessages,
		"/v1/embeddings":                KindUnknown,
		"/":                             KindUnknown,
		"":                              KindUnknown,
		"%zz":                           KindUnknown,
		"/v1/models?limit=1":            KindUnknown,
	}
	for in, want := range cases {
		if got := ClassifyPath(in); got != want {
			t.Errorf("ClassifyPath(%q)=%v want %v", in, got, want)
		}
	}
}

func TestPeekStream(t *testing.T) {
	if !PeekStream([]byte(`{"stream":true}`)) {
		t.Fatal("stream:true non détecté")
	}
	for _, b := range []string{`{"stream":false}`, `{}`, `not json`, ``} {
		if PeekStream([]byte(b)) {
			t.Fatalf("faux positif pour %q", b)
		}
	}
}

func TestMutateOpenAIChatStringAndParts(t *testing.T) {
	body := `{"messages":[{"role":"user","content":"abc"},{"role":"user","content":[{"type":"text","text":"def"}]}]}`
	out, err := MutateOpenAIChat([]byte(body), upper)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(out), "ABC") || !strings.Contains(string(out), "DEF") {
		t.Fatalf("mutation incomplète: %s", out)
	}
	if got := CollectOpenAIChatText([]byte(body)); !strings.Contains(got, "abc") || !strings.Contains(got, "def") {
		t.Fatalf("collect: %q", got)
	}
}

func TestMutateOpenAIChatPropagatesError(t *testing.T) {
	_, err := MutateOpenAIChat([]byte(`{"messages":[{"content":"x"}]}`), func(string) (string, error) {
		return "", errors.New("boom")
	})
	if err == nil {
		t.Fatal("erreur du moteur ignorée : le prompt partirait en clair")
	}
}

func TestMutateOpenAICompletionPromptForms(t *testing.T) {
	for _, body := range []string{`{"prompt":"abc"}`, `{"prompt":["abc","xyz"]}`} {
		out, err := MutateOpenAICompletion([]byte(body), upper)
		if err != nil {
			t.Fatal(err)
		}
		if strings.Contains(string(out), "abc") {
			t.Fatalf("prompt non muté: %s", out)
		}
	}
	if got := CollectOpenAICompletionText([]byte(`{"prompt":["a1","b2"]}`)); !strings.Contains(got, "a1") || !strings.Contains(got, "b2") {
		t.Fatalf("collect completion: %q", got)
	}
}

func TestMutateAnthropicSystemAndMessages(t *testing.T) {
	body := `{"system":"sys secret","messages":[{"role":"user","content":"hello"},{"role":"user","content":[{"type":"text","text":"world"}]}]}`
	out, err := MutateAnthropicMessages([]byte(body), upper)
	if err != nil {
		t.Fatal(err)
	}
	s := string(out)
	for _, w := range []string{"SYS SECRET", "HELLO", "WORLD"} {
		if !strings.Contains(s, w) {
			t.Fatalf("%s absent de %s", w, s)
		}
	}
	col := CollectAnthropicText([]byte(body))
	for _, w := range []string{"sys secret", "hello", "world"} {
		if !strings.Contains(col, w) {
			t.Fatalf("collect: %s absent de %q", w, col)
		}
	}
}

func TestRestoreResponses(t *testing.T) {
	fn := func(s string) string { return strings.ReplaceAll(s, "[E]", "a@b.c") }
	out, err := RestoreOpenAIChatResponse([]byte(`{"choices":[{"message":{"content":"to [E]"}}]}`), fn)
	if err != nil || !strings.Contains(string(out), "a@b.c") {
		t.Fatalf("openai chat restore: %s %v", out, err)
	}
	out, err = RestoreOpenAICompletionResponse([]byte(`{"choices":[{"text":"to [E]"}]}`), fn)
	if err != nil || !strings.Contains(string(out), "a@b.c") {
		t.Fatalf("openai completion restore: %s %v", out, err)
	}
	out, err = RestoreAnthropicResponse([]byte(`{"content":[{"type":"text","text":"to [E]"}]}`), fn)
	if err != nil || !strings.Contains(string(out), "a@b.c") {
		t.Fatalf("anthropic restore: %s %v", out, err)
	}
	if _, err := RestoreOpenAIChatResponse([]byte(`not json`), fn); err == nil {
		t.Fatal("JSON invalide accepté")
	}
}

func TestInvalidJSONRejected(t *testing.T) {
	for _, f := range []func([]byte, func(string) (string, error)) ([]byte, error){MutateOpenAIChat, MutateOpenAICompletion, MutateAnthropicMessages} {
		if _, err := f([]byte(`{`), upper); err == nil {
			t.Fatal("JSON invalide accepté")
		}
	}
	var v map[string]interface{}
	if json.Unmarshal([]byte(`{}`), &v) != nil {
		t.Fatal()
	}
}
