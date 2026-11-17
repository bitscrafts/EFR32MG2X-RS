#!/bin/sh
# Turn on notifications for the dosimeter's RMS characteristic and count what arrives.
#
# The characteristic is declared at handle 0x0002 with its value at 0x0003 (read from `gatttool --characteristics`),
# so its Client Characteristic Configuration descriptor is the next handle, 0x0004. Writing 0x0100 enables
# notifications; 0x0000 disables them.
#
# This is the proof the phone will repeat: the service is discoverable, the descriptor is writable and the samples
# arrive at the advertised rate. The count is compared against the firmware's own `published=` in the same run.
#
# Usage: gatt-subscribe.sh [address] [seconds]
set -u
ADDR="${1:-E4:0A:75:13:3E:E0}"
SECONDS_TO_LISTEN="${2:-55}"
CCCD=0x0004
VALUE_HANDLE=0x0003

OUT="${TMPDIR:-/tmp}/notify-proof.txt"
# `gatttool` defaults to `--sec-level=low`, which never asks the link to encrypt, so notifications that arrive prove the
# GATT server works and nothing about the bond. SEC=medium makes the client request encryption (and fail without a bond),
# which is the check that a reconnect after a reset really used the stored key. Default stays `low` for the unpaired test.
SEC="${SEC:-low}"
sudo -n timeout $((SECONDS_TO_LISTEN + 10)) gatttool -b "${ADDR}" --sec-level="${SEC}" \
  --char-write-req -a "${CCCD}" -n 0100 --listen > "${OUT}" 2>&1 || true

TOTAL=$(grep -c "Notification handle = ${VALUE_HANDLE}" "${OUT}" || true)
UNIQUE=$(grep "Notification handle = ${VALUE_HANDLE}" "${OUT}" | awk '{print $NF}' | sort -u | wc -l)
echo "subscricao escrita em 0x0004 = 0100"
echo "window de escuta: ${SECONDS_TO_LISTEN} s"
echo "notificacoes em 0x0003: ${TOTAL}  (valores distintos: ${UNIQUE})"
echo "primeiras tres:"
grep "Notification handle = ${VALUE_HANDLE}" "${OUT}" | head -3
echo "erros, se houver:"
grep -iE "error|failed|refused" "${OUT}" | head -3
