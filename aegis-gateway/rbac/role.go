// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rbac

// Built-in AEGIS roles.
const (
	RoleViewer   = "viewer"
	RoleOperator = "operator"
	RoleAdmin    = "admin"
	RoleAuditor  = "auditor"
)

// Fine-grained permissions (endpoint / capability).
const (
	PermPublicProbe = "probe:public"

	PermAnalyzeExecute    = "analyze:execute"
	PermAnalyzeBatch      = "analyze:batch"
	PermAnonymizeExecute  = "anonymize:execute"
	PermDeanonymizeExecute = "deanonymize:execute"
	PermConfigWrite       = "config:write"
	PermMetaRead          = "meta:read"
	PermMetricsView       = "metrics:view"
	PermAuditRead         = "audit:read"
	PermAuditExport       = "audit:export"
	PermAPIKeysManage     = "apikeys:manage"
	PermPolicyList        = "policy:list"
	PermPolicyDPIA        = "policy:dpia"
	PermSubjectsErase     = "subjects:erase"
)

// DefaultRolePermissions maps role → permissions.
var DefaultRolePermissions = map[string][]string{
	RoleViewer: {
		PermAnalyzeExecute,
		PermAnalyzeBatch,
		PermMetaRead,
	},
	RoleOperator: {
		PermAnalyzeExecute,
		PermAnalyzeBatch,
		PermAnonymizeExecute,
		PermMetaRead,
	},
	RoleAdmin: {
		PermAnalyzeExecute,
		PermAnalyzeBatch,
		PermAnonymizeExecute,
		PermDeanonymizeExecute,
		PermConfigWrite,
		PermMetaRead,
		PermMetricsView,
		PermAuditRead,
		PermAuditExport,
		PermAPIKeysManage,
		PermPolicyList,
		PermPolicyDPIA,
		PermSubjectsErase,
	},
	RoleAuditor: {
		PermAuditRead,
		PermAuditExport,
		PermMetricsView,
		PermMetaRead,
		PermPolicyList,
		PermPolicyDPIA,
	},
}

// EffectivePermissions returns permissions for a role (with optional YAML override).
func EffectivePermissions(role string, override map[string][]string) []string {
	if override != nil {
		if p, ok := override[role]; ok {
			return p
		}
	}
	return DefaultRolePermissions[role]
}

// Can reports whether a permission is granted.
func Can(granted []string, perm string) bool {
	for _, g := range granted {
		if g == perm {
			return true
		}
	}
	return false
}
