#!/bin/sh
# Read the board's Environmental Sensing characteristics with bluetoothctl's own GATT client.
#
#   sh host-env-bluetoothctl.sh <address> [seconds]
#
# Sequential on purpose. Earlier attempts backgrounded a scan and killed it, or stopped discovery right after finding
# the device; both leave BlueZ with no usable device object and the next command says "not available", which reads like
# a firmware fault. This runs exactly the sequence that has been working: reset, a scan that is allowed to finish, then
# connect.
ADDR="${1:-E4:0A:75:13:3E:E0}"

sudo hciconfig hci0 reset >/dev/null 2>&1
sleep 3
sudo timeout 18 bluetoothctl --timeout 15 scan on >/dev/null 2>&1
echo "=== connect:"
sudo timeout 30 bluetoothctl --timeout 25 connect "$ADDR" 2>&1 | grep -iE "Connection successful|Failed|not available" | head -2

echo "=== attributes:"
sudo timeout 30 bluetoothctl --timeout 25 > /tmp/gatt-list.log 2>&1 <<'EOF'
menu gatt
list-attributes
EOF
grep -E "Characteristic" /tmp/gatt-list.log | sed 's/.*\(0000....-0000-1000-8000-00805f9b34fb\).*/\1/' | head -6
echo "characteristics: $(grep -c 'Characteristic' /tmp/gatt-list.log)"

# One read per standard characteristic, found by UUID in the listing.
for UUID in 00002a6e 00002a6f 00002a6d; do
  PATH_LINE="$(grep -i "Characteristic.*$UUID" /tmp/gatt-list.log | head -1 | awk '{print $2}')"
  [ -z "$PATH_LINE" ] && continue
  echo "=== read $UUID ($PATH_LINE):"
  printf 'menu gatt\nselect-attribute %s\nread\n' "$PATH_LINE" | sudo timeout 25 bluetoothctl --timeout 20 2>&1 |
    grep -E "Value|Attempting" | head -2
done

echo "=== notify (temperature, 10 s):"
TPATH="$(grep -i 'Characteristic.*00002a6e' /tmp/gatt-list.log | head -1 | awk '{print $2}')"
if [ -n "$TPATH" ]; then
  printf 'menu gatt\nselect-attribute %s\nnotify on\n' "$TPATH" | sudo timeout 20 bluetoothctl --timeout 15 2>&1 |
    grep -E "Notify|Value" | head -6
fi
