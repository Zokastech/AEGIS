// AEGIS — zokastech.fr — Apache 2.0 / MIT

//go:build aegisffi && cgo

package bridge

/*
#cgo CFLAGS: -I${SRCDIR}/../../crates/aegis-ffi/include
#cgo darwin LDFLAGS: -L${SRCDIR}/../../target/release -laegis_ffi
#cgo linux LDFLAGS: -L${SRCDIR}/../../target/release -laegis_ffi
#include <stdlib.h>
#include "aegis.h"
*/
import "C"

import (
	"context"
	"encoding/json"
	"errors"
	"sync"
	"unsafe"
)

// RustEngine wraps an aegis-ffi handle (one mutex per handle — no internal Rust concurrency).
type RustEngine struct {
	mu sync.Mutex
	h  *C.AegisHandle
}

// NewRustEngine loads the shared library; empty configJSON → engine defaults.
func NewRustEngine(configJSON string) (*RustEngine, error) {
	var ccfg *C.char
	if configJSON != "" {
		ccfg = C.CString(configJSON)
		defer C.free(unsafe.Pointer(ccfg))
	}
	h := C.aegis_init(ccfg)
	if h == nil {
		return nil, errors.New(C.GoString(C.aegis_last_error()))
	}
	return &RustEngine{h: h}, nil
}

func (r *RustEngine) Close() {
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.h != nil {
		C.aegis_free(r.h)
		r.h = nil
	}
}

func (r *RustEngine) Analyze(_ context.Context, text, analysisConfigJSON string) (string, error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	ct := C.CString(text)
	defer C.free(unsafe.Pointer(ct))
	var ccfg *C.char
	if analysisConfigJSON != "" {
		ccfg = C.CString(analysisConfigJSON)
		defer C.free(unsafe.Pointer(ccfg))
	}
	out := C.aegis_analyze(r.h, ct, ccfg)
	if out == nil {
		return "", errors.New(C.GoString(C.aegis_last_error()))
	}
	defer C.aegis_free_string(out)
	return C.GoString(out), nil
}

func (r *RustEngine) AnalyzeBatch(_ context.Context, texts []string) (string, error) {
	payload, err := json.Marshal(texts)
	if err != nil {
		return "", err
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	cj := C.CString(string(payload))
	defer C.free(unsafe.Pointer(cj))
	out := C.aegis_analyze_batch(r.h, cj)
	if out == nil {
		return "", errors.New(C.GoString(C.aegis_last_error()))
	}
	defer C.aegis_free_string(out)
	return C.GoString(out), nil
}

func (r *RustEngine) Anonymize(_ context.Context, text, configJSON string) (string, error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	ct := C.CString(text)
	defer C.free(unsafe.Pointer(ct))
	var ccfg *C.char
	if configJSON != "" {
		ccfg = C.CString(configJSON)
		defer C.free(unsafe.Pointer(ccfg))
	}
	out := C.aegis_anonymize(r.h, ct, ccfg)
	if out == nil {
		return "", errors.New(C.GoString(C.aegis_last_error()))
	}
	defer C.aegis_free_string(out)
	return C.GoString(out), nil
}

func (r *RustEngine) Deanonymize(context.Context, string) (string, error) {
	return "", ErrNotImplemented
}

func (r *RustEngine) LastError() string {
	return C.GoString(C.aegis_last_error())
}

func (r *RustEngine) Version() string {
	return C.GoString(C.aegis_version())
}
