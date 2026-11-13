#!/usr/bin/env bash
# Scan BLE at 100% duty cycle, through raw HCI, and report every advertiser seen.
#
# `hcitool lescan` uses the controller's default scan parameters, whose window is a small fraction of
# the interval: a beacon that transmits ~11 times a second on rotating channels can be missed for long
# stretches and the result looks like "the device is not transmitting" when it is.
#
# This sets window == interval (continuous listening) and captures the raw reports with btmon, which is
# the observer that actually works here -- `bluetoothctl` and `hcitool lescan` print nothing on this
# adapter even while btmon records dozens of advertising reports.
set -u

SECONDS_TO_SCAN=${1:-20}
SNOOP=/tmp/scan-raw.snoop

if ! command -v hcitool >/dev/null || ! command -v btmon >/dev/null; then
  echo "hcitool/btmon missing" >&2
  exit 1
fi

# The daemon holds the adapter; it is a daemon, not the radio, so stopping it is safe here. The radio
# itself is never powered down: this host's Wi-Fi shares the same USB device (see HOST-RADIO-SAFETY.md).
systemctl stop bluetooth >/dev/null 2>&1 || true
sleep 1
hciconfig hci0 up >/dev/null 2>&1 || true

timeout $((SECONDS_TO_SCAN + 6)) btmon -w "$SNOOP" >/dev/null 2>&1 &
sleep 1

# LE Set Scan Parameters: active, interval 100 ms, window 100 ms (continuous), public address, no filter.
hcitool cmd 0x08 0x000b 01 a0 00 a0 00 00 00 >/dev/null 2>&1
# LE Set Scan Enable: on, no duplicate filtering (so a single device is still counted repeatedly).
hcitool cmd 0x08 0x000c 01 00 >/dev/null 2>&1
sleep "$SECONDS_TO_SCAN"
hcitool cmd 0x08 0x000c 00 00 >/dev/null 2>&1
sleep 5

echo "=== advertising reports, by address:"
btmon -r "$SNOOP" 2>/dev/null |
  grep -oE "Address: ([0-9A-F]{2}:){5}[0-9A-F]{2}" |
  sort | uniq -c | sort -rn | head -12

echo "=== any report carrying a local name:"
btmon -r "$SNOOP" 2>/dev/null | grep -iE "Name \(|Complete Local Name|Shortened Local Name" | head -8

echo "=== our board (F5:FF:FE:13:3E:E0) or its name:"
if btmon -r "$SNOOP" 2>/dev/null | grep -qiE "F5:FF:FE:13:3E:E0|xiao"; then
  btmon -r "$SNOOP" 2>/dev/null | grep -iE "F5:FF:FE:13:3E:E0|xiao" | head -6
  echo "FOUND"
else
  echo "not seen"
fi
