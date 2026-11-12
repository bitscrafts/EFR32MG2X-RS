#!/usr/bin/env bash
# Make the host's adapter advertise an advertisement we control byte by byte.
#
# ## THE RULE THIS SCRIPT EXISTS TO RESPECT
#
# On this host the Bluetooth controller and the Wi-Fi interface are two functions of ONE USB dongle,
# and that Wi-Fi interface is the only route off the machine:
#
#     wlx90de80814455  UP   10.10.10.217/24   default via 10.10.10.1   (USB 1-8, interface 1.0)
#     hci0             BD   90:DE:80:81:44:56                          (USB 1-8, interface 1.1)
#     enp6s0           DOWN
#
# So anything that powers down, resets or unbinds that USB device takes the host off the network — and
# takes away the SSH session being used to run this. `hciconfig hci0 down` is in that class.
#
# ## Why the guard compares USB PARENTS and not device links
#
# The first version of this guard compared `/sys/class/net/<if>/device` with
# `/sys/class/bluetooth/hci0/device` and concluded "not shared" on this host. That was wrong: on a combo
# dongle those are two INTERFACES of one device (`1-8:1.0` and `1-8:1.1`), so the paths differ while the
# silicon is the same. A guard that answers "safe" because it compared siblings is worse than no guard,
# so the check now walks each path up to its USB device directory and compares those.
#
# ## What it advertises
#
#   02 01 06                        flags: LE general discoverable, BR/EDR not supported
#   0A 09 "xiao-mg24"               complete local name
#   07 FF FF FF 58 49 41 4F         manufacturer data, company 0xFFFF, payload "XIAO"
set -euo pipefail

IFACE=${IFACE:-hci0}
NAME=${NAME:-xiao-mg24}

# Walk up a sysfs path until it names a USB device directory (e.g. .../usb1/1-8) and print it.
usb_parent() {
  local p
  p=$(readlink -f "$1" 2>/dev/null) || return 1
  while [ -n "$p" ] && [ "$p" != "/" ]; do
    if [ -e "$p/idVendor" ]; then
      echo "$p"
      return 0
    fi
    p=$(dirname "$p")
  done
  return 1
}

route_iface=$(ip route | awk '/^default/ {for (i=1;i<=NF;i++) if ($i=="dev") print $(i+1)}' | head -1)

bt_usb=$(usb_parent "/sys/class/bluetooth/$IFACE/device" || true)
net_usb=""
[ -n "$route_iface" ] && net_usb=$(usb_parent "/sys/class/net/$route_iface/device" || true)

same_radio=no
if [ -n "$bt_usb" ] && [ "$bt_usb" = "$net_usb" ]; then
  same_radio=yes
fi

if [ "$same_radio" = "yes" ] && [ "${ALLOW_SHARED_RADIO:-0}" != "1" ]; then
  cat >&2 <<EOF
REFUSED: the Bluetooth controller and the default route share one USB device.

  default route interface : $route_iface
  bluetooth controller    : $IFACE
  USB device              : $(basename "${bt_usb:-unknown}")

Powering this device down removes the host from the network, including the SSH session running this.
No HCI command here does that, but the script will not run at all while the two are shared unless you
say you understand:

  ALLOW_SHARED_RADIO=1 bash scripts/host-advertise.sh
EOF
  exit 1
fi

# No `hciconfig down` anywhere: bringing the interface up is enough, and taking it down is the operation
# that can cost the network.
sudo -n hciconfig "$IFACE" up 2>/dev/null || true

hexname=$(printf '%s' "$NAME" | xxd -p | tr -d '\n')
# AD structure is [length][type][data]; length counts the type byte plus the data.
namelen=$(printf '%02x' $(( ${#NAME} + 1 )))
payload="020106 ${namelen}09${hexname} 07ffff 58 49 41 4f"
total=$(printf '%02x' $(( $(echo "$payload" | tr -d ' ' | wc -c) / 2 )))

sudo -n hcitool -i "$IFACE" cmd 0x08 0x0006 A0 00 A0 00 00 00 00 00 00 00 00 00 00 00 07 00 >/dev/null
sudo -n hcitool -i "$IFACE" cmd 0x08 0x0008 "$total" $payload >/dev/null
sudo -n hcitool -i "$IFACE" cmd 0x08 0x000a 01 >/dev/null

echo "advertising as '$NAME' on $IFACE (shared radio: $same_radio, USB $(basename "${bt_usb:-?}"))"
echo "AD bytes: $payload"
