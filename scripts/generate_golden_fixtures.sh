#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE_DIR="${ROOT_DIR}/tests/fixtures"
FIGLET_BIN="${FIGLET_BIN:-figlet}"

mkdir -p "${FIXTURE_DIR}"

generate_fixture() {
    local prefix="$1"
    shift

    "${FIGLET_BIN}" "$@" Test > "${FIXTURE_DIR}/${prefix}_test.txt"
    "${FIGLET_BIN}" "$@" FIGlet > "${FIXTURE_DIR}/${prefix}_figlet.txt"
    "${FIGLET_BIN}" "$@" -- -4.5 > "${FIXTURE_DIR}/${prefix}_negative_float.txt"
    "${FIGLET_BIN}" "$@" "Hello Rust" > "${FIXTURE_DIR}/${prefix}_hello_rust.txt"
}

generate_fixture "standard"
generate_fixture "small" -f "${ROOT_DIR}/resources/small.flf"
generate_fixture "big" -f "${ROOT_DIR}/resources/big.flf"
generate_fixture "slant" -f "${ROOT_DIR}/resources/slant.flf"
