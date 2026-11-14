#!/bin/sh
# Count how often the board appears on air during a fixed window.
timeout 12 btmon > /tmp/air.txt 2>/dev/null &
BT=$!
sleep 2
timeout 8 bluetoothctl --timeout 6 scan on >/dev/null 2>&1
sleep 2
wait $BT 2>/dev/null
echo "board=$(grep -c 'E4:0A:75:13:3E:E0' /tmp/air.txt) total=$(grep -c 'LE Advertising Report' /tmp/air.txt)"
