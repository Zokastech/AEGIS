// AEGIS — zokastech.fr — Apache 2.0 / MIT

package config

import (
	"fmt"
	"os"
	"strings"

	"gopkg.in/yaml.v3"
)

// Mode d’interception des prompts.
type Mode string

const (
	ModeTransparent Mode = "transparent"
	ModeAnonymize Mode = "anonymize"
	ModeBlock     Mode = "block"
	ModeAlert     Mode = "alert"
)

// EngineType : moteur AEGIS via HTTP (gateway) ou CLI local.
type EngineType string

const (
	EngineHTTP EngineType = "http"
	EngineCLI  EngineType = "cli"
)

// Config fichier YAML + surcharge par variables d’environnement.
type Config struct {
	Listen string `yaml:"listen"`

	// UpstreamURL URL du fournisseur LLM (ex. https://api.openai.com ou http://ollama:11434).
	UpstreamURL string `yaml:"upstream_url"`

	Mode Mode `yaml:"mode"`

	// ScoreThreshold seuil analyse (transmis au moteur dans analysis_config_json).
	ScoreThreshold float64 `yaml:"score_threshold"`

	// BlockMinScore en mode block : bloquer si une entité dépasse ce score (0 = toute PII).
	BlockMinScore float64 `yaml:"block_min_score"`

	// WebhookURL mode alert : POST JSON asynchrone.
	WebhookURL string `yaml:"webhook_url"`

	// AnonymizeConfigJSON opérateurs FFI (ex. pseudonymize par type).
	AnonymizeConfigJSON string `yaml:"anonymize_config_json"`

	// ProtectedEntityTypes si non vide, seuls ces types (config_key) déclenchent anonymisation / block.
	ProtectedEntityTypes []string `yaml:"protected_entity_types"`

	Engine EngineBlock `yaml:"engine"`

	// InjectAPIKeyEnv nom de variable : si la requête cliente n’a pas Authorization, injecter Bearer depuis cette env.
	InjectAPIKeyEnv string `yaml:"inject_api_key_env"`

	Dashboard DashboardConfig `yaml:"dashboard"`
}

// EngineBlock configuration accès moteur.
type EngineBlock struct {
	Type    EngineType `yaml:"type"`
	BaseURL string     `yaml:"base_url"`
	// CLIPath chemin binaire aegis (mode cli).
	CLIPath string `yaml:"cli_path"`
	// TimeoutSecond timeout appels moteur.
	TimeoutSeconds int `yaml:"timeout_seconds"`
}

// DashboardConfig exposition stats pour le dashboard AEGIS.
type DashboardConfig struct {
	Enabled bool   `yaml:"enabled"`
	Prefix  string `yaml:"prefix"`
}

// Default values.
func (c *Config) applyDefaults() {
	if c.Listen == "" {
		c.Listen = ":8080"
	}
	if c.Mode == "" {
		c.Mode = ModeAnonymize
	}
	if c.ScoreThreshold == 0 {
		c.ScoreThreshold = 0.5
	}
	if c.Engine.Type == "" {
		c.Engine.Type = EngineHTTP
	}
	if c.Engine.BaseURL == "" {
		c.Engine.BaseURL = "http://127.0.0.1:9847"
	}
	if c.Engine.TimeoutSeconds == 0 {
		c.Engine.TimeoutSeconds = 60
	}
	if c.AnonymizeConfigJSON == "" {
		c.AnonymizeConfigJSON = `{"analysis":{"score_threshold":0.5},"operators_by_entity":{"EMAIL":{"operator_type":"mask","params":{"visible_prefix":"2","visible_suffix":"0"}},"PHONE":{"operator_type":"mask","params":{"visible_prefix":"2","visible_suffix":"0"}},"CREDIT_CARD":{"operator_type":"redact","params":{}},"IBAN":{"operator_type":"redact","params":{}}}}`
	}
	if c.Dashboard.Prefix == "" {
		c.Dashboard.Prefix = "/internal/llm-proxy"
	}
}

// Load lit le YAML puis applique les variables d’environnement usuelles.
func Load(path string) (*Config, error) {
	raw, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var c Config
	if err := yaml.Unmarshal(raw, &c); err != nil {
		return nil, err
	}
	c.applyDefaults()

	if v := os.Getenv("AEGIS_UPSTREAM_URL"); v != "" {
		c.UpstreamURL = v
	}
	if v := os.Getenv("AEGIS_ENGINE_URL"); v != "" {
		c.Engine.BaseURL = strings.TrimSuffix(v, "/")
	}
	if v := os.Getenv("AEGIS_LLM_MODE"); v != "" {
		c.Mode = Mode(v)
	}
	if v := os.Getenv("AEGIS_WEBHOOK_URL"); v != "" {
		c.WebhookURL = v
	}
	if c.UpstreamURL == "" {
		return nil, fmt.Errorf("upstream_url requis (YAML ou AEGIS_UPSTREAM_URL)")
	}
	return &c, nil
}
