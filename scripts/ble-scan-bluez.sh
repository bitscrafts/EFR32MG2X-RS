#!/usr/bin/env bash
# Does the BlueZ stack accept the board's advertisement, or only the raw HCI capture?
#
# `btmon` reads the controller's events directly and will show a report the host stack later drops. If the
# question is "will a phone see it", the answer has to come from a stack, not from the capture. This runs a
# scan with bluetoothd, confirms the adapter is really discovering, and lists what the stack kept.
set -u

echo "=== adapter state"
bluetoothctl show 2>/dev/null | grep -E "Powered|Discovering|Controller" | head -4

echo "=== scan on"
(bluetoothctl scan on >/tmp/bt-scan.log 2>&1 &) || true
sleep 18

echo "=== during the scan"
bluetoothctl show 2>/dev/null | grep -E "Discovering" | head -2

echo "=== devices the stack kept:"
bluetoothctl devices 2>/dev/null | head -12

echo "=== our board:"
if bluetoothctl devices 2>/dev/null | grep -qi "C0:DE:00:00:00:01"; then
  echo "LISTED"
  bluetoothctl info C0:DE:00:00:00:01 2>/dev/null | head -10
else
  echo "not listed by the stack"
fi

bluetoothctl scan off >/dev/null 2>&1 || true
