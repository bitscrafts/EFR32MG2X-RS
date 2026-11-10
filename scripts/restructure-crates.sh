#!/usr/bin/env bash
# Restructure the repository into a crates/ workspace layout.
#
# Every crate directory moves one level down. All intra-workspace path
# dependencies are sibling references ("../efr32mg24-pac"), so moving all
# crates together keeps every path valid with no Cargo.toml edits.
#
# Idempotent: refuses to run if crates/ already holds a directory it would move.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

CRATES=(
  efr32mg24-pac
  efr32mg24-hal
  efr32mg24-rail-sys
  efr32mg24-bluetooth-sys
  xiao-mg24-bsp
  efr32mg24-bootloader
  bootloader-test-app
)

mkdir -p crates

for c in "${CRATES[@]}"; do
  if [ -e "crates/${c}" ]; then
    echo "REFUSE: crates/${c} already exists"
    exit 1
  fi
done

for c in "${CRATES[@]}"; do
  if [ -d "${c}" ]; then
    git mv "${c}" "crates/${c}"
    echo "moved ${c} -> crates/${c}"
  else
    echo "skip ${c} (not present at top level)"
  fi
done

echo "done"
