// AEGIS — zokastech.fr — Apache 2.0 / MIT

//go:build aegisffi

package main

import (
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/config"
)

func newEngineFactory(cfg config.Config) func() (bridge.Engine, error) {
	return func() (bridge.Engine, error) {
		return bridge.NewRustEngine(cfg.EngineInitJSON)
	}
}
