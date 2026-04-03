// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"strings"
	"testing"
)

func TestMinimizeAtIngestion_TruncateAndStrip(t *testing.T) {
	pol := &PolicyDocument{
		Name: "t",
		DataMinimization: DataMinimizationConfig{
			Enabled:                true,
			MaxInputRunes:          5,
			StripControlCharacters: true,
		},
	}
	rep := NewMinimizationReport("t")
	in := "ab\x01cd\x7fefghi"
	out, err := MinimizeAtIngestion(in, pol, rep)
	if err != nil {
		t.Fatal(err)
	}
	if utf8Len(out) != 5 {
		t.Fatalf("longueur runes: %q → %d", out, utf8Len(out))
	}
	if out != "abcde" {
		t.Fatalf("troncature attendue abcde, obtenu %q", out)
	}
	if len(rep.Events) == 0 {
		t.Fatal("aucun événement de minimisation")
	}
}

func utf8Len(s string) int {
	return len([]rune(s))
}

func TestMinimizeDisabledPassthrough(t *testing.T) {
	pol := &PolicyDocument{DataMinimization: DataMinimizationConfig{Enabled: false}}
	out, err := MinimizeAtIngestion("hello", pol, nil)
	if err != nil || out != "hello" {
		t.Fatalf("%q %v", out, err)
	}
}

func TestStripControlRunesPreservesNewline(t *testing.T) {
	s := "a\nb\x00c"
	out := stripControlRunes(s)
	if !strings.Contains(out, "\n") || strings.Contains(out, "\x00") {
		t.Fatalf("%q", out)
	}
}
