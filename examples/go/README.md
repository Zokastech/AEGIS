# AEGIS — zokastech.fr — Apache 2.0 / MIT

## Go + CGO (libaegis_ffi)

Ce programme charge le moteur via la **bibliothèque C** exposée par le crate `aegis-ffi`.

### 1. Compiler la bibliothèque Rust

Depuis la **racine du dépôt** :

```bash
cargo build -p aegis-ffi --release
```

Fichiers produits :

- Linux : `target/release/libaegis_ffi.so`
- macOS : `target/release/libaegis_ffi.dylib`
- Windows : `target/release/aegis_ffi.dll`

### 2. Variables d’environnement pour le linker

**Linux / macOS** (adapter le chemin absolu vers votre clone) :

```bash
export CGO_CFLAGS="-I$(pwd)/crates/aegis-ffi/include"
export CGO_LDFLAGS="-L$(pwd)/target/release -laegis_ffi"
# macOS au runtime :
export DYLD_LIBRARY_PATH="$(pwd)/target/release:${DYLD_LIBRARY_PATH}"
```

**Linux** au runtime :

```bash
export LD_LIBRARY_PATH="$(pwd)/target/release:${LD_LIBRARY_PATH}"
```

### 3. Compiler et exécuter l’exemple

```bash
cd examples/go
go build -o aegis-cgo-demo .
./aegis-cgo-demo
```

### Dépannage

- `undefined reference to aegis_init` : vérifiez `CGO_LDFLAGS` et que `libaegis_ffi` est bien dans `target/release`.
- macOS refuse de charger la dylib : chemins avec `install_name_tool` ou `DYLD_LIBRARY_PATH` comme ci-dessus.
