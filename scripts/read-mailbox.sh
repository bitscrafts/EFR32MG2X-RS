#!/usr/bin/env bash
# Read the diagnostic mailbox (0x2003_F000) from the XIAO MG24 over SWD.
#
# READ ONLY. Nothing is written to the target; the core is halted only for the
# duration of the read.
#
# Uses OpenOCD rather than probe-rs: probe-rs 0.32 can wedge its own CMSIS-DAP
# session (see docs/01-getting-started/FLASHING.md), while OpenOCD keeps
# working against the same probe.
#
# TORN RECORDS: the halt can land in the middle of a write. Images built after
# 2026-11-10 clear the magic word first and publish it last, so a torn record
# reads as magic=0 and this script simply retries. Older images (including the
# one on the board before the next flash) can return a record whose `seq` and
# message disagree -- that is the tearing, not a firmware bug.
#
# Usage: scripts/read-mailbox.sh [host] [user]
set -uo pipefail

HOST="${1:-${FLASH_HOST:-localhost}}"
USER_NAME="${2:-${FLASH_USER:-$(id -un)}}"
ADDR=0x2003F000
WORDS=72          # 288 bytes: 16 of header + 272 of message. 64 truncated the record exactly where the
                  # fields that explain a failure live, which twice produced "no data" where data existed.
CFG=/tmp/openocd-mg24-mini.cfg
ATTEMPTS=5

read_words() {
  local out
  out="$(ssh -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
    "sudo -n openocd -f interface/cmsis-dap.cfg -f ${CFG} \
       -c 'init; halt' -c 'mdw ${ADDR} ${WORDS}' -c 'resume; shutdown'" 2>&1)"
  # OpenOCD prints 8 words per line; take every line of the dump, in order.
  printf '%s\n' "${out}" | sed -n 's/^0x2003f[0-9a-f][0-9a-f][0-9a-f]: \(.*\)/\1/p' | tr '\n' ' ' | sed 's/[[:space:]]*$//' 
}

for attempt in $(seq 1 "${ATTEMPTS}"); do
  WORDS_TEXT="$(read_words)"
  if [ -z "${WORDS_TEXT}" ]; then
    echo "attempt ${attempt}: read failed" >&2
    sleep 0.3
    continue
  fi

  parsed="$(WORDS_TEXT="${WORDS_TEXT}" python3 -c '
import os, struct, sys

words = [int(w, 16) for w in os.environ["WORDS_TEXT"].split()]
data = b"".join(struct.pack("<I", w) for w in words)
magic, seq, status, length = struct.unpack_from("<IIII", data, 0)
if magic != 0x4D424F58:
    # 0 = a write is in progress; anything else = no mailbox at all.
    sys.exit(5 if magic == 0 else 4)
length = min(length, len(data) - 16, 200)
names = {0: "info", 1: "PASS", 2: "FAIL"}
print(f"seq={seq} status={names.get(status, status)} len={length}")
print(data[16:16 + length].decode("utf-8", "replace"))
')"
  rc=$?
  if [ "${rc}" -eq 0 ]; then
    printf '%s\n' "${parsed}"
    exit 0
  fi
  if [ "${rc}" -eq 4 ]; then
    echo "no mailbox record (magic mismatch) — the image on the board may predate the mailbox" >&2
    exit 4
  fi
  sleep 0.2
done

echo "record still being written after ${ATTEMPTS} attempts (core halts mid-write)" >&2
exit 5
