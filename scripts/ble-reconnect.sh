#!/bin/sh
# Reconnect to the probe WITHOUT pairing, to prove the bond survived a reset.
# The point is not "can it connect" -- it is whether the link stays up: a missing long-term key shows as
# "Connected: yes" followed by an immediate drop with HCI 0x1006 PIN or Key Missing.
set -u
DEV="E4:0A:75:13:3E:E0"

{
  echo "connect $DEV"
  sleep 8
  echo "info $DEV"
  sleep 2
  echo "quit"
} | timeout 60 bluetoothctl 2>&1 | sed 's/\r//' | grep -iE "Attempting|Connection successful|Failed|Connected: |Bonded: |Paired: |0x1006|auth failed" | tail -10
