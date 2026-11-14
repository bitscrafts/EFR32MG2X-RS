#!/bin/sh
# Count the board's advertising reports using the raw HCI path, without BlueZ's discovery.
#
# `bluetoothctl scan on` goes through bluetoothd, and on this host that path stopped reporting anything at all after a
# few daemon restarts -- not even neighbours -- which makes every conclusion about the board untestable. `hcitool lescan`
# drives the controller directly, so it answers "is the radio working" even when the daemon is not.
timeout 12 btmon > /tmp/raw.txt 2>/dev/null &
BT=$!
sleep 2
timeout 8 hcitool -i hci0 lescan --duplicates >/dev/null 2>&1
sleep 2
wait $BT 2>/dev/null
echo "board=$(grep -c 'E4:0A:75:13:3E:E0' /tmp/raw.txt) reports=$(grep -c 'LE Advertising Report' /tmp/raw.txt) neighbours=$(grep -oE '[0-9A-F]{2}:[0-9A-F]{2}:[0-9A-F]{2}:[0-9A-F]{2}:[0-9A-F]{2}:[0-9A-F]{2}' /tmp/raw.txt | sort -u | wc -l)"
