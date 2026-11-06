// AEGIS — zokastech.fr — Apache 2.0 / MIT

package config

import "time"

// SecurityConfig is the “secure by default” policy (see security.yaml).
type SecurityConfig struct {
	Production bool `mapstructure:"production"`

	Development SecurityDevConfig `mapstructure:"development"`

	TLS     SecurityTLSConfig         `mapstructure:"tls"`
	MTLS    SecurityMTLSConfig        `mapstructure:"mtls"`
	CORS    SecurityCORSConfig        `mapstructure:"cors"`
	APIKeys SecurityAPIKeysConfig     `mapstructure:"api_keys"`
	RBAC    SecurityRBACConfig        `mapstructure:"rbac"`
	JWT     SecurityJWTConfig         `mapstructure:"jwt"`
	OIDC    SecurityOIDCConfig        `mapstructure:"oidc"`
	Audit   SecurityAuditConfig       `mapstructure:"audit"`
	Deanon  SecurityDeanonymizeConfig `mapstructure:"deanonymize"`
	Alerts  SecurityAlertsConfig      `mapstructure:"alerts"`

	// Public paths without auth (K8s probes). Never use /* on /v1/* in production.
	PublicPaths []string `mapstructure:"public_paths"`

	// TrustedProxies lists CIDRs/IPs of reverse proxies allowed to set X-Forwarded-For / X-Real-IP.
	// Empty (default) = the TCP peer address is used (headers ignored → no rate-limit bypass by spoofing).
	TrustedProxies []string `mapstructure:"trusted_proxies"`

	// MaxBodyBytes caps HTTP request bodies (default 10 MiB).
	MaxBodyBytes int64 `mapstructure:"max_body_bytes"`

	GRPC SecurityGRPCConfig `mapstructure:"grpc"`
}

// SecurityGRPCConfig hardens the gRPC listener.
type SecurityGRPCConfig struct {
	// Reflection exposes the gRPC reflection service (default: only when production=false).
	Reflection bool `mapstructure:"reflection"`
	// MaxRecvMsgBytes caps incoming gRPC messages (default 10 MiB).
	MaxRecvMsgBytes int `mapstructure:"max_recv_msg_bytes"`
}

// SecurityDevConfig relaxes safeguards (tests only).
type SecurityDevConfig struct {
	DisableAuth bool `mapstructure:"disable_auth"`
}

type SecurityTLSConfig struct {
	Enabled                bool   `mapstructure:"enabled"`
	CertFile               string `mapstructure:"cert_file"`
	KeyFile                string `mapstructure:"key_file"`
	ClientCAFile           string `mapstructure:"client_ca_file"`
	AutoGenerateSelfSigned bool   `mapstructure:"auto_generate_self_signed"`
	AutoGenCertDir         string `mapstructure:"auto_gen_cert_dir"`
}

type SecurityMTLSConfig struct {
	Require bool `mapstructure:"require"`
}

type SecurityCORSConfig struct {
	AllowOrigins []string `mapstructure:"allow_origins"`
}

type SecurityAPIKeysConfig struct {
	FilePath string `mapstructure:"file_path"`
	Pepper   string `mapstructure:"pepper"`
}

type SecurityRBACConfig struct {
	FilePath    string `mapstructure:"file_path"`
	Backend     string `mapstructure:"backend"` // yaml | postgres
	PostgresDSN string `mapstructure:"postgres_dsn"`
}

type SecurityJWTConfig struct {
	Enabled    bool   `mapstructure:"enabled"`
	Issuer     string `mapstructure:"issuer"`
	Audience   string `mapstructure:"audience"`
	JWKSURL    string `mapstructure:"jwks_url"`
	HMACSecret string `mapstructure:"hmac_secret"`
}

type SecurityOIDCConfig struct {
	Enabled   bool   `mapstructure:"enabled"`
	IssuerURL string `mapstructure:"issuer_url"`
	ClientID  string `mapstructure:"client_id"`
}

type SecurityAuditConfig struct {
	Backend      string        `mapstructure:"backend"` // file | postgres
	FilePath     string        `mapstructure:"file_path"`
	PostgresDSN  string        `mapstructure:"postgres_dsn"`
	RotateMaxMB  int           `mapstructure:"rotate_max_mb"`
	RotateMaxAge time.Duration `mapstructure:"rotate_max_age"`
	ArchiveDir   string        `mapstructure:"archive_dir"`
}

type SecurityDeanonymizeConfig struct {
	MaxPerHour           int    `mapstructure:"max_per_hour"`
	ApprovalHeader1      string `mapstructure:"approval_header_1"`
	ApprovalHeader2      string `mapstructure:"approval_header_2"`
	CircuitMaxSuccess    int    `mapstructure:"circuit_max_success"`
	CircuitWindowMinutes int    `mapstructure:"circuit_window_minutes"`
}

type SecurityAlertsConfig struct {
	WebhookURL string `mapstructure:"webhook_url"`
}

// DefaultSecurity returns secure defaults (TLS + no wide-open CORS).
func DefaultSecurity() SecurityConfig {
	return SecurityConfig{
		Production: true,
		TLS: SecurityTLSConfig{
			Enabled:                true,
			CertFile:               "data/tls/server.crt",
			KeyFile:                "data/tls/server.key",
			AutoGenerateSelfSigned: true,
			AutoGenCertDir:         "data/tls",
		},
		MTLS: SecurityMTLSConfig{
			// true in prod with tls.client_ca_file; false by default for local boot without client PKI.
			Require: false,
		},
		CORS: SecurityCORSConfig{
			AllowOrigins: []string{},
		},
		APIKeys: SecurityAPIKeysConfig{
			FilePath: "config/api_keys.yaml",
		},
		RBAC: SecurityRBACConfig{
			FilePath: "config/rbac.yaml",
			Backend:  "yaml",
		},
		Audit: SecurityAuditConfig{
			Backend:      "file",
			FilePath:     "data/audit/audit.jsonl",
			RotateMaxMB:  100,
			RotateMaxAge: 24 * time.Hour,
			ArchiveDir:   "data/audit/archive",
		},
		Deanon: SecurityDeanonymizeConfig{
			MaxPerHour:           10,
			ApprovalHeader1:      "X-Aegis-Approval-1",
			ApprovalHeader2:      "X-Aegis-Approval-2",
			CircuitMaxSuccess:    5,
			CircuitWindowMinutes: 15,
		},
		PublicPaths:  []string{"/livez", "/readyz"},
		MaxBodyBytes: 10 << 20,
		GRPC: SecurityGRPCConfig{
			Reflection:      false,
			MaxRecvMsgBytes: 10 << 20,
		},
	}
}

// Validate rejects dangerous combinations at startup (fail closed).
// insecureHTTP mirrors AEGIS_INSECURE_HTTP=1 (cleartext listener, TLS checks skipped).
func (s SecurityConfig) Validate(insecureHTTP bool) error {
	if s.Production && s.Development.DisableAuth {
		return errSecurity("development.disable_auth=true est interdit quand production=true")
	}
	if !insecureHTTP && s.MTLS.Require && s.TLS.Enabled && !s.Development.DisableAuth && s.TLS.ClientCAFile == "" {
		return errSecurity("mtls.require=true impose tls.client_ca_file")
	}
	if s.JWT.Enabled && len(s.JWT.HMACSecret) < 32 {
		return errSecurity("jwt.hmac_secret doit contenir au moins 32 caractères")
	}
	return nil
}

type securityError string

func (e securityError) Error() string { return "security: " + string(e) }

func errSecurity(msg string) error { return securityError(msg) }
