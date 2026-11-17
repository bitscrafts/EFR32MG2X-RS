#!/bin/sh
# Pair with the XIAO pairing probe from the lnx dongle, in ONE bluetoothctl session.
#
# One session matters: a device learned during a scan is dropped when the discovering process exits, so a `pair`
# issued from a second process answers "Device not available" for something that is right there on the air.
#
# Run as root on the lnx (`sudo -n`): without it the HCI socket is not available and discovery answers
# "Operation not permitted".
#
# Usage: pair-probe.sh [address]
set -e
ADDR="${1:-E4:0A:75:13:3E:E0}"

{
  echo "scan on"
  sleep 10
  echo "pair ${ADDR}"
  sleep 16
  echo "info ${ADDR}"
  sleep 2
  echo "quit"
} | bluetoothctl --timeout 45
