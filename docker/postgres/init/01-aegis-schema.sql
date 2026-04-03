-- AEGIS — zokastech.fr — Apache 2.0 / MIT
-- Agrégat audit + RBAC (backends postgres optionnels du gateway).

CREATE TABLE IF NOT EXISTS aegis_audit_log (
    id              BIGSERIAL PRIMARY KEY,
    ts              TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor           TEXT NOT NULL,
    auth_method     TEXT,
    action          TEXT NOT NULL,
    endpoint        TEXT NOT NULL,
    method          TEXT,
    request_id      TEXT,
    success         BOOLEAN NOT NULL,
    status_code     INT,
    prev_hash_sha256 TEXT NOT NULL,
    entry_hash_sha256 TEXT NOT NULL UNIQUE,
    raw_json        JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_aegis_audit_ts ON aegis_audit_log (ts DESC);
CREATE INDEX IF NOT EXISTS idx_aegis_audit_actor ON aegis_audit_log (actor);
CREATE INDEX IF NOT EXISTS idx_aegis_audit_action ON aegis_audit_log (action);
CREATE INDEX IF NOT EXISTS idx_aegis_audit_request ON aegis_audit_log (request_id);

CREATE TABLE IF NOT EXISTS aegis_rbac_bindings (
    subject_kind   TEXT NOT NULL,
    subject_value  TEXT NOT NULL,
    role           TEXT NOT NULL,
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (subject_kind, subject_value)
);

CREATE INDEX IF NOT EXISTS idx_aegis_rbac_role ON aegis_rbac_bindings (role);
