#!/bin/sh
# Read the board's Environmental Sensing values with one long-lived bluetoothctl session.
#
#   sh host-env-session.sh <address> [notify-seconds]
#
# **One process, with the timing in the writer.** BlueZ removes a device object when the bluetoothctl that discovered it
# exits, so a scan in one invocation and a connect in the next answers "not available" for a board that is on the air --
# verified on 2026-11-14 with the board transmitting and the scan seeing it. Feeding the commands into a single session
# keeps the object alive, and the sleeps in the writer give discovery time without a second process.
ADDR="${1:-E4:0A:75:13:3E:E0}"
NOTIFY="${2:-12}"

{
  printf 'scan on\n'
  sleep 12
  printf 'connect %s\n' "$ADDR"
  # BlueZ answers a connect attempt by doing its own directed scan; the link is up a few seconds later. Commands issued
  # before that land on a device with no attributes and `list-attributes` prints nothing -- which looks like a firmware
  # with an empty database.
  sleep 10
  printf 'menu gatt\n'
  printf 'list-attributes\n'
  sleep 3
  # The three standard characteristics of the Environmental Sensing service. `select-attribute` accepts the UUID.
  for UUID in 00002a6e-0000-1000-8000-00805f9b34fb 00002a6f-0000-1000-8000-00805f9b34fb 00002a6d-0000-1000-8000-00805f9b34fb; do
    printf 'select-attribute %s\n' "$UUID"
    sleep 1
    printf 'read\n'
    sleep 1
  done
  printf 'select-attribute 00002a6e-0000-1000-8000-00805f9b34fb\n'
  sleep 1
  printf 'notify on\n'
  sleep "$NOTIFY"
  printf 'quit\n'
} | sudo timeout $((NOTIFY + 45)) bluetoothctl --timeout $((NOTIFY + 40)) > /tmp/envsession.log 2>&1

echo "=== connect:"
grep -iE "Connection successful|Failed to connect|not available" /tmp/envsession.log | head -2

echo "=== characteristics discovered: $(grep -c 'Characteristic' /tmp/envsession.log)"
grep -oE "Characteristic - [0-9a-f-]+" /tmp/envsession.log | sort -u | head -8

echo "=== reads:"
grep -A1 -E "Attempting to read|^\[.*\]# read" /tmp/envsession.log | grep -E "Value|hex" | head -6
grep -E "Value:|value/hex" /tmp/envsession.log | head -6

echo "=== notifications (temperature):"
grep -E "Notify started|Value:|Notification" /tmp/envsession.log | head -8
