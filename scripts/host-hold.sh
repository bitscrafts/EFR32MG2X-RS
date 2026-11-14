#!/bin/sh
# Connect to the board and hold the link, subscribing only if asked.
#
#   sh host-hold.sh <address> <hold-seconds> [notify]
#
# Two experiments share this: a client that connects and does nothing, and one that subscribes. The difference between
# the heap numbers the board reports afterwards is what says whether the damage comes from the connection or from the
# notification path.
ADDR="${1:-E4:0A:75:13:3E:E0}"
HOLD="${2:-20}"
NOTIFY="${3:-}"

{
  printf 'scan on\n'
  sleep 12
  printf 'connect %s\n' "$ADDR"
  sleep 8
  if [ -n "$NOTIFY" ]; then
    printf 'menu gatt\n'
    printf 'select-attribute 00002a6e-0000-1000-8000-00805f9b34fb\n'
    sleep 1
    printf 'notify on\n'
  fi
  sleep "$HOLD"
  printf 'quit\n'
} | sudo timeout $((HOLD + 45)) bluetoothctl --timeout $((HOLD + 40)) 2>&1 |
  grep -iE "Connection successful|Notify started|Failed|not available" | head -3
