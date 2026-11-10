#!/usr/bin/env bash
# Build every example in efr32mg24-pac and report PASS/FAIL per example.
#
# Why a script: the build matrix is the "do the PAC examples still compile"
# check, and it must be reproducible byte-for-byte from a clean shell
# (same rustflags, same feature set, same output directory).
#
# Usage: scripts/build-pac-examples.sh [--features rt]
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FEATURES="${1:---features rt}"
LOG_DIR="${REPO_ROOT}/_tmp/build-logs"
mkdir -p "${LOG_DIR}"

cd "${REPO_ROOT}" || { echo "REFUSE: cannot cd to ${REPO_ROOT}"; exit 1; }

pass=0
fail=0
for src in crates/efr32mg24-pac/examples/*.rs; do
  ex="$(basename "${src}" .rs)"
  log="${LOG_DIR}/pac-${ex}.log"
  # shellcheck disable=SC2086
  if cargo build --release ${FEATURES} -p efr32mg24-pac --example "${ex}" >"${log}" 2>&1; then
    printf 'OK   %s\n' "${ex}"
    pass=$((pass + 1))
  else
    printf 'FAIL %s :: %s\n' "${ex}" "$(grep -m1 '^error' "${log}" || echo 'no error line')"
    fail=$((fail + 1))
  fi
done

printf '\n%s passed, %s failed (logs: %s)\n' "${pass}" "${fail}" "${LOG_DIR}"
[ "${fail}" -eq 0 ]
