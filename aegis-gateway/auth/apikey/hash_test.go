// AEGIS — zokastech.fr — Apache 2.0 / MIT

package apikey

import "testing"

func TestHashSecret_Deterministic(t *testing.T) {
	a := HashSecret("pepper", "secret")
	b := HashSecret("pepper", "secret")
	if a != b {
		t.Fatalf("same inputs: got %q and %q", a, b)
	}
	if len(a) != 64 {
		t.Fatalf("expected 64 hex chars (sha256), got len=%d", len(a))
	}
}

func TestHashSecret_DifferentPepperOrSecret(t *testing.T) {
	h1 := HashSecret("p1", "s")
	h2 := HashSecret("p2", "s")
	h3 := HashSecret("p1", "t")
	if h1 == h2 || h1 == h3 {
		t.Fatal("hash should differ when pepper or secret changes")
	}
}

func TestConstantTimeEquals(t *testing.T) {
	h := HashSecret("x", "y")
	if !ConstantTimeEquals(h, h) {
		t.Fatal("same hash should match")
	}
	if ConstantTimeEquals(h, HashSecret("x", "z")) {
		t.Fatal("different secrets must not match")
	}
	if ConstantTimeEquals("nothex", h) {
		t.Fatal("invalid hex should not match")
	}
	if ConstantTimeEquals(h, "zz") {
		t.Fatal("invalid hex should not match")
	}
}
