# Testing the radio with an nRF52840

The EFR32 board can advertise and scan on its own, but a receiver built by somebody else is what proves
an antenna. An nRF52840 dongle is the cheapest way to get two independent roles: a **sniffer** that sees
the packets, and a **central** that connects.

## What the dongle can be, and what each role answers

| Role | Firmware | The question it answers |
|---|---|---|
| Sniffer | nRF Sniffer for Bluetooth LE | **what did the board actually transmit?** — address, AD structures, RSSI, per-packet |
| Central | nRF Connect for Desktop → Bluetooth Low Energy | **can something connect, discover a service and subscribe?** — the `ble_gatt` test |
| Advertiser | Nordic's `ble_app_beacon` sample (nRF5 SDK) or a Zephyr `beacon` | **does the board's scanner hear a known-good advertiser?** — the receive path |
| Phone | nRF Connect for Mobile | the human check: does the name appear, does the payload change |

The vendor's own product page for the nRF Sniffer lists the **nRF52840 Dongle** among the supported
boards, so a dongle is not a workaround here — it is the intended hardware.

## The sniffer is the instrument this project lacked

Every claim about advertising so far has been indirect: the board says it is advertising, and a phone
either sees it or does not. A sniffer turns that into the packet itself:

```text
AdvA = the board's address        AD Structure 1: Flags (0x01) = 0x06
AD Structure 2: Manufacturer Specific (0xFF), company 0x02E5
                payload 01 02 00 3c 1f 5f 09 00 00 e8 03
```

That hex is the fall example's frame: version 1, flags 0x02 (a fall was reported), two falls, peak
0x001f = 31? — the reader does not have to trust the board's own mailbox, which is the point.

## Order of work, when the dongle is in hand

1. **Sniffer first, no connection involved.** Flash the sniffer firmware (the package ships a `.hex` for
   the dongle), start Wireshark with the nRF Sniffer plugin, and capture the board running
   `module_ble_motion` (the IMU one). A frame that changes when you tilt the board proves the sensor,
   the encoder and the radio in one capture — with no phone in the loop.
2. **Then the scanner.** Run `ble_scanner` on the EFR32 and let the dongle advertise (Nordic's beacon
   sample). The mailbox should report an address and an RSSI. This is the receive path proven against an
   implementation that was not written here.
3. **Then `ble_gatt`, once it links.** A central role is what the GATT example needs, and the desktop app
   is a stricter central than a phone: it shows discovery errors, MTU negotiation and CCC writes
   individually.

## What the dongle does not solve

It does not give the EFR32 a PSA crypto library, so it does not unblock `ble_gatt` by itself — see
`BLE-GATT-GAP.md`. It is a *measuring instrument*, not a missing dependency: it tells you whether what
you built works, not how to build what does not link.

## Cautions learned elsewhere in this project

- **The probe is a shared resource.** If the EFR32 is being flashed while the sniffer captures, expect
  the capture to show the reset, not the advertising.
- **A dongle in the same USB hub** sees a much stronger RSSI than the truth. Put them apart, or accept
  that the RSSI is only comparable within one capture.
- **The sniffer firmware replaces the dongle's factory firmware.** Recovery is the same procedure as
  flashing it: the dongle has a button that puts it in DFU mode.
