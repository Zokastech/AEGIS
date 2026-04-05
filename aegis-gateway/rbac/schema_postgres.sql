-- AEGIS — zokastech.fr — Apache 2.0 / MIT
-- Liaison sujet → rôle (IdP / clés API ID) pour backend postgres RBAC.

CREATE TABLE IF NOT EXISTS aegis_rbac_bindings (
    subject_kind   TEXT NOT NULL,  -- api_key_id | jwt_sub | mtls_cn
    subject_value  TEXT NOT NULL,
    role           TEXT NOT NULL,
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (subject_kind, subject_value)
);

CREATE INDEX IF NOT EXISTS idx_aegis_rbac_role ON aegis_rbac_bindings (role);
