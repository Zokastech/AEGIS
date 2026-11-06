// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rbac

import (
	"os"
	"path/filepath"
	"testing"
)

func TestDefaultMatrixLeastPrivilege(t *testing.T) {
	if Can(EffectivePermissions(RoleViewer, nil), PermDeanonymizeExecute) ||
		Can(EffectivePermissions(RoleOperator, nil), PermConfigWrite) ||
		Can(EffectivePermissions(RoleAuditor, nil), PermAnalyzeExecute) {
		t.Fatal("matrice par défaut trop permissive")
	}
	if !Can(EffectivePermissions(RoleAdmin, nil), PermDeanonymizeExecute) {
		t.Fatal("admin sans deanonymize")
	}
	if len(EffectivePermissions("unknown-role", nil)) != 0 {
		t.Fatal("rôle inconnu avec permissions")
	}
}

func TestLoadYAMLMappingsAndEntityFilter(t *testing.T) {
	p := filepath.Join(t.TempDir(), "rbac.yaml")
	_ = os.WriteFile(p, []byte(`
api_key_roles: {k1: admin}
jwt_subject_roles: {alice: viewer}
mtls_cn_roles: {svc: operator}
role_entity_types: {viewer: [EMAIL]}
`), 0o600)
	s, err := LoadYAML(p)
	if err != nil {
		t.Fatal(err)
	}
	if r, ok := s.RoleForAPIKey("k1"); !ok || r != RoleAdmin {
		t.Fatal("api key role")
	}
	if r, ok := s.RoleForJWTSubject("alice"); !ok || r != RoleViewer {
		t.Fatal("jwt role")
	}
	if r, ok := s.RoleForMTLSCN("svc"); !ok || r != RoleOperator {
		t.Fatal("mtls role")
	}
	if _, ok := s.RoleForMTLSCN("nobody"); ok {
		t.Fatal("CN inconnu mappé")
	}
	if !s.EntityTypeAllowed(RoleViewer, "EMAIL") || s.EntityTypeAllowed(RoleViewer, "IBAN") || !s.EntityTypeAllowed(RoleAdmin, "IBAN") {
		t.Fatal("filtrage des types d’entités")
	}
	if _, err := LoadYAML(filepath.Join(t.TempDir(), "absent.yaml")); err == nil {
		t.Fatal("fichier absent accepté")
	}
}
