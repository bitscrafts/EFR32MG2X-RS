#!/usr/bin/env bash
# Flash a firmware image to the XIAO MG24 through the host the probe is attached to.
#
# The programmer is the onboard SAMD11 CMSIS-DAP bridge (USB 2886:0062). Builds happen wherever you
# like; flashing happens on the machine the probe is plugged into, reached over SSH.
#
# WHY THIS SCRIPT EXISTS
# The Gecko bootloader lives at 0x08000000-0x08005FFF. An image linked at
# 0x08000000 overwrites it and the board loses its serial-upload path. On
# 2026-06-29 a crashing clock configuration also locked the debug Access Port
# and the board had to be recovered externally (docs/01-getting-started/
# BRICKING_INCIDENT_2026-06-29.md). Flashing therefore goes through the guards
# below and never runs implicitly.
#
# Usage:
#   scripts/flash.sh <elf> --confirm [--run]
#
#   --confirm   required; without it the script only prints what it would do
#   --run       reset and run the image after a successful verify (default:
#               leave the core halted in reset so the operator decides)
#
# Guards, all fail-safe:
#   1. --confirm present                otherwise dry-run
#   2. ELF parses, 32-bit little-endian otherwise refuse
#   3. load_min >= 0x08006000           otherwise refuse (bootloader protected)
#   4. load_max <= 0x0817FFFF           otherwise refuse (past end of flash)
#   5. exactly one CMSIS-DAP probe visible on the target host, otherwise refuse
#   6. probe-rs verify after write      read-back compare, not a self-report
set -euo pipefail

BOOTLOADER_END=$((0x08006000))          # first byte available to an app
FLASH_END=$((0x08180000))               # 0x08006000 + 1512 KiB
CHIP="EFR32MG24B220F1536IM48"
# Where the board is. The script always drives the programmer over SSH, because in this project the
# board hangs off a different machine from the one that builds — but nothing about the guard depends
# on that: set FLASH_HOST to the machine the probe is plugged into (or to `localhost`) and the same
# checks, the same dry run and the same read-back verification apply.
#
# Keeping the address out of the script is also what lets this repository be published: a hostname
# and a user in a public file is an invitation, and the value belongs in the environment.
FLASH_HOST="${FLASH_HOST:-localhost}"
FLASH_USER="${FLASH_USER:-$(id -un)}"
REMOTE_TMP="/tmp/efr32mg24-flash"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() { sed -n '9,26p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }

ELF=""
CONFIRM=0
RUN=0
for arg in "$@"; do
  case "${arg}" in
    --confirm) CONFIRM=1 ;;
    --run) RUN=1 ;;
    -h|--help) usage; exit 0 ;;
    *) ELF="${arg}" ;;
  esac
done

if [ -z "${ELF}" ]; then
  echo "REFUSE: no ELF given" >&2
  usage >&2
  exit 64
fi
if [ ! -f "${ELF}" ]; then
  echo "REFUSE: ${ELF} does not exist" >&2
  exit 66
fi

echo "== image =="
RANGE_LINE="$(python3 "${REPO_ROOT}/scripts/elf-load-range.py" "${ELF}")" || {
  echo "REFUSE: cannot parse ${ELF} as a 32-bit little-endian ELF" >&2
  exit 65
}
echo "${RANGE_LINE}"

LOAD_MIN=$(printf '%s\n' "${RANGE_LINE}" | sed -n 's/.*load_min=0x\([0-9A-F]*\).*/\1/p')
LOAD_MAX=$(printf '%s\n' "${RANGE_LINE}" | sed -n 's/.*load_max=0x\([0-9A-F]*\).*/\1/p')
LOAD_MIN=$((16#${LOAD_MIN}))
LOAD_MAX=$((16#${LOAD_MAX}))

if [ "${LOAD_MIN}" -lt "${BOOTLOADER_END}" ]; then
  printf 'REFUSE: load_min 0x%08X is inside the bootloader region (0x08000000-0x08005FFF)\n' "${LOAD_MIN}" >&2
  exit 3
fi
if [ "${LOAD_MAX}" -ge "${FLASH_END}" ]; then
  printf 'REFUSE: load_max 0x%08X is past the end of flash (0x%08X)\n' "${LOAD_MAX}" "${FLASH_END}" >&2
  exit 4
fi

echo "== probe (${FLASH_HOST}) =="
PROBES="$(ssh -o BatchMode=yes -o ConnectTimeout=10 "${FLASH_USER}@${FLASH_HOST}" \
  "sudo -n probe-rs list 2>/dev/null | grep -c 'CMSIS-DAP'" || true)"
if [ "${PROBES:-0}" -ne 1 ]; then
  echo "REFUSE: expected exactly 1 CMSIS-DAP probe on ${FLASH_HOST}, found ${PROBES:-0}" >&2
  exit 5
fi
echo "1 CMSIS-DAP probe present"

if [ "${CONFIRM}" -ne 1 ]; then
  echo
  echo "DRY RUN: guards passed. Re-run with --confirm to write the image."
  exit 0
fi

echo "== write =="
ssh "${FLASH_USER}@${FLASH_HOST}" "mkdir -p ${REMOTE_TMP}"
scp -q "${ELF}" "${FLASH_USER}@${FLASH_HOST}:${REMOTE_TMP}/image.elf"
ssh "${FLASH_USER}@${FLASH_HOST}" "sudo -n probe-rs download --chip ${CHIP} ${REMOTE_TMP}/image.elf"

echo "== verify (read-back) =="
ssh "${FLASH_USER}@${FLASH_HOST}" "sudo -n probe-rs verify --chip ${CHIP} ${REMOTE_TMP}/image.elf"

if [ "${RUN}" -eq 1 ]; then
  echo "== reset + run =="
  ssh "${FLASH_USER}@${FLASH_HOST}" "sudo -n probe-rs reset --chip ${CHIP}"
else
  echo "image written and verified; core left in reset (use --run to start it)"
fi
