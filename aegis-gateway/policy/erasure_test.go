// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"context"
	"errors"
	"path/filepath"
	"runtime"
	"testing"
	"time"
)

func testPolicyPath(t *testing.T, name string) string {
	t.Helper()
	_, file, _, ok := runtime.Caller(1)
	if !ok {
		t.Fatal("runtime.Caller")
	}
	return filepath.Join(filepath.Dir(file), "policies", name)
}

func TestErasureSubject(t *testing.T) {
	store := NewMemoryMappingStore()
	e := NewEngine(store)
	pol, err := LoadFile(testPolicyPath(t, "gdpr-analytics.yaml"))
	if err != nil {
		t.Fatal(err)
	}
	e.mu.Lock()
	e.policies[pol.Name] = pol
	e.mu.Unlock()

	ctx := context.Background()
	_ = store.Register(ctx, "user-42", "map-key-1", ptrTime(time.Now().Add(24*time.Hour)))
	_ = store.Register(ctx, "user-42", "map-key-2", nil)

	svc := NewErasureService(e)
	cert, err := svc.EraseSubject(ctx, "gdpr-analytics", "user-42")
	if err != nil {
		t.Fatal(err)
	}
	if cert.MappingsRemoved != 2 {
		t.Fatalf("removed=%d", cert.MappingsRemoved)
	}
	if cert.IntegrityHash == "" || cert.CertificateID == "" {
		t.Fatal("certificat incomplet")
	}
	if store.CountMappings("user-42") != 0 {
		t.Fatal("store non purgé")
	}
}

func TestErasureDisabled(t *testing.T) {
	store := NewMemoryMappingStore()
	e := NewEngine(store)
	pol, err := LoadFile(testPolicyPath(t, "gdpr-article-17.yaml"))
	if err != nil {
		t.Fatal(err)
	}
	// disable erasure
	pol.Rights.ErasureEndpointEnabled = false
	e.mu.Lock()
	e.policies[pol.Name] = pol
	e.mu.Unlock()

	svc := NewErasureService(e)
	_, err = svc.EraseSubject(context.Background(), "gdpr-article-17", "x")
	if !errors.Is(err, ErrErasureDisabled) {
		t.Fatalf("attendu ErrErasureDisabled, %v", err)
	}
}

func ptrTime(t time.Time) *time.Time { return &t }
