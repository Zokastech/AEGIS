// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rbac

import (
	"fmt"
	"os"
	"sync"

	"gopkg.in/yaml.v3"
)

// YAMLStore maps identities (API key, JWT sub, mTLS CN) to roles.
type YAMLStore struct {
	mu sync.RWMutex

	APIKeyRoles      map[string]string            `yaml:"api_key_roles"`
	JWTSubjectRoles  map[string]string            `yaml:"jwt_subject_roles"`
	MTLSCNRoles      map[string]string            `yaml:"mtls_cn_roles"`
	RolePermissions  map[string][]string          `yaml:"role_permissions_override"`
	// RoleEntityTypes limits entity types per role (empty = no limit).
	RoleEntityTypes map[string][]string `yaml:"role_entity_types"`
}

// EmptyYAMLStore returns a store with no bindings (tests / fallback).
func EmptyYAMLStore() *YAMLStore {
	return &YAMLStore{
		APIKeyRoles:     map[string]string{},
		JWTSubjectRoles: map[string]string{},
		MTLSCNRoles:     map[string]string{},
		RoleEntityTypes: map[string][]string{},
	}
}

// LoadYAML loads an rbac.yaml file.
func LoadYAML(path string) (*YAMLStore, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var s YAMLStore
	if err := yaml.Unmarshal(b, &s); err != nil {
		return nil, err
	}
	if s.APIKeyRoles == nil {
		s.APIKeyRoles = map[string]string{}
	}
	if s.JWTSubjectRoles == nil {
		s.JWTSubjectRoles = map[string]string{}
	}
	if s.MTLSCNRoles == nil {
		s.MTLSCNRoles = map[string]string{}
	}
	if s.RoleEntityTypes == nil {
		s.RoleEntityTypes = map[string][]string{}
	}
	return &s, nil
}

// EntityTypeAllowed reports whether the role may see this entity type (analyze / meta).
func (s *YAMLStore) EntityTypeAllowed(role, entityType string) bool {
	s.mu.RLock()
	defer s.mu.RUnlock()
	list, ok := s.RoleEntityTypes[role]
	if !ok || len(list) == 0 {
		return true
	}
	for _, t := range list {
		if t == entityType {
			return true
		}
	}
	return false
}

// RoleForAPIKey returns the role for a key ID (not the secret).
func (s *YAMLStore) RoleForAPIKey(keyID string) (string, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	r, ok := s.APIKeyRoles[keyID]
	return r, ok
}

// RoleForJWTSubject maps the sub claim to a role.
func (s *YAMLStore) RoleForJWTSubject(sub string) (string, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	r, ok := s.JWTSubjectRoles[sub]
	return r, ok
}

// RoleForMTLSCN maps client CN to a role.
func (s *YAMLStore) RoleForMTLSCN(cn string) (string, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	r, ok := s.MTLSCNRoles[cn]
	return r, ok
}

// PermissionsForRole returns effective permissions for a role.
func (s *YAMLStore) PermissionsForRole(role string) []string {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return EffectivePermissions(role, s.RolePermissions)
}

// PostgresRoleResolver resolves roles from PostgreSQL (optional).
type PostgresRoleResolver struct {
	// Empty DSN disables the resolver.
	DSN string
}

// RoleForSubject returns the role for a subject (e.g. api_key_id) — not implemented without the SQL driver.
func (p *PostgresRoleResolver) RoleForSubject(kind, subject string) (string, error) {
	if p.DSN == "" {
		return "", fmt.Errorf("rbac: postgres DSN vide")
	}
	return "", fmt.Errorf("rbac: compilez avec -tags aegis_postgres pour le backend SQL (voir rbac/store_postgres.go)")
}
