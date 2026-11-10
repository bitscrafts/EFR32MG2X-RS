#!/usr/bin/env bash
# Build every workspace crate once and print OK/FAIL per crate.
#
# The workspace does not build end to end as of 2026-11-10: this script exists
# to keep that fact visible instead of discovering it at dispatch time.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}" || { echo "REFUSE: cannot cd to ${REPO_ROOT}"; exit 1; }

# Logs live OUTSIDE the repository: the project root must stay clean.
LOG_DIR="${TMP_LOG_ROOT:-/projects/_tmp/efr32mg24-rs/build-logs}"
mkdir -p "${LOG_DIR}"

run() {
  local label="$1"; shift
  local log="${LOG_DIR}/crate-${label}.log"
  printf '%-40s ' "${label}"
  if cargo build --release "$@" >"${log}" 2>&1; then
    echo OK
  else
    echo "FAIL :: $(grep -m1 '^error' "${log}" || echo 'no error line')"
  fi
}

run pac            -p efr32mg24-pac
run hal            -p efr32mg24-hal --features rt
run rail-sys       -p efr32mg24-rail-sys
run bluetooth-sys  -p efr32mg24-bluetooth-sys
run bsp            -p xiao-mg24-bsp
run bsp-all-targets -p xiao-mg24-bsp --features ble,microphone --examples
