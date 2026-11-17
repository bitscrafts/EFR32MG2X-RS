#!/bin/sh
# Forget the pairing probe on the lnx, so the next `pair` is a real one.
#
# `pair` against a device the host already knows answers "org.bluez.Error.AlreadyExists" and never touches the
# link, which reads exactly like a working bond and hides the state of the device's own store.
#
# Usage: unpair-probe.sh [address]
set -e
ADDR="${1:-E4:0A:75:13:3E:E0}"

{
  echo "remove ${ADDR}"
  sleep 2
  echo "quit"
} | bluetoothctl --timeout 20
