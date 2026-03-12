#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE_DIR="${ROOT_DIR}/tests/fixtures"
FIGLET_BIN="${FIGLET_BIN:-figlet}"

mkdir -p "${FIXTURE_DIR}"

"${FIGLET_BIN}" Test > "${FIXTURE_DIR}/standard_test.txt"
"${FIGLET_BIN}" FIGlet > "${FIXTURE_DIR}/standard_figlet.txt"
"${FIGLET_BIN}" -- -4.5 > "${FIXTURE_DIR}/standard_negative_float.txt"
"${FIGLET_BIN}" -f "${ROOT_DIR}/resources/small.flf" Test > "${FIXTURE_DIR}/small_test.txt"
"${FIGLET_BIN}" -f "${ROOT_DIR}/resources/small.flf" -- -4.5 > "${FIXTURE_DIR}/small_negative_float.txt"
