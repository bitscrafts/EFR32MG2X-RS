#!/usr/bin/env bash
# Make the host's adapter advertise an advertisement we control byte by byte.
#
# ## THE RULE THIS SCRIPT EXISTS TO RESPECT
#
# On this host the Bluetooth controller and the Wi-Fi interface are two functions of ONE USB dongle,
# and that Wi-Fi interface is the only route off the machine:
#
#     wlx90de80814455  UP   10.10.10.217/24   default via 10.10.10.1   (USB 1-8, interface 1.2)
#     hci0             BD   90:DE:80:81:44:56                          (USB 1-8, interface 1.0)
#     enp6s0           DOWN
#
# Anything that powers this device down removes the host from the network, including the SSH session
# running this. `hciconfig hci0 down` is in that class and is not used here. The guard below refuses to
# proceed while the two are shared unless ALLOW_SHARED_RADIO=1 is set deliberately.
#
# ## The three mistakes this script has already paid for
#
# 1. **hcitool cmd takes ONE BYTE PER ARGUMENT.** Passing `020106` as an argument sends seven bytes and
#    silently truncates the rest: the controller answered Success, the advertisement went out without a
#    name, and a scanner showed the address with "N/A". Every byte here is its own argument.
# 2. **bluetoothd 5.82 drives extended advertising.** While an extended set is enabled, the legacy
#    advertising data is not what goes on the air — and disabling "no sets" disables nothing. The set
#    handle has to be named.
# 3. The advertising parameters must be set while advertising is DISABLED, otherwise the controller
#    answers Command Disallowed (0x0c).
#
# ## What it advertises
#
#   02 01 06                        flags: LE general discoverable, BR/EDR not supported
#   08 09 "lnx-ble"                 complete local name
#   07 FF FF FF 58 49 41 4F         manufacturer data, company 0xFFFF, payload "XIAO"
set -euo pipefail

IFACE=${IFACE:-hci0}
# The name on THIS host is the host's. "xiao-mg24" belongs to the board, and using it here once made a
# scanner listing look like the board was advertising when it was not.
NAME=${NAME:-lnx-ble}

usb_parent() {
  local p
  p=$(readlink -f "$1" 2>/dev/null) || return 1
  while [ -n "$p" ] && [ "$p" != "/" ]; do
    if [ -e "$p/idVendor" ]; then echo "$p"; return 0; fi
    p=$(dirname "$p")
  done
  return 1
}

route_iface=$(ip route | awk '/^default/ {for (i=1;i<=NF;i++) if ($i=="dev") print $(i+1)}' | head -1)
bt_usb=$(usb_parent "/sys/class/bluetooth/$IFACE/device" || true)
net_usb=""
[ -n "$route_iface" ] && net_usb=$(usb_parent "/sys/class/net/$route_iface/device" || true)

same_radio=no
[ -n "$bt_usb" ] && [ "$bt_usb" = "$net_usb" ] && same_radio=yes

if [ "$same_radio" = "yes" ] && [ "${ALLOW_SHARED_RADIO:-0}" != "1" ]; then
  cat >&2 <<EOF
REFUSED: the Bluetooth controller and the default route share one USB device (1-8).

Powering it down removes the host from the network, including the SSH session running this. Nothing
below does that — this is a conservative refusal, not a warning about the next line.

  ALLOW_SHARED_RADIO=1 bash scripts/host-advertise.sh
EOF
  exit 1
fi

hci() { sudo -n hcitool -i "$IFACE" cmd "$@"; }

# Build the AD payload as individual bytes, one per argument.
name_bytes=""
for ((i = 0; i < ${#NAME}; i++)); do
  name_bytes+=" $(printf '%02x' "'${NAME:$i:1}")"
done
namelen=$(printf '%02x' $(( ${#NAME} + 1 )))
adv="02 01 06 ${namelen} 09${name_bytes} 07 ff ff ff 58 49 41 4f"
advlen=$(printf '%02x' "$(echo "$adv" | wc -w)")

sudo -n systemctl stop bluetooth 2>/dev/null || true
sleep 1
sudo -n hciconfig "$IFACE" up 2>/dev/null || true

# 1. Disable advertising: extended set 0 by handle, then any legacy advertising.
hci 0x08 0x0039 00 01 00 00 00 00 >/dev/null 2>&1 || true
hci 0x08 0x000a 00 >/dev/null 2>&1 || true

# 2. Legacy parameters, built field by field so the length cannot drift: the field list is exactly
#    15 bytes and a stray zero answers Invalid HCI Command Parameters (0x12). Fourteen and sixteen
#    were both tried before this was assembled from the field list.
params="A0 00 A0 00"           # interval min, interval max (0x00A0 = 100 ms)
params+=" 00 00 00"            # adv type (ADV_IND), own address type, peer address type
params+=" 00 00 00 00 00 00"   # peer address
params+=" 07 00"               # channel map (all three), filter policy
if [ "$(echo "$params" | wc -w)" -ne 15 ]; then
  echo "REFUSED: advertising parameters are $(echo "$params" | wc -w) bytes, the command takes 15" >&2
  exit 1
fi
hci 0x08 0x0006 $params >/dev/null

# 3. The same data in the advertising and the scan-response block: a scanner that only surfaces one of
#    them must still see the name.
hci 0x08 0x0008 "$advlen" $adv >/dev/null
hci 0x08 0x0009 "$advlen" $adv >/dev/null

# 4. Enable legacy advertising.
hci 0x08 0x000a 01 >/dev/null

echo "advertising as '$NAME' on $IFACE (shared radio: $same_radio, USB $(basename "${bt_usb:-?}"))"
echo "AD bytes ($((16#${advlen})) total): $adv"
echo "note: bluetoothd is stopped; restore with 'systemctl start bluetooth'"
