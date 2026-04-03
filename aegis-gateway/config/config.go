// AEGIS — zokastech.fr — Apache 2.0 / MIT

package config

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/fsnotify/fsnotify"
	"github.com/spf13/pflag"
	"github.com/spf13/viper"
)

// Config is the gateway runtime configuration.
type Config struct {
	HTTPListen        string        `mapstructure:"http_listen"`
	GRPCListen        string        `mapstructure:"grpc_listen"`
	ReadTimeout       time.Duration `mapstructure:"read_timeout"`
	WriteTimeout      time.Duration `mapstructure:"write_timeout"`
	ShutdownTimeout   time.Duration `mapstructure:"shutdown_timeout"`
	EngineTimeout     time.Duration `mapstructure:"engine_timeout"`
	PoolSize          int           `mapstructure:"pool_size"`
	EngineInitJSON    string        `mapstructure:"engine_init_json"`
	CORSAllowOrigins  []string      `mapstructure:"cors_allow_origins"`
	RateLimitRPS      float64       `mapstructure:"rate_limit_rps"`
	RateLimitBurst    int           `mapstructure:"rate_limit_burst"`
	APIKeyHeader      string        `mapstructure:"api_key_header"`
	TrustedAPIKeys    []string      `mapstructure:"trusted_api_keys"`
	AdminAPIKeys      []string      `mapstructure:"admin_api_keys"`
	ConfigFile        string        `mapstructure:"-"`
	HotReload         bool          `mapstructure:"hot_reload"`
	UseRustFFI        bool          `mapstructure:"use_rust_ffi"`
	Security          SecurityConfig `mapstructure:"security"`
	PolicyDir         string        `mapstructure:"policy_dir"` // optional override for embedded YAML policies
}

// Loader loads viper + CLI flags.
type Loader struct {
	mu     sync.RWMutex
	v      *viper.Viper
	cfg    Config
	onChange func(Config)
}

func NewLoader() *Loader {
	return &Loader{v: viper.New()}
}

func (l *Loader) BindFlags(fs *pflag.FlagSet) {
	fs.String("config", "", "chemin fichier YAML")
	fs.String("http-listen", ":8080", "adresse HTTP")
	fs.String("grpc-listen", ":9090", "adresse gRPC")
	fs.Bool("hot-reload", false, "recharger la config sans redémarrer")
	fs.Bool("use-rust-ffi", false, "utiliser CGO aegis-ffi (build tag aegisffi)")
	fs.String("security-config", "", "fichier YAML sécurité (fusionné après la config principale)")
}

func (l *Loader) Load(fs *pflag.FlagSet) (Config, error) {
	_ = fs.Parse(os.Args[1:])
	l.v.SetEnvPrefix("AEGIS")
	l.v.AutomaticEnv()

	if c, _ := fs.GetString("config"); c != "" {
		l.v.SetConfigFile(c)
	} else {
		l.v.SetConfigName("aegis-gateway")
		l.v.SetConfigType("yaml")
		l.v.AddConfigPath(".")
		l.v.AddConfigPath("/etc/aegis")
	}

	l.v.SetDefault("http_listen", ":8080")
	l.v.SetDefault("grpc_listen", ":9090")
	l.v.SetDefault("read_timeout", "30s")
	l.v.SetDefault("write_timeout", "30s")
	l.v.SetDefault("shutdown_timeout", "15s")
	l.v.SetDefault("engine_timeout", "20s")
	l.v.SetDefault("pool_size", 4)
	l.v.SetDefault("rate_limit_rps", 50)
	l.v.SetDefault("rate_limit_burst", 80)
	l.v.SetDefault("api_key_header", "X-API-Key")
	l.v.SetDefault("cors_allow_origins", []string{"*"})
	l.v.SetDefault("policy_dir", "")

	if err := l.v.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); !ok && l.v.ConfigFileUsed() != "" {
			return Config{}, fmt.Errorf("config: %w", err)
		}
	}

	l.mergeSecurityYAMLFiles()
	if p, _ := fs.GetString("security-config"); fs.Changed("security-config") && p != "" {
		b, err := os.ReadFile(p)
		if err != nil {
			return Config{}, fmt.Errorf("security-config: %w", err)
		}
		l.v.SetConfigType("yaml")
		if err := l.v.MergeConfig(bytes.NewReader(b)); err != nil {
			return Config{}, fmt.Errorf("security-config merge: %w", err)
		}
	}

	// Flags > env > file
	if s, _ := fs.GetString("http-listen"); fs.Changed("http-listen") {
		l.v.Set("http_listen", s)
	}
	if s, _ := fs.GetString("grpc-listen"); fs.Changed("grpc-listen") {
		l.v.Set("grpc_listen", s)
	}
	if b, _ := fs.GetBool("hot-reload"); fs.Changed("hot-reload") {
		l.v.Set("hot_reload", b)
	}
	if b, _ := fs.GetBool("use-rust-ffi"); fs.Changed("use-rust-ffi") {
		l.v.Set("use_rust_ffi", b)
	}

	var c Config
	if err := l.v.Unmarshal(&c); err != nil {
		return Config{}, err
	}
	c.ConfigFile = l.v.ConfigFileUsed()
	c.Security = MergeSecurityConfig(DefaultSecurity(), c.Security, l.v)
	applyExplicitSecurityEnv(&c)

	l.mu.Lock()
	l.cfg = c
	l.mu.Unlock()

	if c.HotReload && l.v.ConfigFileUsed() != "" {
		l.v.WatchConfig()
		l.v.OnConfigChange(func(_ fsnotify.Event) {
			var nc Config
			if err := l.v.Unmarshal(&nc); err != nil {
				return
			}
			nc.ConfigFile = l.v.ConfigFileUsed()
			nc.Security = MergeSecurityConfig(DefaultSecurity(), nc.Security, l.v)
			applyExplicitSecurityEnv(&nc)
			l.mu.Lock()
			l.cfg = nc
			cb := l.onChange
			l.mu.Unlock()
			if cb != nil {
				cb(nc)
			}
		})
	}

	return c, nil
}

// OnChange registers a hot-reload callback (e.g. recreate the engine pool).
func (l *Loader) OnChange(fn func(Config)) {
	l.mu.Lock()
	l.onChange = fn
	l.mu.Unlock()
}

func (l *Loader) Get() Config {
	l.mu.RLock()
	defer l.mu.RUnlock()
	return l.cfg
}

// Viper exposes the viper instance (tests or advanced merging).
func (l *Loader) Viper() *viper.Viper {
	return l.v
}

// MergeYAML merges a YAML fragment into viper (hot update, e.g. PUT /v1/config).
func (l *Loader) MergeYAML(data []byte) error {
	l.v.SetConfigType("yaml")
	if err := l.v.MergeConfig(bytes.NewReader(data)); err != nil {
		return err
	}
	var nc Config
	if err := l.v.Unmarshal(&nc); err != nil {
		return err
	}
	nc.ConfigFile = l.v.ConfigFileUsed()
	nc.Security = MergeSecurityConfig(DefaultSecurity(), nc.Security, l.v)
	applyExplicitSecurityEnv(&nc)
	l.mu.Lock()
	l.cfg = nc
	cb := l.onChange
	l.mu.Unlock()
	if cb != nil {
		cb(nc)
	}
	return nil
}

func (l *Loader) mergeSecurityYAMLFiles() {
	for _, dir := range []string{".", "/etc/aegis"} {
		p := filepath.Join(dir, "security.yaml")
		b, err := os.ReadFile(p)
		if err != nil {
			continue
		}
		l.v.SetConfigType("yaml")
		_ = l.v.MergeConfig(bytes.NewReader(b))
		return
	}
}

// applyExplicitSecurityEnv forces security.development.disable_auth from the OS.
// Flat Viper + security.yaml (development.* at root) does not always populate
// security.development.disable_auth for AEGIS_SECURITY_DEVELOPMENT_DISABLE_AUTH.
func applyExplicitSecurityEnv(c *Config) {
	s := strings.TrimSpace(os.Getenv("AEGIS_SECURITY_DEVELOPMENT_DISABLE_AUTH"))
	if s == "" {
		return
	}
	if b, err := strconv.ParseBool(s); err == nil {
		c.Security.Development.DisableAuth = b
		return
	}
	if strings.EqualFold(s, "yes") || s == "1" {
		c.Security.Development.DisableAuth = true
	}
}
