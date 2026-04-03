// AEGIS — zokastech.fr — Apache 2.0 / MIT

// Package domain holds the core business logic independent of HTTP/gRPC adapters.
// It has no dependency on api/, concrete bridge, Echo, etc.
package domain

import "errors"

var (
	// ErrEmptyText means input text is missing or whitespace-only.
	ErrEmptyText = errors.New("domain: texte requis")
)
