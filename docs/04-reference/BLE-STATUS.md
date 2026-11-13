# Bluetooth: the state, measured

2026-11-13. Every number here came from the board or from a build, not from a plan. `scripts/ble-examples-status.sh`
reproduces the table; `scripts/ble-run-and-report.sh <pkg> <example> [features]` flashes one and prints what the
mailbox and RAIL's assert slot say.

## The one paragraph

**The radio works. The stack links and flashes but does not boot.** Advertising is proven on hardware through the
RAIL path alone — the board is heard by the host's controller, by BlueZ, and accepts a link-layer connection.
Everything that goes through the vendored Bluetooth stack stops at one named defect: RAIL asserts
`RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (code 66) because nothing has told it the crystal is 39 MHz, and the default
implementation of `RAILCb_AssertFailed` is an infinite loop, so it presents as a hang.

## What is proven on the board

| Path | Evidence |
|---|---|
| RAIL-only advertising (`rail_ble_tx`) | `ADV_IND`, address `C0:DE:00:00:00:01` (static random), `Flags 0x06`, `Name (complete): xiao-mg24`, RSSI −71…−73 dBm, 130 reports in 14 s at 20 dBm. Accepted by BlueZ (`Device C0:DE:00:00:00:01 xiao-mg24`) and by the link layer (`Connected: yes` → `disconnected reason 0`, since the firmware has no link layer of its own). |
| RAIL capabilities (`rail_caps`) | `ble1m=0 vit=1 2m=0 coded=1` — the non-Viterbi 1 Mbps PHY is NULL in this blob. |
| Timer and its interrupt (`ble_boot_probe`, `hal_timer_compare`) | compare fires exactly one count after the deadline (`late=1`); a scheduled timer runs its callback from `SYSRTC_APP_IRQHandler` (`a=1 b=1`). |
| Crypto under the stack | AES-CMAC against RFC 4493 and CCM\* against RFC 3610 (ciphertext *and* tag), with tamper cases. |
| The stack's link | zero undefined symbols, 130 KB, `load_min=0x08006000`, flashes with read-back verification. |

## What does not work, and exactly where

```
module_ble_beacon   -> RAIL assert 66, handle 0x20005298, count 1     (BSP level)
ble_boot_probe      -> RAIL assert 66, handle 0x20005290, count 1     (radio level)
```

Same code, two levels: the wall is not a BSP or wrapper problem. The stack's initialisation reaches RAIL, RAIL
refuses to configure a radio whose crystal is unknown, and the assertion has nowhere to go.

**The fix is identified and the data is in the tree**: `RAIL_ChannelConfig_t.xtalFrequencyHz` is 39 MHz in the SDK's
own generated table for this part (`rail-sys/vendor/ble_config/`, Zlib). It is vendored but not compiled, because
including `rail_types.h` in our own translation unit duplicates symbols the RAIL blob already has. The route is to
emit it as Rust statics — a `uint32_t` array, three channel entries, and a six-field struct.

## The examples, all thirteen

`bash scripts/ble-examples-status.sh` builds each in turn and prints size and load range. Measured 2026-11-13:

| Package | Example | Features | State |
|---|---|---|---|
| `efr32mg24-rail-sys` | `rail_ble_tx` | none | **runs, advertising proven at three layers** |
| `efr32mg24-rail-sys` | `rail_ble_rx` | none | builds; receive path not measured this session |
| `efr32mg24-rail-sys` | `rail_caps` | none | **runs**, reports the PHY capabilities |
| `efr32mg24-radio` | `ble_beacon` | stack | builds, flashes, blocked at assert 66 |
| `efr32mg24-radio` | `ble_advertise_custom` | stack | builds, flashes, blocked at assert 66 |
| `efr32mg24-radio` | `ble_scanner` | stack | builds, flashes, blocked at assert 66 |
| `efr32mg24-radio` | `ble_gatt` | ble,gatt,stack | builds, flashes, blocked at assert 66 |
| `efr32mg24-radio` | `ble_boot_probe` | none | **runs**; proves the timer and names the assert |
| `efr32mg24-bluetooth-sys` | `bt_version` | none | builds; the BGAPI call needs a started stack, so it cannot report yet |
| `xiao-mg24-bsp` | `module_ble_beacon` | stack | builds, flashes, blocked at assert 66 (measured) |
| `xiao-mg24-bsp` | `module_ble_motion` | sense,stack | builds, flashes, blocked at assert 66 |
| `xiao-mg24-bsp` | `module_ble_fall` | sense,stack | builds, flashes, blocked at assert 66 |
| `xiao-expansion-base-bsp` | `base_ble_env` | stack | builds, flashes, blocked at assert 66 |

All thirteen build for `thumbv8m.main-none-eabihf`. None loads below `0x08006000`.

## The honest summary of the distance

- Advertising on the air: **done, measured, reproducible.**
- A connectable peripheral with a GATT service: **not done.** It needs the stack to boot, which needs the RAIL
  channel configuration delivered from Rust, which is the next piece of work and is not a research question — the
  data is vendored and the shape of the code is known.
- Bonding, pairing, security: **not started**, and `psa_generate_random` deliberately refuses because this part
  declares no TRNG. That refusal is the first thing a security feature will hit, and it is documented rather than
  papered over.
