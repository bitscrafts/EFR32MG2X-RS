#!/bin/sh
# Connect to the board and read its Environmental Sensing values.
#
#   sh host-env-read.sh <address> [notify-seconds]
#
# Two things had to be right, and both cost a round to find:
#
#   1. `list-attributes` takes a device address. Without it the answer is "Missing device address argument", which
#      looks like a device with no attributes.
#   2. A device learned during a scan is *temporary*: BlueZ drops it when discovery stops, so `connect` in a later
#      process answers "not available" for a board that is transmitting. Keeping one bluetoothctl alive with discovery
#      on holds the object for the whole test.
ADDR="${1:-E4:0A:75:13:3E:E0}"
NOTIFY="${2:-12}"

# A persistent scanner: discovery stays on, so the device object survives.
{
  printf 'scan on\n'
  sleep $((NOTIFY + 40))
  printf 'quit\n'
} | sudo timeout $((NOTIFY + 45)) bluetoothctl --timeout $((NOTIFY + 42)) > /tmp/scan-hold.log 2>&1 &

SCANNER=$!
sleep 10

echo "=== connect:"
sudo timeout 30 bluetoothctl --timeout 25 connect "$ADDR" 2>&1 |
  grep -iE "Connection successful|Failed to connect|not available|AlreadyConnected" | head -2

echo "=== attributes:"
{
  printf 'menu gatt\n'
  printf 'list-attributes %s\n' "$ADDR"
  sleep 3
  printf 'quit\n'
} | sudo timeout 30 bluetoothctl --timeout 25 > /tmp/envattrs.log 2>&1
grep -E "Characteristic" /tmp/envattrs.log | sed 's/.*Characteristic - //' | head -8
echo "characteristics: $(grep -c 'Characteristic -' /tmp/envattrs.log)"

echo "=== reads:"
for UUID in 00002a6e-0000-1000-8000-00805f9b34fb 00002a6f-0000-1000-8000-00805f9b34fb 00002a6d-0000-1000-8000-00805f9b34fb; do
  {
    printf 'menu gatt\n'
    printf 'select-attribute %s\n' "$UUID"
    sleep 1
    printf 'read\n'
    sleep 2
    printf 'quit\n'
  } | sudo timeout 25 bluetoothctl --timeout 20 > "/tmp/envread-$UUID.log" 2>&1
  VALUE="$(grep -oE "Value: .*|value: .*" "/tmp/envread-$UUID.log" | head -1)"
  printf '  %s  %s\n' "$UUID" "${VALUE:-(no value)}"
done

echo "=== notify (temperature, ${NOTIFY}s):"
{
  printf 'menu gatt\n'
  printf 'select-attribute 00002a6e-0000-1000-8000-00805f9b34fb\n'
  sleep 1
  printf 'notify on\n'
  sleep "$NOTIFY"
  printf 'quit\n'
} | sudo timeout $((NOTIFY + 20)) bluetoothctl --timeout $((NOTIFY + 15)) > /tmp/envnotify.log 2>&1
grep -E "Notify started|Value:|Notification" /tmp/envnotify.log | head -6
echo "notifications: $(grep -cE 'Value:' /tmp/envnotify.log)"

kill $SCANNER 2>/dev/null
