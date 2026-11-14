#!/bin/sh
# Capture the board's advertising payloads during a scan, and print the distinct frames.
#
#   sh host-payload.sh [seconds]
#
# `host-aircheck.sh` counts appearances; this prints what was in them, which is the difference between "something
# transmits" and "this example transmits its own data".
SECS="${1:-12}"
timeout $((SECS + 4)) btmon > /tmp/payload.txt 2>/dev/null &
BT=$!
sleep 2
timeout "$SECS" bluetoothctl --timeout $((SECS - 2)) scan on >/dev/null 2>&1
sleep 2
wait $BT 2>/dev/null
echo "board reports: $(grep -c 'E4:0A:75:13:3E:E0' /tmp/payload.txt)"
grep -o 'Data\[[0-9]*\]: [0-9a-f]*' /tmp/payload.txt | sort -u | head -6
