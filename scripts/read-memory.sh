#!/usr/bin/env bash
# Read a range of target memory over SWD, read-only.
#
#   scripts/read-memory.sh 0x20001A70 32 [host] [user]
#
# Halts the core only for the duration of the dump and resumes it afterwards -- the same mechanism
# scripts/read-mailbox.sh uses, generalised because the interesting addresses are not always the mailbox.
set -uo pipefail

ADDR="${1:?usage: read-memory.sh <addr> <word-count> [host] [user]}"
WORDS="${2:?usage: read-memory.sh <addr> <word-count> [host] [user]}"
HOST="${3:-10.10.10.217}"
USER_NAME="${4:-mvcorrea}"
CFG=/tmp/openocd-mg24-mini.cfg

ssh -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
  "sudo -n openocd -f interface/cmsis-dap.cfg -f ${CFG} \
     -c 'init; halt' -c 'mdw ${ADDR} ${WORDS}' -c 'resume; shutdown'" 2>&1 |
  sed -n 's/^\(0x[0-9a-f]*\): \(.*\)/\1: \2/p'
