#!/bin/sh
# Pair the XIAO MG24 probe from this host, from a clean state.
# The device's bonds are cleared by the firmware (CLEAR_BONDS ran in its previous boot), so the only thing that can be
# stale here is the host's own bond -- and "AlreadyExists" is exactly what a stale host bond looks like.
set -u
DEV="E4:0A:75:13:3E:E0"

{
  echo "remove $DEV"
  sleep 2
  echo "scan on"
  sleep 6
  echo "scan off"
  echo "pair $DEV"
  sleep 12
  echo "info $DEV"
  sleep 2
  echo "quit"
} | timeout 60 bluetoothctl 2>&1 | sed 's/\r//' | grep -iE "Device has been removed|not available|Attempting|Pairing successful|Failed|Bonded:|Paired:|Connected:" | tail -12
