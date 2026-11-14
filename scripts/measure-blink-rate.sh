#!/usr/bin/env bash
# Measure the LED toggle rate of the running image, over SWD, with no eyes.
#
# How: blink examples increment a counter (`seq`) in the mailbox on every
# toggle. Read it twice with a wall-clock gap on the host and divide:
#
#     toggle_rate = (seq1 - seq0) / (t1 - t0)          [toggles per second]
#     blink_hz    = toggle_rate / 2                     [full ON+OFF cycles]
#
# The counter runs on the chip, so the sampling interval does not have to be
# short: the only host-side quantity is elapsed time, which is measured with
# `date +%s.%N`.
#
# READ ONLY. Nothing is written to the target.
#
# Usage: scripts/measure-blink-rate.sh [seconds] [host] [user]
set -uo pipefail

SECONDS_TO_MEASURE="${1:-8}"
HOST="${2:-${FLASH_HOST:-localhost}}"
USER_NAME="${3:-${FLASH_USER:-$(id -un)}}"
CFG=/tmp/openocd-mg24-mini.cfg

read_seq() {
  local out
  out="$(ssh -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
    "sudo -n openocd -f interface/cmsis-dap.cfg -f ${CFG} \
       -c 'init; halt' -c 'mdw 0x2003F000 2' -c 'resume; shutdown'" 2>&1)"
  printf '%s\n' "${out}" | sed -n 's/^0x2003f000: 4d424f58 \([0-9a-f]*\).*/\1/p' | head -1
}

seq0_hex="$(read_seq)"
t0=$(date +%s.%N)
if [ -z "${seq0_hex}" ]; then
  echo "no mailbox — is a blink example with the counter running?" >&2
  exit 2
fi

sleep "${SECONDS_TO_MEASURE}"

seq1_hex="$(read_seq)"
t1=$(date +%s.%N)
if [ -z "${seq1_hex}" ]; then
  echo "second read failed" >&2
  exit 2
fi

SEQ0=$((16#${seq0_hex}))
SEQ1=$((16#${seq1_hex}))
SEQ0="${SEQ0}" SEQ1="${SEQ1}" T0="${t0}" T1="${t1}" python3 -c '
import os
s0 = int(os.environ["SEQ0"]); s1 = int(os.environ["SEQ1"])
t0 = float(os.environ["T0"]); t1 = float(os.environ["T1"])
d = (s1 - s0) % (2**32)
elapsed = t1 - t0
if elapsed <= 0 or d == 0:
    print(f"no progress: seq {s0} -> {s1} in {elapsed:.2f}s")
    raise SystemExit(1)
rate = d / elapsed
print(f"seq {s0} -> {s1}  ({d} toggles in {elapsed:.2f}s)")
print(f"toggle_rate = {rate:.2f} Hz")
print(f"blink_hz    = {rate/2:.2f}  (one ON + one OFF)")
'
