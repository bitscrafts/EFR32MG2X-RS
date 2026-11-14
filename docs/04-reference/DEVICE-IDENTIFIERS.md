# Identifiers of every device on the bench

Measured 2026-11-12 from the devices themselves. Nothing here is copied off a label; the "how" column is
the provenance, because an identifier whose source is unknown is not evidence.

## The XIAO MG24 Sense (the board under development)

| Identifier | Value | How |
|---|---|---|
| USB VID:PID | `2886:0062` | USB descriptor (`/sys/bus/usb/devices/1-2`) |
| USB serial | `28DB6336` | USB descriptor, iSerial |
| USB port | `1-2` (xHCI, `0000:00:14.0`) | sysfs |
| CDC device | `/dev/ttyACM0` | `udevadm info`, serial matches |
| Debug probe | onboard SAMD11 CMSIS-DAP, same `2886:0062` | the same USB device carries the DAP |
| **Silicon unique id** | **`E40A75FFFE133EE0`** | `SYSTEM_GetUnique()` on the part, via `examples/emlib_identity.rs` |
| Chip revision | major 1, minor 1, part number `0x003C` | `SYSTEM_ChipRevisionGet()` |
| Core clock | 19 000 kHz | `SystemSYSCLKGet()` and `CMU_ClockFreqGet` agree; DWT confirms |

The unique id is the one that matters for the radio: Silicon Labs' stacks derive a device address from
it, so it is how a packet on the air gets attributed to this board rather than to "some board".

## The nice!nano (nRF52840, the second board)

| Identifier | Value | How |
|---|---|---|
| USB VID:PID | `239a:00b3` | USB descriptor (`1-1`) |
| USB serial | `DC92ED80EF7E8C29` | USB descriptor |
| Manufacturer / product strings | `Nice Keyboards` / `nice!nano` | USB descriptor |
| CDC device | `/dev/ttyACM1` | `udevadm info` |
| Bootloader entry | 1200-bps touch (no reset button on this board) | measured: the device re-enumerated |
| UF2 volume | not present — it is not in bootloader mode | `lsblk`, no new block device |

Note the trap: the **same physical board** identified itself as `239a:8029 "Feather nRF52840 Express"`
before the touch and as `239a:00b3 "nice!nano"` after it. The USB identity is the firmware's claim, not
the hardware's — which is why the serial number is the field to match on, and it did not change.

## The host's radios

| Identifier | Value | How |
|---|---|---|
| Dongle USB | `0bda:b851`, serial `00e04c000001` | USB descriptor (`1-8`) |
| Wi-Fi interface | `wlx90de80814455`, MAC `90:de:80:81:44:55` | sysfs |
| Bluetooth controller | `hci0`, BD address `90:DE:80:81:44:56` | `hciconfig` |
| Wired NIC | `enp6s0`, MAC `00:23:09:48:3b:89` | sysfs |
| USB controllers | xHCI `0000:00:14.0`, EHCI `0000:00:1a.0`, `0000:00:1d.0` | sysfs |

The Wi-Fi and Bluetooth addresses differ by one (`…:55` and `…:56`), which is what a single combo radio
looks like: two functions on one chip, adjacent addresses.

## Devices that were part of the bench and are not any more

| Device | Identifier | Why it left |
|---|---|---|
| nRF52840 module (first) | appeared as a full-speed device, never delivered a configuration descriptor (`-110`/`-71`, 52 attempts, two ports, two cables) | genuinely faulty; replaced |
| USB keyboard | `05ac:021e` behind hub `05ac:1005` | removed to simplify the USB tree during the enumeration work |

## Reading the identifiers again

```bash
bash scripts/device-identifiers.sh                 # the host side, from sysfs/udev/hciconfig
```

```bash
# the silicon side: flash the identity example and read its mailbox
cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg24-emlib-sys --example emlib_identity
bash scripts/flash.sh <elf> --confirm --run
ssh <build-host> 'sudo -n openocd -f interface/cmsis-dap.cfg -f /tmp/openocd-mg24-mini.cfg \
  -c "init; halt" -c "mdw 0x2003F000 16" -c "resume; shutdown"'
```
