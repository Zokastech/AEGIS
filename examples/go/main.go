// AEGIS — zokastech.fr — Apache 2.0 / MIT
//
// Exemple minimal : initialiser le moteur AEGIS via CGO et analyser une chaîne synthétique.
//
// Compilation (depuis ce dossier) :
//
//	export CGO_CFLAGS="-I${REPO_ROOT}/crates/aegis-ffi/include"
//	export CGO_LDFLAGS="-L${REPO_ROOT}/target/release -laegis_ffi"
//	go build -o aegis-cgo-demo .
//
// Voir README.md pour LD_LIBRARY_PATH / DYLD_LIBRARY_PATH.

package main

/*
#cgo CFLAGS: -I../../crates/aegis-ffi/include
#cgo darwin LDFLAGS: -L../../target/release -laegis_ffi
#cgo linux LDFLAGS: -L../../target/release -laegis_ffi
#include <stdlib.h>
#include "aegis.h"
*/
import "C"

import (
	"fmt"
	"os"
	"unsafe"
)

func main() {
	// Config vide → recognizers regex en / fr par défaut (si aegis-regex lié au build FFI).
	h := C.aegis_init(nil)
	if h == nil {
		err := C.GoString(C.aegis_last_error())
		fmt.Fprintln(os.Stderr, "aegis_init failed:", err)
		os.Exit(1)
	}
	defer C.aegis_free(h)

	text := C.CString("Synthetic: contact@example.com or +33 6 00 00 00 00")
	defer C.free(unsafe.Pointer(text))

	out := C.aegis_analyze(h, text, nil)
	if out == nil {
		err := C.GoString(C.aegis_last_error())
		fmt.Fprintln(os.Stderr, "aegis_analyze failed:", err)
		os.Exit(1)
	}
	defer C.aegis_free_string(out)

	fmt.Println("aegis_version:", C.GoString(C.aegis_version()))
	fmt.Println("analyze JSON:", C.GoString(out))
}
