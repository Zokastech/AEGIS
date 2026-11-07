// AEGIS — zokastech.fr — Apache 2.0 / MIT

package deanon

import "testing"

func TestParseAnonymizedFFIAndDirect(t *testing.T) {
	ffi := `{"anonymized":{"text":"hi [E]","transformations":[{"replacement":"[E]","original_text":"a@b.c"}]},"analysis":{}}`
	p, err := ParseAnonymized([]byte(ffi))
	if err != nil || p.Text != "hi [E]" || len(p.Transformations) != 1 {
		t.Fatalf("ffi: %+v %v", p, err)
	}
	p, err = ParseAnonymized([]byte(`{"text":"x","transformations":[]}`))
	if err != nil || p.Text != "x" {
		t.Fatalf("direct: %+v %v", p, err)
	}
	if _, err := ParseAnonymized([]byte(`{}`)); err == nil {
		t.Fatal("payload vide accepté")
	}
	if _, err := ParseAnonymized([]byte(`nope`)); err == nil {
		t.Fatal("JSON invalide accepté")
	}
}

func TestMergeAndRestoreLongestFirst(t *testing.T) {
	a := &AnonymizedPayload{Transformations: []Transformation{{Replacement: "[E1]", OriginalText: "a@b.c"}}}
	b := &AnonymizedPayload{
		Transformations: []Transformation{{Replacement: "[E10]", OriginalText: "z@y.x"}},
		MappingHints:    map[string]string{"[P]": "Jean"},
	}
	m := Merge([]*AnonymizedPayload{a, nil, b})
	got := Restore("[E1] [E10] [P]", m)
	if got != "a@b.c z@y.x Jean" {
		t.Fatalf("restore: %q", got)
	}
	if Merge(nil) != nil || Merge([]*AnonymizedPayload{{}}) != nil {
		t.Fatal("merge vide doit renvoyer nil")
	}
	if Restore("unchanged", nil) != "unchanged" {
		t.Fatal("restore nil")
	}
}
