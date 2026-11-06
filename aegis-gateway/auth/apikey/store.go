// AEGIS — zokastech.fr — Apache 2.0 / MIT

package apikey

import (
	"errors"
	"os"
	"path/filepath"
	"sync"
	"time"

	"gopkg.in/yaml.v3"
)

// Record is one persisted API key row (YAML file).
type Record struct {
	ID               string `yaml:"id"`
	SecretSHA256Hex  string `yaml:"secret_sha256_hex"`
	Role             string `yaml:"role"`
	CreatedAtRFC3339 string `yaml:"created_at"`
	Revoked          bool   `yaml:"revoked"`
	RotatedFromID    string `yaml:"rotated_from_id,omitempty"`
	Notes            string `yaml:"notes,omitempty"`
}

// FileStore loads API keys from a file.
type FileStore struct {
	mu     sync.RWMutex
	path   string
	Pepper string   `yaml:"pepper"`
	Keys   []Record `yaml:"keys"`
	byID   map[string]*Record
	// filePepper is the pepper read from the YAML file (kept on Save). A pepper injected through
	// AEGIS_API_KEY_PEPPER is never written to disk next to the hashes it protects.
	filePepper string
}

type persistedStore struct {
	Pepper string   `yaml:"pepper,omitempty"`
	Keys   []Record `yaml:"keys"`
}

// LoadFileStore reads YAML (empty store if the file is missing).
func LoadFileStore(path, envPepper string) (*FileStore, error) {
	fs := &FileStore{path: path, byID: map[string]*Record{}}
	b, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			if envPepper != "" {
				fs.Pepper = envPepper
			}
			return fs, nil
		}
		return nil, err
	}
	if err := yaml.Unmarshal(b, fs); err != nil {
		return nil, err
	}
	fs.filePepper = fs.Pepper
	if envPepper != "" {
		fs.Pepper = envPepper
	}
	fs.reindex()
	return fs, nil
}

func (fs *FileStore) reindex() {
	fs.byID = make(map[string]*Record)
	for i := range fs.Keys {
		r := &fs.Keys[i]
		fs.byID[r.ID] = r
	}
}

// Validate returns (keyID, role) when the secret matches an active key.
func (fs *FileStore) Validate(plainSecret string) (keyID, role string, ok bool) {
	fs.mu.RLock()
	defer fs.mu.RUnlock()
	if fs.Pepper == "" || plainSecret == "" {
		return "", "", false
	}
	h := HashSecret(fs.Pepper, plainSecret)
	for i := range fs.Keys {
		r := &fs.Keys[i]
		if r.Revoked {
			continue
		}
		if ConstantTimeEquals(r.SecretSHA256Hex, h) {
			return r.ID, r.Role, true
		}
	}
	return "", "", false
}

// LookupID returns the record by ID (for admin dual approval).
func (fs *FileStore) LookupID(id string) (*Record, bool) {
	fs.mu.RLock()
	defer fs.mu.RUnlock()
	r, ok := fs.byID[id]
	if !ok || r.Revoked {
		return nil, false
	}
	return r, true
}

// ValidateKeyIDWithSecret checks that an ID matches the given secret (separate id+secret header patterns).
// Validates secret only; dual approval uses two full secrets → two Validate calls → both key IDs must be admin.
func (fs *FileStore) ValidateKeyIDWithSecret(id, plainSecret string) (role string, ok bool) {
	fs.mu.RLock()
	defer fs.mu.RUnlock()
	r, ok := fs.byID[id]
	if !ok || r.Revoked || fs.Pepper == "" {
		return "", false
	}
	h := HashSecret(fs.Pepper, plainSecret)
	if !ConstantTimeEquals(r.SecretSHA256Hex, h) {
		return "", false
	}
	return r.Role, true
}

// Save persists the store (rotation / revocation).
func (fs *FileStore) Save() error {
	fs.mu.Lock()
	defer fs.mu.Unlock()
	return fs.saveLocked()
}

// Generate adds a new key; the plain secret is returned only once by the caller.
func (fs *FileStore) Generate(id, role string, plainSecret string) error {
	fs.mu.Lock()
	defer fs.mu.Unlock()
	if fs.Pepper == "" {
		return ErrPepperUnset
	}
	rec := Record{
		ID:               id,
		SecretSHA256Hex:  HashSecret(fs.Pepper, plainSecret),
		Role:             role,
		CreatedAtRFC3339: time.Now().UTC().Format(time.RFC3339),
		Revoked:          false,
	}
	if _, exists := fs.byID[id]; exists {
		return ErrKeyExists
	}
	fs.Keys = append(fs.Keys, rec)
	// append may reallocate the backing array: rebuild the index so that byID never points
	// to a stale copy (otherwise Revoke would flip the flag on a copy and the key stayed valid).
	fs.reindex()
	return fs.saveLocked()
}

func (fs *FileStore) saveLocked() error {
	b, err := yaml.Marshal(persistedStore{Pepper: fs.filePepper, Keys: fs.Keys})
	if err != nil {
		return err
	}
	return writeFileAtomic(fs.path, b, 0o600)
}

// writeFileAtomic writes to a temp file in the same directory then renames it,
// so a crash never leaves a truncated key store.
func writeFileAtomic(path string, data []byte, perm os.FileMode) error {
	if path == "" {
		return errors.New("apikey: chemin du fichier non défini")
	}
	tmp, err := os.CreateTemp(filepath.Dir(path), ".api_keys-*.tmp")
	if err != nil {
		return err
	}
	tmpName := tmp.Name()
	defer func() { _ = os.Remove(tmpName) }()
	if _, err := tmp.Write(data); err != nil {
		_ = tmp.Close()
		return err
	}
	if err := tmp.Chmod(perm); err != nil {
		_ = tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	return os.Rename(tmpName, path)
}

// Revoke marks a key as revoked.
func (fs *FileStore) Revoke(id string) error {
	fs.mu.Lock()
	defer fs.mu.Unlock()
	r, ok := fs.byID[id]
	if !ok {
		return ErrKeyNotFound
	}
	r.Revoked = true
	return fs.saveLocked()
}

// Rotate creates a new key with the same role and revokes the old one.
func (fs *FileStore) Rotate(oldID, newID, newPlain string) error {
	fs.mu.Lock()
	defer fs.mu.Unlock()
	old, ok := fs.byID[oldID]
	if !ok || old.Revoked {
		return ErrKeyNotFound
	}
	if _, exists := fs.byID[newID]; exists {
		return ErrKeyExists
	}
	if fs.Pepper == "" {
		return ErrPepperUnset
	}
	old.Revoked = true
	rec := Record{
		ID:               newID,
		SecretSHA256Hex:  HashSecret(fs.Pepper, newPlain),
		Role:             old.Role,
		CreatedAtRFC3339: time.Now().UTC().Format(time.RFC3339),
		Revoked:          false,
		RotatedFromID:    oldID,
	}
	fs.Keys = append(fs.Keys, rec)
	fs.reindex()
	return fs.saveLocked()
}

var (
	ErrPepperUnset = errors.New("apikey: pepper requis (fichier api_keys ou AEGIS_API_KEY_PEPPER)")
	ErrKeyNotFound = errors.New("apikey: clé inconnue")
	ErrKeyExists   = errors.New("apikey: identifiant déjà utilisé")
)
