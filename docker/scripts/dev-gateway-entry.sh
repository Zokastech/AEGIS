#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Point d’entrée du service docker-compose « aegis-gateway » : compile libaegis_ffi puis lance air (CGO + tag aegisffi).
# Ne pas exécuter dans le conteneur « aegis-core » (image rust seule, pas de Go) — voir docker-compose.dev.yml.
set -euo pipefail

if ! command -v go >/dev/null 2>&1; then
	echo "Erreur : la commande « go » est absente. « dev-gateway-entry.sh » est réservé au service Docker aegis-gateway (image golang + FFI)." >&2
	echo "  Le service aegis-core ne contient que Rust (cargo-watch) : pour compiler la lib ici, lancez : cargo build --release -p aegis-ffi" >&2
	echo "  Pour la passerelle : docker compose -f docker-compose.dev.yml up aegis-gateway   (ou make dev)" >&2
	exit 127
fi

cd /work
echo "==> [aegis-gateway dev] cargo build --release -p aegis-ffi"
cargo build --release -p aegis-ffi

# ort (load-dynamic) : il faut un chemin fichier réel lisible par le processus Go/air. Les symlinks copy-dylibs
# vers /root/.cache (ou caches cassés via bind-mount macOS) font échouer la résolution — on copie sur le volume /work.
WORK_ORT_DIR="/work/.aegis-dev"
WORK_ORT_SO="${WORK_ORT_DIR}/libonnxruntime.so"
# Aligné sur crates/ort-sys (ONNXRUNTIME_VERSION) — utilisé seulement si aucune lib issue du build cargo.
ORT_RELEASE_VER="${ORT_RELEASE_VER:-1.19.0}"

find_ort_lib_source() {
	local f=""
	if [[ -e /work/target/release/libonnxruntime.so ]]; then
		f="$(readlink -f /work/target/release/libonnxruntime.so 2>/dev/null || true)"
		if [[ -n "$f" && -f "$f" ]]; then
			printf '%s' "$f"
			return 0
		fi
	fi
	local _cache_root="${XDG_CACHE_HOME:-}"
	[[ -z "$_cache_root" && -n "${HOME:-}" ]] && _cache_root="${HOME}/.cache"
	local _base
	for _base in "${_cache_root}/ort.pyke.io" /root/.cache/ort.pyke.io "${HOME}/.cache/ort.pyke.io"; do
		[[ -n "$_base" && -d "$_base" ]] || continue
		f="$(find "$_base" -name libonnxruntime.so -type f 2>/dev/null | head -1 || true)"
		if [[ -n "$f" && -f "$f" ]]; then
			printf '%s' "$f"
			return 0
		fi
	done
	# Tout le cache root (noms de dossier variables selon versions ort)
	if [[ -d /root/.cache ]]; then
		f="$(find /root/.cache -name libonnxruntime.so -type f 2>/dev/null | head -1 || true)"
		if [[ -n "$f" && -f "$f" ]]; then
			printf '%s' "$f"
			return 0
		fi
	fi
	# Artefact build ort-sys (hors racine release)
	f="$(find /work/target/release/build -name libonnxruntime.so -type f 2>/dev/null | head -1 || true)"
	if [[ -n "$f" && -f "$f" ]]; then
		printf '%s' "$f"
		return 0
	fi
	f="$(find /work/target/release -maxdepth 3 -name libonnxruntime.so -type f 2>/dev/null | head -1 || true)"
	if [[ -n "$f" && -f "$f" ]]; then
		printf '%s' "$f"
		return 0
	fi
	return 1
}

copy_ort_libs_from_dir() {
	# $1 = répertoire source (ex. .../onnxruntime/lib ou tarball .../lib)
	local _from="$1"
	mkdir -p "$WORK_ORT_DIR"
	find "$_from" -maxdepth 1 -name 'libonnx*.so' \( -type f -o -type l \) -exec cp -fL {} "$WORK_ORT_DIR/" \;
	[[ -f "$WORK_ORT_SO" ]]
}

download_ort_prebuilt_linux() {
	local ver="$1"
	local arch slug tmp
	arch="$(uname -m)"
	case "$arch" in
	x86_64) slug="onnxruntime-linux-x64-${ver}" ;;
	aarch64 | arm64) slug="onnxruntime-linux-aarch64-${ver}" ;;
	*)
		echo "==> [aegis-gateway dev] ORT : architecture non gérée pour téléchargement : $arch" >&2
		return 1
		;;
	esac
	local url="https://github.com/microsoft/onnxruntime/releases/download/v${ver}/${slug}.tgz"
	tmp="/tmp/${slug}"
	rm -rf "$tmp"
	echo "==> [aegis-gateway dev] ORT : téléchargement ${url}"
	curl -fsSL "$url" | tar -xzf - -C /tmp
	if [[ ! -d "${tmp}/lib" ]]; then
		echo "==> [aegis-gateway dev] ORT : archive ${slug}.tgz sans répertoire lib/" >&2
		rm -rf "$tmp"
		return 1
	fi
	rm -f "$WORK_ORT_DIR"/libonnx*.so 2>/dev/null || true
	copy_ort_libs_from_dir "${tmp}/lib"
	local ok=$?
	rm -rf "$tmp"
	return "$ok"
}

ensure_ort_dylib_on_volume() {
	mkdir -p "$WORK_ORT_DIR"
	if [[ -n "${ORT_DYLIB_PATH:-}" && -f "${ORT_DYLIB_PATH}" ]]; then
		echo "==> [aegis-gateway dev] ORT_DYLIB_PATH déjà défini et fichier présent : ${ORT_DYLIB_PATH}"
		return 0
	fi
	if [[ -f "$WORK_ORT_SO" && -s "$WORK_ORT_SO" && -z "${FORCE_ORT_REDOWNLOAD:-}" ]]; then
		export ORT_DYLIB_PATH="$WORK_ORT_SO"
		echo "==> [aegis-gateway dev] ORT_DYLIB_PATH=${ORT_DYLIB_PATH} (copie persistée .aegis-dev)"
		return 0
	fi
	if [[ -n "${FORCE_ORT_REDOWNLOAD:-}" ]]; then
		rm -f "$WORK_ORT_DIR"/libonnx*.so 2>/dev/null || true
	fi
	local _src=""
	_src="$(find_ort_lib_source)" || true
	if [[ -n "$_src" ]]; then
		rm -f "$WORK_ORT_DIR"/libonnx*.so 2>/dev/null || true
		local _dir
		_dir="$(dirname "$_src")"
		if copy_ort_libs_from_dir "$_dir"; then
			export ORT_DYLIB_PATH="$WORK_ORT_SO"
			echo "==> [aegis-gateway dev] ORT : libonnx*.so copiés depuis ${_dir} → ${WORK_ORT_DIR}"
			return 0
		fi
		cp -fL "$_src" "$WORK_ORT_SO"
		export ORT_DYLIB_PATH="$WORK_ORT_SO"
		echo "==> [aegis-gateway dev] ORT : copié (fichier seul) depuis ${_src} → ${ORT_DYLIB_PATH}"
		return 0
	fi
	if download_ort_prebuilt_linux "$ORT_RELEASE_VER"; then
		export ORT_DYLIB_PATH="$WORK_ORT_SO"
		echo "==> [aegis-gateway dev] ORT_DYLIB_PATH=${ORT_DYLIB_PATH} (prébuild Microsoft v${ORT_RELEASE_VER})"
		return 0
	fi
	echo "==> [aegis-gateway dev] ERREUR : impossible de fournir libonnxruntime.so (NER ONNX plantera)." >&2
	return 1
}

ensure_ort_dylib_on_volume || true

export CGO_ENABLED=1
export LD_LIBRARY_PATH="/work/target/release:${WORK_ORT_DIR}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

cd /work/aegis-gateway
chmod +x ./scripts/air-build.sh 2>/dev/null || true
go mod tidy
exec air -c .air.toml
