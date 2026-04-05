// AEGIS — zokastech.fr — Apache 2.0 / MIT

//go:build !aegisffi

package main

import (
	"github.com/rs/zerolog/log"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/config"
)

func newEngineFactory(_ config.Config) func() (bridge.Engine, error) {
	log.Warn().Msg(
		"aegis-gateway: MockEngine (build sans -tags=aegisffi) — /v1/analyze ne lance pas le moteur Rust ; " +
			"IBR/NIR/téléphone/IBAN réels absents. Voir aegis-gateway/README.md et scripts/build-gateway-ffi.sh",
	)
	return func() (bridge.Engine, error) {
		return bridge.NewMockEngine(), nil
	}
}
