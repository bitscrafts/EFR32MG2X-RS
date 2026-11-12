# Can the nRF52840 test this board's radio? Measured, 2026-11-12

Short answer: **yes for 802.15.4, not for BLE without a decision** — and neither today, because the
module is not enumerating. Everything below was measured here, not assumed.

## 1. The module is not on the bus

```text
$ ssh <flash host> lsusb
... 2886:0062 Seeed Technology Co., Ltd. Seeed Studio XIAO MG24 (Sense) CMSIS-DAP
... 05ac:1005 / 05ac:021e Apple keyboard hub
... 0bda:b851 Realtek WLAN
/dev/ttyACM0        <- the XIAO
```

No 1915:xxxx (Nordic), no 1209:xxxx (pid.codes / UF2 bootloader), no extra ttyUSB. Three candidates,
in order of likelihood:

1. **The board is in its bootloader and the bootloader does not present a serial port.** A Pro Micro
   nRF52840 with a UF2 bootloader appears as a **mass-storage volume** (`/dev/sdX`, `INFO_UF2.TXT`),
   not a tty. Check `lsblk` and `ls /media/*/*UF2*`.
2. **It is a bare module** (no USB), wired to something else — then it needs SWD, and a probe.
3. It is plugged into another machine.

## 2. There is no BLE radio API in the Rust HALs this stack uses

Read from the crate sources, not from memory:

| Crate | radio modules present | BLE mode |
|---|---|---|
| `embassy-nrf 0.11` | `radio/ieee802154.rs`, `radio/mod.rs` | **absent** |
| `nrf52840-hal 0.19` (→ `nrf-hal-common`) | `ieee802154` | **absent** |

Both expose 802.15.4 and nothing else. That is the whole constraint: a BLE advertiser built from the
Rust HALs means writing the RADIO peripheral in BLE 1M mode by hand (PCNF0/PCNF1, CRCINIT/CRC polynomial,
`MODE = BLE_1Mbit`, whitening) — around 200–300 lines of register code, and it is the kind of code that
only a board can prove.

## 3. What that project already has, and what it does not

`/projects/promicro-nrf52840`:

- **Works here**: `cargo build -p promicro-firmware --features poc1` succeeds with
  `thumbv7em-none-eabihf`, which is installed.
- **Proven on air, per its own record**: PoC-1 (802.15.4 TX/RX between two boards) and PoC-2 (ACK
  turnaround measured at 38 µs, a lower bound, against the 192 µs deadline).
- **Not a BLE instrument**: its `docs/radio.md` decides BLE equals SoftDevice blob + async runtime, and
  both are excluded by that project's rules.

`/projects/nrf52-lab`:

- **Does not build here today**: `nrf52-blink` fails with `#[panic_handler] function required`. Its
  `build-uf2.sh` also targets `thumbv7em-none-eabi`, which is **not installed** (only the `eabihf`
  variant is). So it is not a working starting point until those two are fixed.

## 4. The four ways to get a BLE peer, with their real costs

| Option | What it gives | Cost / risk |
|---|---|---|
| **Phone + nRF Connect** | sees advertising, payload, RSSI; can connect and subscribe | zero; available now |
| **nRF52840 dongle + Nordic sniffer hex** | per-packet truth: address, AD structures, CRC | flash a prebuilt image (not Rust); the vendor lists the nRF52840 Dongle as supported |
| **nrf-softdevice crate** (S140 blob) | a full BLE peer in Rust: advertise, scan, connect | vendor blob + async runtime — contradicts that project's rules, so it belongs in a lab crate |
| **Raw RADIO in BLE mode** | a peer with no blob, no runtime, in that project's style | ~200–300 lines of audited `unsafe`; needs the board to test, so it cannot be written ahead of the hardware |

## 5. What I would do, in order

1. **Find the board.** `lsblk` for a UF2 volume, or replug it and watch `dmesg -w`. Nothing else can be
   decided until it enumerates.
2. **Meanwhile the phone is the instrument**, and it is enough for what exists today: it proves
   `ble_beacon`, `ble_advertise_custom`, `module_ble_motion`, `module_ble_fall` and `base_ble_env` — all
   of which carry a payload a scanner app can display.
3. **For the 802.15.4 direction the nRF52840 is already proven** — but the EFR32 side is not, because
   `RAIL_Init` needs emlib (see `RADIO-NOTES.md`). That is the pairing worth doing *after* emlib: two
   boards that were both proven separately, finally talking.
4. **For BLE on the nRF52**, prefer the sniffer hex over writing a BLE radio from scratch: it answers
   "is the EFR32 transmitting what I think it is", which is the question that has been answered so far
   only by inference.

## 6. The honest summary

The nRF52840 is the right *class* of instrument — an independent receiver. It is not a free one: in
Rust, on these HALs, BLE is either a vendor blob or hand-written radio registers. The cheapest path to
per-packet truth today is Nordic's sniffer firmware on a dongle; the most interesting path, for this
project's discipline, is the audited raw-radio advertiser — but it must be written *with* the board in
hand, not before.
## Update, 2026-11-12 evening: two cables, two ports, same failure

The module was moved to a second port and then a second cable. The failure is unchanged:

```
usb 1-3: new full-speed USB device number 26 using xhci_hcd
usb 1-3: unable to read config ind...[truncated]