// AEGIS — zokastech.fr — Apache 2.0 / MIT

package apikey

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func newStore(t *testing.T) (*FileStore, string) {
	t.Helper()
	p := filepath.Join(t.TempDir(), "api_keys.yaml")
	fs, err := LoadFileStore(p, "env-pepper-value")
	if err != nil {
		t.Fatal(err)
	}
	return fs, p
}

func TestGenerateValidate(t *testing.T) {
	fs, _ := newStore(t)
	if err := fs.Generate("k1", "admin", "s1"); err != nil {
		t.Fatal(err)
	}
	id, role, ok := fs.Validate("s1")
	if !ok || id != "k1" || role != "admin" {
		t.Fatalf("validate: %q %q %v", id, role, ok)
	}
	if _, _, ok := fs.Validate("s2"); ok {
		t.Fatal("secret inconnu accepté")
	}
	if _, _, ok := fs.Validate(""); ok {
		t.Fatal("secret vide accepté")
	}
}

// Regression: after Generate re-allocated the slice, Revoke flipped a stale copy and the key stayed valid.
func TestRevokeAfterManyGenerates(t *testing.T) {
	fs, _ := newStore(t)
	if err := fs.Generate("victim", "admin", "victim-secret"); err != nil {
		t.Fatal(err)
	}
	for i := 0; i < 20; i++ {
		if err := fs.Generate("k"+strings.Repeat("x", i), "viewer", "s"+strings.Repeat("y", i)); err != nil {
			t.Fatal(err)
		}
	}
	if err := fs.Revoke("victim"); err != nil {
		t.Fatal(err)
	}
	if _, _, ok := fs.Validate("victim-secret"); ok {
		t.Fatal("clé révoquée toujours valide")
	}
	if _, ok := fs.LookupID("victim"); ok {
		t.Fatal("LookupID renvoie une clé révoquée")
	}
}

func TestRotate(t *testing.T) {
	fs, _ := newStore(t)
	_ = fs.Generate("old", "operator", "old-secret")
	for i := 0; i < 5; i++ {
		_ = fs.Generate("pad"+string(rune('a'+i)), "viewer", "pad"+string(rune('a'+i)))
	}
	if err := fs.Rotate("old", "new", "new-secret"); err != nil {
		t.Fatal(err)
	}
	if _, _, ok := fs.Validate("old-secret"); ok {
		t.Fatal("ancienne clé encore valide après rotation")
	}
	id, role, ok := fs.Validate("new-secret")
	if !ok || id != "new" || role != "operator" {
		t.Fatalf("nouvelle clé: %q %q %v", id, role, ok)
	}
	if err := fs.Rotate("old", "again", "x"); err == nil {
		t.Fatal("rotation d’une clé révoquée acceptée")
	}
	if err := fs.Rotate("new", "pada", "x"); err == nil {
		t.Fatal("rotation vers un id existant acceptée")
	}
	if _, _, ok := fs.Validate("new-secret"); !ok {
		t.Fatal("échec de rotation a révoqué la clé courante")
	}
}

func TestDuplicateIDRejected(t *testing.T) {
	fs, _ := newStore(t)
	_ = fs.Generate("dup", "viewer", "a")
	if err := fs.Generate("dup", "admin", "b"); err == nil {
		t.Fatal("id dupliqué accepté")
	}
}

func TestPersistenceNeverWritesEnvPepperOrSecrets(t *testing.T) {
	fs, p := newStore(t)
	if err := fs.Generate("k1", "viewer", "plain-secret-value"); err != nil {
		t.Fatal(err)
	}
	b, err := os.ReadFile(p)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(b), "env-pepper-value") {
		t.Fatal("pepper d’environnement écrit sur disque")
	}
	if strings.Contains(string(b), "plain-secret-value") {
		t.Fatal("secret en clair écrit sur disque")
	}
	st, _ := os.Stat(p)
	if st.Mode().Perm() != 0o600 {
		t.Fatalf("permissions %o, attendu 600", st.Mode().Perm())
	}
	// Reload with the same env pepper → still valid.
	fs2, err := LoadFileStore(p, "env-pepper-value")
	if err != nil {
		t.Fatal(err)
	}
	if _, _, ok := fs2.Validate("plain-secret-value"); !ok {
		t.Fatal("clé invalide après rechargement")
	}
	// Wrong pepper → invalid.
	fs3, _ := LoadFileStore(p, "other-pepper")
	if _, _, ok := fs3.Validate("plain-secret-value"); ok {
		t.Fatal("mauvais pepper accepté")
	}
}

func TestFilePepperPreserved(t *testing.T) {
	p := filepath.Join(t.TempDir(), "k.yaml")
	_ = os.WriteFile(p, []byte("pepper: file-pepper\nkeys: []\n"), 0o600)
	fs, err := LoadFileStore(p, "")
	if err != nil {
		t.Fatal(err)
	}
	if err := fs.Generate("k", "viewer", "s"); err != nil {
		t.Fatal(err)
	}
	b, _ := os.ReadFile(p)
	if !strings.Contains(string(b), "file-pepper") {
		t.Fatal("pepper du fichier perdu à la sauvegarde")
	}
}

func TestNoPepperNoKeys(t *testing.T) {
	fs, err := LoadFileStore(filepath.Join(t.TempDir(), "absent.yaml"), "")
	if err != nil {
		t.Fatal(err)
	}
	if err := fs.Generate("k", "viewer", "s"); err != ErrPepperUnset {
		t.Fatalf("want ErrPepperUnset got %v", err)
	}
	if _, _, ok := fs.Validate("s"); ok {
		t.Fatal("validation sans pepper")
	}
}

func TestValidateKeyIDWithSecret(t *testing.T) {
	fs, _ := newStore(t)
	_ = fs.Generate("k1", "admin", "s1")
	if r, ok := fs.ValidateKeyIDWithSecret("k1", "s1"); !ok || r != "admin" {
		t.Fatal("couple id/secret valide refusé")
	}
	if _, ok := fs.ValidateKeyIDWithSecret("k1", "s2"); ok {
		t.Fatal("mauvais secret accepté")
	}
	if _, ok := fs.ValidateKeyIDWithSecret("zz", "s1"); ok {
		t.Fatal("id inconnu accepté")
	}
}

func TestInvalidYAML(t *testing.T) {
	p := filepath.Join(t.TempDir(), "bad.yaml")
	_ = os.WriteFile(p, []byte("keys: [\n"), 0o600)
	if _, err := LoadFileStore(p, "x"); err == nil {
		t.Fatal("YAML invalide accepté")
	}
}
