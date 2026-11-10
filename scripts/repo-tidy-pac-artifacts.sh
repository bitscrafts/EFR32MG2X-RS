#!/usr/bin/env bash
# Move build artefacts and one-off scripts out of the PAC crate.
#
# The repository must contain sources, tests and docs -- not hex dumps, vendor
# packs and ad-hoc shell one-liners. Everything this script moves lands in a
# staging directory OUTSIDE the repository (per the workspace rule that
# temporary material lives under /projects/_tmp/), so nothing is lost: the files
# are relocated, never deleted.
#
# Kept in the crate on purpose:
#   src/            PAC sources (generated, but they are the crate)
#   examples/       the examples
#   docs/           crate documentation
#   svd/            the SVD inputs the PAC was generated from (provenance)
#   interface/      the OpenOCD interface file the flashing recipe references
#   Cargo.toml, build.rs, memory.x, memory.x.bootloader, memory_direct.x,
#   device.x, README.md, COMPARISON_RESULTS.md
#
# Usage: scripts/repo-tidy-pac-artifacts.sh [--confirm] [dest]
#   without --confirm it only prints what it would move.
set -euo pipefail

CONFIRM=0
DEST=""
for arg in "$@"; do
  case "${arg}" in
    --confirm) CONFIRM=1 ;;
    *) DEST="${arg}" ;;
  esac
done
DEST="${DEST:-/projects/_tmp/efr32mg24-rs/pac-artifacts}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PAC="${REPO_ROOT}/crates/efr32mg24-pac"
cd "${PAC}" || { echo "REFUSE: ${PAC} not found"; exit 1; }

# Globs of what leaves the crate. Directories are handled separately.
FILE_PATTERNS=(
  "*.hex" "*.bin" "*.pdf" "*.pack"
  "check_*.sh" "flash_*.sh" "patch_*.sh" "patch_*.py"
  "debug_session.sh" "test_direct_flash.sh" "start-page.py"
)
DIRS=("test_blink_c" "src_backup_a020")
EXTRA_FILES=("blink_unlock_cmu")

echo "staging destination: ${DEST}"
echo

moved=0
plan=()
for pat in "${FILE_PATTERNS[@]}"; do
  for f in ${pat}; do
    [ -e "${f}" ] && plan+=("${f}")
  done
done
for d in "${DIRS[@]}"; do
  [ -d "${d}" ] && plan+=("${d}/")
done
for f in "${EXTRA_FILES[@]}"; do
  [ -e "${f}" ] && plan+=("${f}")
done

if [ ${#plan[@]} -eq 0 ]; then
  echo "nothing to do: the crate is already clean"
  exit 0
fi

for item in "${plan[@]}"; do
  printf '  %-45s %s\n' "${item}" "$(du -sh "${item%/}" 2>/dev/null | cut -f1)"
done

if [ "${CONFIRM}" -ne 1 ]; then
  echo
  echo "DRY RUN: ${#plan[@]} items. Re-run with --confirm to move them out of the repository."
  exit 0
fi

mkdir -p "${DEST}"
for item in "${plan[@]}"; do
  src="${item%/}"
  if [ -e "${DEST}/$(basename "${src}")" ]; then
    echo "SKIP (already staged): $(basename "${src}")"
    continue
  fi
  mv "${src}" "${DEST}/"
  moved=$((moved + 1))
done

echo
echo "moved ${moved} items to ${DEST}"
echo "the repository now records their removal; the files themselves still exist there"
