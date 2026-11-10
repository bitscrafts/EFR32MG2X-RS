#!/usr/bin/env bash
# Dump the XIAO MG24's whole 1.5 MiB flash to a file on lnx, in chunks.
#
# A single 1.5 MiB probe-rs read times out ("CMSIS-DAP command Disconnect:
# Timeout in USB communication") because one CMSIS-DAP transfer of that size
# exceeds the probe's buffer. 16 KiB chunks with a per-chunk retry work.
#
# READ ONLY: this script never writes to the target.
#
# Usage (run ON lnx):  bash dump-flash-lnx.sh <out.bin> [chunk_bytes] [base] [size]
set -uo pipefail

OUT="${1:?usage: dump-flash-lnx.sh <out.bin> [chunk] [base] [size]}"
CHUNK="${2:-16384}"
BASE="${3:-0x08000000}"
SIZE="${4:-1572864}"
CHIP="EFR32MG24B220F1536IM48"

TMP="$(mktemp -d)" || exit 1
trap 'rm -rf "$TMP"' EXIT

: > "${OUT}"
total_written=0
addr=$((BASE))
end=$((BASE + SIZE))

echo "dumping ${SIZE} bytes from 0x$(printf '%08X' "$addr") in ${CHUNK}-byte chunks -> ${OUT}"
while [ "$addr" -lt "$end" ]; do
  n=${CHUNK}
  [ $((addr + n)) -gt "$end" ] && n=$((end - addr))

  ok=0
  for attempt in 1 2 3; do
    if sudo -n probe-rs read b8 "$(printf '0x%08X' "$addr")" "$n" --output "${TMP}/chunk.bin" >"${TMP}/log" 2>&1; then
      got=$(stat -c %s "${TMP}/chunk.bin" 2>/dev/null || echo 0)
      if [ "$got" -eq "$n" ]; then ok=1; break; fi
    fi
    echo "  retry at 0x$(printf '%08X' "$addr") (attempt ${attempt}): $(tail -1 "${TMP}/log")" >&2
    sleep 1
  done

  if [ "$ok" -ne 1 ]; then
    echo "FAILED at 0x$(printf '%08X' "$addr") after 3 attempts; partial file has ${total_written} bytes" >&2
    exit 2
  fi

  cat "${TMP}/chunk.bin" >> "${OUT}"
  total_written=$((total_written + n))
  addr=$((addr + n))
  printf '\r  %d / %d bytes' "$total_written" "$SIZE"
done

printf '\n'
echo "wrote ${total_written} bytes to ${OUT}"
sha256sum "${OUT}"
