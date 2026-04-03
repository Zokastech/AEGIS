#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Invoked by hyperfine — évite les problèmes de quoting.
exec "${AEGIS_BIN:?}" analyze "Email alice@bench.test tel +33 6 11 22 33 44 card 4532015112830366 ip 192.168.1.1 url https://zokastech.fr/x"
