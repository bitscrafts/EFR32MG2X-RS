#!/usr/bin/env bash
# Every identifier of every device on the bench, read from the device itself.
set -u
echo "=== /sys/bus/usb (vendor:product, serial, product, port) ==="
for d in /sys/bus/usb/devices/*/; do
  v=$(cat "$d/idVendor" 2>/dev/null) || continue
  p=$(cat "$d/idProduct" 2>/dev/null)
  s=$(cat "$d/serial" 2>/dev/null)
  pr=$(cat "$d/product" 2>/dev/null)
  m=$(cat "$d/manufacturer" 2>/dev/null)
  printf '%-8s %s:%s  serial=%-20s  %s / %s\n' "$(basename "$d")" "$v" "$p" "${s:--}" "${m:--}" "${pr:--}"
done

echo
echo "=== ttys ==="
for t in /dev/ttyACM* /dev/ttyUSB*; do
  [ -e "$t" ] || continue
  echo "$t -> $(udevadm info -q property -n "$t" 2>/dev/null | grep -E '^ID_SERIAL=' | cut -d= -f2-)"
done

echo
echo "=== bluetooth controllers ==="
for h in /sys/class/bluetooth/hci*; do
  [ -e "$h" ] || continue
  echo "$(basename "$h"): $(hciconfig "$(basename "$h")" 2>/dev/null | grep 'BD Address' | awk '{print $3}') ($(hciconfig "$(basename "$h")" 2>/dev/null | grep -o 'Bus: [A-Za-z]*'))"
done

echo
echo "=== network interfaces (the dongle) ==="
for n in /sys/class/net/*; do
  [ -e "$n" ] || continue
  mac=$(cat "$n/address" 2>/dev/null)
  echo "$(basename "$n"): $mac"
done
