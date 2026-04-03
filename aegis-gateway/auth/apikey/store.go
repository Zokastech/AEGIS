// AEGIS — zokastech.fr — Apache 2.0 / MIT

package apikey

import (
	"errors"
	"os"
	"sync"
	"time"

	"gopkg.in/yaml.v3"
)

// Record is one persisted API key row (YAML file).
type Record struct {
	ID               string    `yaml:"id"`
	SecretSHA256Hex  string    `yaml:"secret_sha256_hex"`
	Role             string    `yaml:"role"`
	CreatedAtRFC3339 string    `yaml:"created_at"`
	Revoked          bool      `yaml:"revoked"`
	RotatedFromID    string    `yaml:"rotated_from_id,omitempty"`
	Notes            string    `yaml:"notes,omitempty"`
}

// FileStore loads API keys from a file.
type FileStore struct {
	mu     sync.RWMutex
	path   string
	Pepper string       `yaml:"pepper"`
	Keys   []Record     `yaml:"keys"`
	byID   map[string]*Record
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
	b, err := yaml.Marshal(fs)
	if err != nil {
		return err
	}
	return os.WriteFile(fs.path, b, 0o600)
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
	fs.Keys = append(fs.Keys, rec)
	fs.byID[id] = &fs.Keys[len(fs.Keys)-1]
	return fs.saveLocked()
}

func (fs *FileStore) saveLocked() error {
	b, err := yaml.Marshal(fs)
	if err != nil {
		return err
	}
	return os.WriteFile(fs.path, b, 0o600)
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
	fs.byID[newID] = &fs.Keys[len(fs.Keys)-1]
	return fs.saveLocked()
}

var (
	ErrPepperUnset = errors.New("apikey: pepper requis (fichier api_keys ou AEGIS_API_KEY_PEPPER)")
	ErrKeyNotFound = errors.New("apikey: clé inconnue")
)
