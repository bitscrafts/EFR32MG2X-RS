#!/usr/bin/env bash
# Make the host's adapter advertise an advertisement we control byte by byte.
#
# `bluetoothctl advertise on` only advertises flags, so the device shows up in a scanner as an
# unnamed address. For a radio test we want two things it does not give: a name a human can
# recognise, and a payload our own firmware can verify byte for byte.
#
# The advertisement built here:
#
#   02 01 06                        flags: LE general discoverable, BR/EDR not supported
#   08 09 6C 6E 78 2D 62 6C 65      complete local name "lnx-ble"
#   07 FF FF FF 58 49 41 4F         manufacturer data, company 0xFFFF, payload "XIAO"
#
# 0xFFFF is the "test, do not use in production" company id, which is the honest choice for a test
# payload: it cannot be mistaken for a real product identifier.
#
# NOTE the service stop: bluetoothd manages the adapter and will fight hand-written HCI commands. This
# script stops it, programs the controller directly, and says so. Start it again with
# `systemctl start bluetooth` when done.
set -euo pipefail

IFACE=${IFACE:-hci0}
NAME=${NAME:-lnx-ble}

# Build the AD payload from the name, so changing NAME cannot desynchronise the length byte.
hexname=$(printf '%s' "$NAME" | xxd -p | tr -d '\n')
namelen=$(printf '%02x' $(( ${#NAME} + 1 )))
# AD structure is [length][type][data]: length counts the type byte plus the data.
payload="020106 ${namelen}09${hexname} 07ffff 58 49 41 4f"

sudo -n systemctl stop bluetooth 2>/dev/null || true
sudo -n hciconfig "$IFACE" down 2>/dev/null || true
sudo -n hciconfig "$IFACE" up

# Advertise every 100 ms (0x00A0 = 160 units of 0.625 ms) on all three advertising channels,
# connectable, public address.
sudo -n hcitool -i "$IFACE" # 15 parameter bytes: min,max interval | adv_type | own_addr_type | peer_addr_type |
# peer address (6 bytes) | channel map | filter policy. The first version was one byte short and
# the controller answered Invalid HCI Command Parameters -- which is the good kind of failure.
cmd 0x08 0x0006 A0 00 A0 00 00 00 00 00 00 00 00 00 00 00 07 00 >/dev/null
sudo -n hcitool -i "$IFACE" cmd 0x08 0x0008 $(printf '%02x' $(( $(echo $payload | tr -d ' ' | wc -c) / 2 ))) $payload >/dev/null
sudo -n hcitool -i "$IFACE" cmd 0x08 0x000a 01 >/dev/null

echo "advertising as '$NAME' on $IFACE"
echo "AD bytes: $payload"
