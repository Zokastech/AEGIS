// AEGIS — zokastech.fr — Apache 2.0 / MIT

package metrics

import (
	"reflect"
	"testing"
)

func TestLanguageAndPipeline_Empty(t *testing.T) {
	lang, pl := LanguageAndPipeline("")
	if lang != "unknown" || pl != "unknown" {
		t.Fatalf("empty: got lang=%q pipeline=%q", lang, pl)
	}
}

func TestLanguageAndPipeline_ValidJSON(t *testing.T) {
	lang, pl := LanguageAndPipeline(`{"language":"fr","pipeline_level":2}`)
	if lang != "fr" {
		t.Fatalf("lang: got %q", lang)
	}
	if pl != "2" {
		t.Fatalf("pipeline: got %q", pl)
	}
}

func TestLanguageAndPipeline_StringPipeline(t *testing.T) {
	_, pl := LanguageAndPipeline(`{"language":"de","pipeline_level":"3"}`)
	if pl != "3" {
		t.Fatalf("pipeline: got %q", pl)
	}
}

func TestLanguageAndPipeline_InvalidJSON(t *testing.T) {
	lang, pl := LanguageAndPipeline(`{`)
	if lang != "unknown" || pl != "unknown" {
		t.Fatalf("invalid json: got lang=%q pipeline=%q", lang, pl)
	}
}

func TestCountEntitiesByType_RootEntities(t *testing.T) {
	raw := []byte(`{"entities":[{"entity_type":"EMAIL"},{"entity_type":"PHONE"},{"entity_type":"EMAIL"}]}`)
	got := CountEntitiesByType(raw, "")
	// entity_type keys pass through SanitizeLabel (lowercase, Prometheus-safe).
	want := map[string]int{"email": 2, "phone": 1}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("got %#v want %#v", got, want)
	}
}

func TestCountEntitiesByType_NestedResult(t *testing.T) {
	raw := []byte(`{"result":{"entities":[{"entity_type":"PERSON"}]}}`)
	got := CountEntitiesByType(raw, "")
	if got["person"] != 1 {
		t.Fatalf("got %#v", got)
	}
}

func TestCountEntitiesByType_Empty(t *testing.T) {
	if len(CountEntitiesByType(nil, "")) != 0 {
		t.Fatal("expected empty map")
	}
}
