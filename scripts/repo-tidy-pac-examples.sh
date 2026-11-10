#!/usr/bin/env bash
# Retire the PAC examples that are not part of the hardware-verified ladder.
#
# The crate accumulated fourteen blink variants from the early bring-up
# experiments: raw-pointer versions with wrong addresses, CMU-poking versions,
# a broken pre-0.37 API version, and near-duplicates of each other. They are
# history, not tests: they are not in the verification ladder, they are not
# flashed, and several of them are known-wrong. Keeping them in examples/ makes
# `cargo build --examples` red (blink_pac) and the directory unreadable.
#
# KEEP LIST -- these are the ones exercised on hardware or about to be:
#
#   blink_xiao.rs          the 2026-06-29 hardware-verified PAC blink (PA7)
#   pac_blink_base.rs      NOP-loop baseline, flashed and measured (6.30 Hz)
#   pac_blink_double.rs    double-blink pattern, flashed, seen by the operator
#   pac_timer_blink.rs     TIMER0 1 Hz, flashed and measured (1.02 Hz)
#   pac_i2c_scan.rs        I2C0 scan, flashed and run (0 devices found)
#   pac_gpio_button.rs     Expansion Base button on PC01, compiled
#
# Everything else moves to a staging directory OUTSIDE the repository, so the
# files survive and remain readable as history.
#
# Usage: scripts/repo-tidy-pac-examples.sh [--confirm] [dest]
set -euo pipefail

CONFIRM=0
DEST=""
for arg in "$@"; do
  case "${arg}" in
    --confirm) CONFIRM=1 ;;
    *) DEST="${arg}" ;;
  esac
done
DEST="${DEST:-/projects/_tmp/efr32mg24-rs/pac-artifacts/examples-retired}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES="${REPO_ROOT}/crates/efr32mg24-pac/examples"
cd "${EXAMPLES}" || { echo "REFUSE: ${EXAMPLES} not found"; exit 1; }

KEEP=(
  blink_xiao
  pac_blink_base
  pac_blink_double
  pac_timer_blink
  pac_i2c_scan
  pac_gpio_button
)

is_kept() {
  local name="$1"
  for k in "${KEEP[@]}"; do
    [ "${name}" = "${k}" ] && return 0
  done
  return 1
}

retire=()
for f in *.rs; do
  name="${f%.rs}"
  is_kept "${name}" || retire+=("${f}")
done

echo "keeping: ${KEEP[*]}"
echo "plus:    common/blink.rs common/mailbox.rs"
echo
if [ ${#retire[@]} -eq 0 ]; then
  echo "nothing to retire"
  exit 0
fi

echo "would retire ${#retire[@]} examples:"
printf '  %s\n' "${retire[@]}"

if [ "${CONFIRM}" -ne 1 ]; then
  echo
  echo "DRY RUN: re-run with --confirm to move them out of the repository."
  exit 0
fi

mkdir -p "${DEST}"
moved=0
for f in "${retire[@]}"; do
  if [ -e "${DEST}/${f}" ]; then
    echo "SKIP (already staged): ${f}"
    continue
  fi
  mv "${f}" "${DEST}/"
  moved=$((moved + 1))
done

echo
echo "moved ${moved} files to ${DEST}"
