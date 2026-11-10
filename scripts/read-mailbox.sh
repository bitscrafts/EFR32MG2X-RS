#!/usr/bin/env bash
# Read the diagnostic mailbox (0x2003_F000) from the XIAO MG24 over SWD.
#
# READ ONLY. Nothing is written to the target; the core is halted only for the
# duration of the read.
#
# Usage: scripts/read-mailbox.sh [host] [user]
set -uo pipefail

HOST="${1:-10.10.10.217}"
USER_NAME="${2:-mvcorrea}"
ADDR=0x2003F000
LEN=216
CHIP=EFR32MG24B220F1536IM48

OUT="$(ssh -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
  "sudo -n probe-rs read b8 ${ADDR} ${LEN}" 2>&1)"

if ! printf '%s' "${OUT}" | grep -q -E '([0-9a-f]{2} ){4}'; then
  echo "read failed:" >&2
  printf '%s\n' "${OUT}" | tail -5 >&2
  exit 2
fi

MAILBOX_TEXT="${OUT}" python3 -c '
import os, re, struct, sys

text = os.environ["MAILBOX_TEXT"]
tokens = re.findall(r"(?<![0-9a-fA-F])[0-9a-fA-F]{2}(?![0-9a-fA-F])", text)
data = bytes(int(t, 16) for t in tokens)
if len(data) < 16:
    print(f"short read: {len(data)} bytes")
    sys.exit(3)

magic, seq, status, length = struct.unpack_from("<IIII", data, 0)
if magic != 0x4D424F58:
    print(f"no mailbox record (magic=0x{magic:08X}, expected 0x4D424F58)")
    print("the image on the board probably does not use the mailbox")
    sys.exit(4)

length = min(length, len(data) - 16, 200)
msg = data[16:16 + length]
names = {0: "info", 1: "PASS", 2: "FAIL"}
print(f"seq={seq} status={names.get(status, status)} len={length}")
print(msg.decode("utf-8", "replace"))
'
