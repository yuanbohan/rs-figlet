#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE_DIR="${ROOT_DIR}/tests/fixtures"
TOILET_BIN="${TOILET_BIN:-toilet}"

mkdir -p "${FIXTURE_DIR}"

generate_fixture() {
    local font="$1"
    local prefix="$2"

    "${TOILET_BIN}" -d "${ROOT_DIR}/resources" -f "${font}" Test \
        > "${FIXTURE_DIR}/${prefix}_test.txt"
    "${TOILET_BIN}" -d "${ROOT_DIR}/resources" -f "${font}" FIGlet \
        > "${FIXTURE_DIR}/${prefix}_figlet.txt"
    "${TOILET_BIN}" -d "${ROOT_DIR}/resources" -f "${font}" -- -4.5 \
        > "${FIXTURE_DIR}/${prefix}_negative_float.txt"
    "${TOILET_BIN}" -d "${ROOT_DIR}/resources" -f "${font}" "Hello Rust" \
        > "${FIXTURE_DIR}/${prefix}_hello_rust.txt"
}

generate_fixture "smblock.tlf" "toilet_smblock"
generate_fixture "future.tlf" "toilet_future"
generate_fixture "wideterm.tlf" "toilet_wideterm"
generate_fixture "mono12.tlf" "toilet_mono12"
generate_fixture "mono9.tlf" "toilet_mono9"
