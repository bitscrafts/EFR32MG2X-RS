#!/usr/bin/env bash
# Invalidate the diagnostic mailbox before running a new image.
#
#   scripts/clear-mailbox.sh [host] [user]
#
# The mailbox lives in RAM at 0x2003_F000 and survives a reset. Examples now invalidate it themselves in the first
# instruction of `main`, so this script is a belt-and-braces tool for the cases where the image about to run cannot do it
# (a bare-metal program without the shared module, or a target left running by someone else).
#
# It used to swallow openocd's failure and report success while the record was still in RAM, so every caller believed the
# slot was empty when it was not. The exit status is checked now, and the word "Error" in openocd's output is treated as
# a failure too, because openocd can print an error and still exit zero.
#
# Writes one word inside this project's own diagnostic area; nothing else is touched.
set -euo pipefail

HOST="${1:-${FLASH_HOST:-localhost}}"
USER_NAME="${2:-${FLASH_USER:-$(id -un)}}"
CFG=/tmp/openocd-mg24-mini.cfg
MAILBOX=0x2003F000

if ! out=$(ssh -n -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
  "sudo -n openocd -f interface/cmsis-dap.cfg -f ${CFG} \
     -c 'init; halt' -c 'mww ${MAILBOX} 0' -c 'resume; shutdown'" 2>&1); then
  printf 'clear-mailbox: openocd failed\n%s\n' "$out" >&2
  exit 1
fi

case "$out" in
  *Error*|*error*)
    printf 'clear-mailbox: openocd reported an error\n%s\n' "$out" >&2
    exit 1
    ;;
esac
