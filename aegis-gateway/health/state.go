// AEGIS — zokastech.fr — Apache 2.0 / MIT

package health

import (
	"sync/atomic"
)

var startupOK atomic.Bool

// MarkStartupOK marks startup complete (first successful engine contact).
func MarkStartupOK() {
	startupOK.Store(true)
}

// StartupComplete returns true after the first successful startup.
func StartupComplete() bool {
	return startupOK.Load()
}
