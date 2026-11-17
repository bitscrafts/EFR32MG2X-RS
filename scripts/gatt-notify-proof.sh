#!/bin/sh
# Subscribe to the dosimeter's notifications from the host and count them for a minute.
#
# This is the data path the phone uses: connect, find the characteristic, enable notifications, count what arrives.
# Doing it from the lnx means the proof does not depend on a phone being in someone's hand, and the count can be
# compared with the firmware's own window counter in the same minute.
#
# Usage: gatt-notify-proof.sh [address] [seconds]
set -e
ADDR="${1:-E4:0A:75:13:3E:E0}"
SECONDS_TO_RUN="${2:-60}"

{
  echo "connect ${ADDR}"
  sleep 4
  echo "menu gatt"
  sleep 1
  echo "list-attributes"
  sleep 2
  # The three characteristics of the dosimeter service; the first is the RMS value, notified once a second.
  echo "select-attribute /org/bluez/hci0/dev_$(echo "${ADDR}" | tr ':' '_')/service0010/char0011"
  sleep 1
  echo "notify on"
  sleep "${SECONDS_TO_RUN}"
  echo "notify off"
  sleep 1
  echo "back"
  echo "disconnect ${ADDR}"
  sleep 2
  echo "quit"
} | bluetoothctl --timeout $((SECONDS_TO_RUN + 30))
