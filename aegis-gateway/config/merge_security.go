// AEGIS — zokastech.fr — Apache 2.0 / MIT

// AEGIS — zokastech.fr — Apache 2.0 / MIT

package config

import (
	"github.com/spf13/viper"
)

// MergeSecurityConfig overlays explicit file values (viper) on secure defaults.
func MergeSecurityConfig(base SecurityConfig, o SecurityConfig, v *viper.Viper) SecurityConfig {
	out := base
	if v.IsSet("security.production") {
		out.Production = o.Production
	}
	if v.IsSet("security.development.disable_auth") {
		out.Development.DisableAuth = o.Development.DisableAuth
	}
	if v.IsSet("security.tls.enabled") {
		out.TLS.Enabled = o.TLS.Enabled
	}
	if o.TLS.CertFile != "" {
		out.TLS.CertFile = o.TLS.CertFile
	}
	if o.TLS.KeyFile != "" {
		out.TLS.KeyFile = o.TLS.KeyFile
	}
	if o.TLS.ClientCAFile != "" {
		out.TLS.ClientCAFile = o.TLS.ClientCAFile
	}
	if v.IsSet("security.tls.auto_generate_self_signed") {
		out.TLS.AutoGenerateSelfSigned = o.TLS.AutoGenerateSelfSigned
	}
	if o.TLS.AutoGenCertDir != "" {
		out.TLS.AutoGenCertDir = o.TLS.AutoGenCertDir
	}
	if v.IsSet("security.mtls.require") {
		out.MTLS.Require = o.MTLS.Require
	}
	if v.IsSet("security.cors.allow_origins") {
		out.CORS.AllowOrigins = o.CORS.AllowOrigins
	}
	if o.APIKeys.FilePath != "" {
		out.APIKeys.FilePath = o.APIKeys.FilePath
	}
	if o.APIKeys.Pepper != "" {
		out.APIKeys.Pepper = o.APIKeys.Pepper
	}
	if o.RBAC.FilePath != "" {
		out.RBAC.FilePath = o.RBAC.FilePath
	}
	if o.RBAC.Backend != "" {
		out.RBAC.Backend = o.RBAC.Backend
	}
	if o.RBAC.PostgresDSN != "" {
		out.RBAC.PostgresDSN = o.RBAC.PostgresDSN
	}
	if v.IsSet("security.jwt.enabled") {
		out.JWT.Enabled = o.JWT.Enabled
	}
	if o.JWT.Issuer != "" {
		out.JWT.Issuer = o.JWT.Issuer
	}
	if o.JWT.Audience != "" {
		out.JWT.Audience = o.JWT.Audience
	}
	if o.JWT.JWKSURL != "" {
		out.JWT.JWKSURL = o.JWT.JWKSURL
	}
	if o.JWT.HMACSecret != "" {
		out.JWT.HMACSecret = o.JWT.HMACSecret
	}
	if v.IsSet("security.oidc.enabled") {
		out.OIDC.Enabled = o.OIDC.Enabled
	}
	if o.OIDC.IssuerURL != "" {
		out.OIDC.IssuerURL = o.OIDC.IssuerURL
	}
	if o.OIDC.ClientID != "" {
		out.OIDC.ClientID = o.OIDC.ClientID
	}
	if o.Audit.Backend != "" {
		out.Audit.Backend = o.Audit.Backend
	}
	if o.Audit.FilePath != "" {
		out.Audit.FilePath = o.Audit.FilePath
	}
	if o.Audit.PostgresDSN != "" {
		out.Audit.PostgresDSN = o.Audit.PostgresDSN
	}
	if o.Audit.RotateMaxMB > 0 {
		out.Audit.RotateMaxMB = o.Audit.RotateMaxMB
	}
	if o.Audit.RotateMaxAge > 0 {
		out.Audit.RotateMaxAge = o.Audit.RotateMaxAge
	}
	if o.Audit.ArchiveDir != "" {
		out.Audit.ArchiveDir = o.Audit.ArchiveDir
	}
	if o.Deanon.MaxPerHour > 0 {
		out.Deanon.MaxPerHour = o.Deanon.MaxPerHour
	}
	if o.Deanon.ApprovalHeader1 != "" {
		out.Deanon.ApprovalHeader1 = o.Deanon.ApprovalHeader1
	}
	if o.Deanon.ApprovalHeader2 != "" {
		out.Deanon.ApprovalHeader2 = o.Deanon.ApprovalHeader2
	}
	if o.Deanon.CircuitMaxSuccess > 0 {
		out.Deanon.CircuitMaxSuccess = o.Deanon.CircuitMaxSuccess
	}
	if o.Deanon.CircuitWindowMinutes > 0 {
		out.Deanon.CircuitWindowMinutes = o.Deanon.CircuitWindowMinutes
	}
	if o.Alerts.WebhookURL != "" {
		out.Alerts.WebhookURL = o.Alerts.WebhookURL
	}
	if len(o.PublicPaths) > 0 {
		out.PublicPaths = o.PublicPaths
	}
	return out
}
