# Spec B10: 802.15.4 in the radio crate — one RAIL, a second protocol

**Status: spec.** Written 2026-11-18, after measuring the vendor API. No code yet; the full
Zigbee/Thread implementation is explicitly deferred, as the operator asked ("não descartar a
implementação completa para o futuro").

## Overview

The radio on this part speaks BLE through RAIL, and the same RAIL speaks IEEE 802.15.4 — the
protocol headers are already in the tree:

```
crates/efr32mg24-rail-sys/vendor/rail/protocol/ieee802154/rail_ieee802154.h
    RAIL_Status_t RAIL_IEEE802154_Init(RAIL_Handle_t, const RAIL_IEEE802154_Config_t *);   :508
    RAIL_Status_t RAIL_IEEE802154_Config2p4GHzRadio(RAIL_Handle_t);                        :524
    (plus AntDiv/Coex/Fem/2Mbps variants, :540-:660)
```

**Corrected 2026-11-18 by the operator: the radio crate is common to BLE, Zigbee and Thread, and it
already exists.** The vendor treats the protocols as modules of one RAIL:

```
crates/efr32mg24-rail-sys/vendor/rail/protocol/
    ble/  ieee802154/  sidewalk/  wmbus/  zwave/
crates/efr32mg24-rail-sys/src/
    lib.rs  ble_config.rs          <- one of the five is bound; the other four are not
```

So this item does not create a crate. It adds a second protocol to the crate that already speaks the
first: `efr32mg24-rail-sys/src/ieee802154.rs`, binding the two entry points, with an example that
brings the radio up in 802.15.4 mode and transmits one frame. Zigbee and Thread then stand on
`efr32mg24-radio`, where the BLE protocol logic lives today — one floor, three protocols above it.

## Measured before writing a line

```
vendor header     rail_ieee802154.h:508 (Init), :524 (Config2p4GHzRadio) -- 7 fields in the config
the blob already exports both symbols:
    nm -g librail_efr32xg24_gcc_release.a
      00000001 T RAIL_IEEE802154_Config2p4GHzRadio
      00000001 T RAIL_IEEE802154_Init
rail-sys today    src/lib.rs is 80 lines and both `extern "C" {}` blocks are EMPTY;
                  src/ble_config.rs is a data table, not an FFI call
build.rs          links the pinned Gecko SDK 4.4.5 archive (sha256 above it)
```

This is the whole reason the item is small: the vendor blob already contains the 802.15.4 stack, the
header is in the tree, and the two symbols are exported. There is nothing to vendor, nothing to
download, and no licence question beyond the one the BLE side already answered.

**The feature already exists.** `crates/efr32mg24-radio/Cargo.toml` declares
`rail = []  # RAIL (proprietary and 802.15.4) -- enabled once emlib and a radio config are vendored`.
So the exit criterion is not "a new crate compiles" but "`--features rail` compiles and transmits one
frame".

## What makes it more than an afternoon, honestly

Three things, in order of risk:

1. **The struct layout.** Seven fields, `#[repr(C)]`, and the header may use a union or a bitfield for
   the PTI radio config. This is where the check earns its keep: field by field, compared by script.
2. **The handle.** `RAIL_Init` must run before `RAIL_IEEE802154_Init`, with a RAIL config built for
   802.15.4 rather than for BLE. The BLE path in this tree builds its config as a table; the 802.15.4
   path needs the analogous table and nobody has written it.
3. **The interrupt.** Receiving a frame is an interrupt, and the callback plumbing is the part this
   project has got wrong before (`abort_session`, the timer callback contract). Transmitting one frame
   avoids it; receiving is the next item.

## Requirements

**R1 -- the existing radio crate gains a second protocol, and nothing speculative.**
`efr32mg24-rail-sys` exposes `ieee802154::init(handle, &Config) -> Status` and
`ieee802154::config_2p4ghz(handle) -> Status`, beside the BLE bindings it already has, over the same
vendor blob.
Example: `cargo check -p efr32mg24-rail-sys --target thumbv8m.main-none-eabihf` is silent.

**R2 -- the config struct matches the vendor header field for field.**
`RAIL_IEEE802154_Config_t` is mirrored as `#[repr(C)]` with the same field order and the same types,
verified against the header by a script and not by eye.
Example: `python3 scripts/check-ieee802154-config.py` compares field names in order and prints
`OK: N fields match rail_ieee802154.h`, or names the first difference.

**R3 -- a frame can be transmitted, and that is the proof.**
An example brings the radio up, sets a 802.15.4 channel, and transmits one frame with a known
payload.
Example on the board (through `scripts/flash.sh` with `--confirm` and the `0x08006000` guard): a
second receiver, or the same board in RX, reports the payload.

## Files

- `crates/efr32mg24-rail-sys/src/lib.rs`
- `crates/efr32mg24-rail-sys/src/ieee802154.rs`
- `crates/efr32mg24-rail-sys/examples/tx_one_frame.rs`
- `crates/efr32mg24-rail-sys/build.rs`
- `scripts/check-ieee802154-config.py`

## TDD Contract

| Test Name | Input | Expected Output |
|---|---|---|
| the_config_layout_matches_the_header | `RAIL_IEEE802154_Config_t` in the header | every field name, in order, with the same size and alignment |
| a_missing_field_is_caught | a mirror with one field removed | `FAIL`, naming the field and its offset |
| a_reordered_field_is_caught | two fields swapped | `FAIL`, naming both |
| the_status_maps_to_the_vendor_codes | 0, 1, 2 from `RAIL_Status_t` | `Ok`, `InvalidParam`, `Unsupported`, and no unknown code silently accepted |

## Exit Criteria

- [ ] `python3 scripts/check-ieee802154-config.py`
- [ ] `cargo check -q -p efr32mg24-rail-sys --target thumbv8m.main-none-eabihf`
- [ ] `cargo check -q -p efr32mg24-radio --target thumbv8m.main-none-eabihf` (nada partido acima)
- [ ] `bash scripts/gate.sh`

## Guardrails

- Do NOT write a MAC, a Zigbee stack or a Thread stack here. One frame transmitted is the whole item.
- Do NOT guess a field of `RAIL_IEEE802154_Config_t`: read the header, and let the check compare.
- Do NOT flash without `--confirm` and without the `load_min >= 0x08006000` guard.
- Do NOT publish: the blob licensing for 802.15.4 is a separate question from the BLE one.
- Do NOT delete the deferred design: Zigbee and Thread stay on the roadmap, and this item exists so
  they have a floor to stand on.

## What is deliberately out of scope, and where it goes

```
B11  the MAC in pure Rust: CSMA/CA, ACK, retries, sequence numbers -- host-testable, `cores/` style
     Zigbee: NWK + APS + ZCL, or EmberZNet as a blob
     Thread: 6LoWPAN + MLE + CoAP, or OpenThread as a blob
```

The evidence for the order: `embassy-nrf` ships 802.15.4 PHY/MAC and nothing above it, and
`nrf-802154` wraps Nordic's blob for the rest. Nobody in Rust has a full Zigbee or Thread stack, so
the floor is the honest first step and the ceiling is a decision to be taken with numbers.
