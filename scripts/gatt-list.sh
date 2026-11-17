#!/bin/sh
# List the dosimeter's GATT attributes from the host, so the notification test can name them exactly.
#
# Usage: gatt-list.sh [address]
set -e
ADDR="${1:-E4:0A:75:13:3E:E0}"

{
  echo "connect ${ADDR}"
  sleep 6
  echo "menu gatt"
  sleep 1
  echo "list-attributes ${ADDR}"
  sleep 3
  echo "quit"
} | bluetoothctl --timeout 30
