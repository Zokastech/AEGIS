// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"context"
	"sync"
	"time"
)

// MappingStore persists pseudonym-to-person links (Redis/PostgreSQL in production).
type MappingStore interface {
	// DeleteBySubject removes all entries for a subject (GDPR Art. 17 erasure).
	DeleteBySubject(ctx context.Context, subjectID string) (removed int, err error)
	// Register associates a mapping key with a subject (called on pseudonymization).
	Register(ctx context.Context, subjectID, mappingKey string, expiresAt *time.Time) error
}

// MemoryMappingStore is an in-memory implementation (tests / dev without Redis).
type MemoryMappingStore struct {
	mu       sync.Mutex
	bySub    map[string]map[string]struct{} // subject -> set of keys
	keysMeta map[string]time.Time          // key -> optional expiry (zero = none)
}

// NewMemoryMappingStore creates an empty store.
func NewMemoryMappingStore() *MemoryMappingStore {
	return &MemoryMappingStore{
		bySub:    make(map[string]map[string]struct{}),
		keysMeta: make(map[string]time.Time),
	}
}

// Register stores a mapping key for a subject.
func (m *MemoryMappingStore) Register(_ context.Context, subjectID, mappingKey string, expiresAt *time.Time) error {
	if subjectID == "" || mappingKey == "" {
		return nil
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.bySub[subjectID] == nil {
		m.bySub[subjectID] = make(map[string]struct{})
	}
	m.bySub[subjectID][mappingKey] = struct{}{}
	if expiresAt != nil {
		m.keysMeta[mappingKey] = *expiresAt
	}
	return nil
}

// DeleteBySubject removes all keys for the subject.
func (m *MemoryMappingStore) DeleteBySubject(_ context.Context, subjectID string) (int, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	set := m.bySub[subjectID]
	if set == nil {
		return 0, nil
	}
	n := len(set)
	for k := range set {
		delete(m.keysMeta, k)
	}
	delete(m.bySub, subjectID)
	return n, nil
}

// CountMappings returns mapping count for a subject (tests).
func (m *MemoryMappingStore) CountMappings(subjectID string) int {
	m.mu.Lock()
	defer m.mu.Unlock()
	return len(m.bySub[subjectID])
}
