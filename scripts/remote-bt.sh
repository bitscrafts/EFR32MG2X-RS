#!/bin/sh
# Run a Bluetooth measurement script on the host that owns the adapter, and return its output.
#
# The adapter is on `lnx` (10.10.10.217), not in this container: `bluetoothctl` here has no controller. Every BLE
# measurement in this project therefore runs through this wrapper, so the script has exactly one way to reach the radio and
# there is no chance of a measurement running against a different machine than the one being claimed.
#
#   scripts/remote-bt.sh scripts/pair-clean.sh
#
# The script is piped over stdin rather than copied, so the version that runs is always the version in the tree.
set -eu

HOST="${FLASH_HOST:-10.10.10.217}"
USER_NAME="${FLASH_USER:-mvcorrea}"

if [ "$#" -lt 1 ]; then
  echo "usage: $0 <script> [args...]" >&2
  exit 2
fi

SCRIPT="$1"
shift

if [ ! -f "$SCRIPT" ]; then
  echo "REFUSE: $SCRIPT does not exist" >&2
  exit 3
fi

# `bash -s` with the argument list after it, so a script that takes parameters keeps them.
ssh -o BatchMode=yes -o ConnectTimeout=10 "${USER_NAME}@${HOST}" "bash -s -- $*" < "$SCRIPT"
