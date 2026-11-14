#!/usr/bin/env bash
# Invalidate the diagnostic mailbox before running a new image.
#
#   scripts/clear-mailbox.sh [host] [user]
#
# The mailbox lives in RAM at 0x2003_F000 and **survives a reset**. An example that only writes it on an event can
# therefore be read as if it had reported, when what is there belongs to the previous image -- measured on 2026-11-14:
# after flashing `module_ble_fall`, the mailbox still held `ble_gatt`'s record, complete with the bytes a client had
# written minutes earlier. Clearing it first turns "stale" into "empty", and empty is obvious.
#
# Writes one word inside this project's own diagnostic area; nothing else is touched.
set -uo pipefail

HOST="${1:-10.10.10.217}"
USER_NAME="${2:-mvcorrea}"
CFG=/tmp/openocd-mg24-mini.cfg
MAILBOX=0x2003F000

ssh -n -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
  "sudo -n openocd -f interface/cmsis-dap.cfg -f ${CFG} \
     -c 'init; halt' -c 'mww ${MAILBOX} 0' -c 'resume; shutdown'" >/dev/null 2>&1
