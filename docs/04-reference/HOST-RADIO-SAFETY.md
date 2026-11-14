# HOST RULE — the radio on this host carries the network

Measured 2026-11-12, on the host that owns the boards.

## The fact

```
wlx90de80814455  UP   ${FLASH_HOST:-localhost}/24    default via 10.10.10.1     USB 1-8, interface 1-8:1.2
hci0             UP   90:DE:80:81:44:56                             USB 1-8, interface 1-8:1.0
enp6s0           DOWN
```

The Wi-Fi interface and the Bluetooth controller are **two functions of one USB device**, and the Wi-Fi
interface is the **only route off the machine**. Every SSH session used to do this work arrives over it:

```
ESTAB  ${FLASH_HOST:-localhost}:22 <- 10.10.10.210:54820
```

So a command that powers, resets or unbinds that USB device does not "disrupt a peripheral" — it removes
the host from the network and closes the session that issued it, with no way back in short of physical
access.

## FORBIDDEN on this host

- `hciconfig hci0 down`, `btmgmt power off`, `rfkill block`, `systemctl restart bluetooth` while a
  test is mid-flight — anything that powers the radio block down.
- Unbinding, resetting or `authorized 0/1` on USB device `1-8`, or on the xHCI controller
  `0000:00:14.0` that hosts it. This extends the existing rule against rebinding `xhci_hcd`.
- Reloading `btusb`, `btusb_mtk`, `rtw88*` or any module owned by that device.

## ALLOWED

- `hciconfig hci0 up` when it is already up (no-op).
- HCI commands that configure advertising or scanning: `hcitool cmd 0x08 ...`, `hcitool lescan`.
- Reading: `btmon`, `hciconfig`, `lsusb`, `udevadm`.

## The guard

`scripts/host-advertise.sh` refuses to run while the default route and the Bluetooth controller are the
same USB device, unless `ALLOW_SHARED_RADIO=1` is set deliberately.

Its first version **failed open**, and that is why this file exists. It compared
`/sys/class/net/<if>/device` with `/sys/class/bluetooth/hci0/device`, which on a combo dongle are two
*interfaces* of one device:

```
/sys/devices/.../usb1/1-8/1-8:1.0     <- bluetooth
/sys/devices/.../usb1/1-8/1-8:1.2     <- wi-fi
```

Different paths, same silicon — so the check answered "not shared" and let the script proceed. **A guard
that compares siblings and concludes "safe" is worse than no guard**, because it converts an open risk
into a certified one. It now walks each path up to its USB device directory:

```
USB device: 1-8
REFUSED: the Bluetooth controller and the default route share one USB device.
```

Also removed from the script: the `hciconfig hci0 down` it used to do before programming, which was in
the forbidden class above. It survived being run during development; that is luck, not safety.

## If the network ever does go down

The host is reachable only over Wi-Fi, so recovery needs physical or console access. Do not plan a
recovery that depends on the network being up. Prefer testing from the wired side (`enp6s0` is present
and down — bringing *it* up does not touch the dongle) before any experiment that needs the dongle.
