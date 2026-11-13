# One radio, three protocols

Constraint set by the user on 2026-11-13, while Bluetooth was being brought up:

> *"levar sempre em consideração a futura implementação do zigbee e do thread!"*

This is a design constraint on **today's** code, not a backlog item. The EFR32MG24 has one 2.4 GHz radio,
and Zigbee and Thread are 802.15.4 on that same radio. Anything built for Bluetooth that assumes it owns the
radio, the timer, the flash store or the crypto will have to be taken apart later.

## What is already true, measured

```
$ nm -g librail_efr32xg24_gcc_release.a | grep -c RAIL_IEEE802154
81
```

The RAIL blob already linked for Bluetooth carries **81 `RAIL_IEEE802154_*` symbols** — the same binary
serves both protocols. There is nothing to download for 802.15.4 at the link-layer level.

Also already in hand:

- an 802.15.4 `rail_config.c` (base 2450 MHz, 1 MHz spacing, channels 11-26) fetched earlier by mistake and
  kept in `/projects/_tmp/efr32mg24-rs/rail_config.c` — that accident is exactly the artifact 802.15.4 needs;
- the vendored PA curves cover the 2.4 GHz band for both protocols (`SL_RAIL_UTIL_PA_SELECTION_2P4GHZ`,
  `SL_RAIL_UTIL_PA_SELECTION_SUBGHZ = NONE`, which is correct for this part);
- `RAIL_BLE_*` (54 symbols) and `RAIL_IEEE802154_*` (81) live in the same archive, so a protocol switch is a
  configuration question, not a link question.

## The rules that follow, applied from now on

**1. The radio sits behind a protocol-neutral boundary.** The HAL exposes tuning, transmit and receive. BLE
and 802.15.4 are protocol modules above it. A HAL function that mentions `BLE`, `advertising`, `access
address` or `connection` is in the wrong layer.

**2. Channel numbering is protocol-specific and must not leak upward.** Learned the hard way on
2026-11-13: RAIL's channel argument is the **physical** channel, and BLE's advertising channels are
`(0,37)`, `(12,38)`, `(39,39)`. 802.15.4 numbers channels 11-26 on a different grid entirely. So the
logical-to-physical mapping belongs to the protocol module, never to a caller and never to the HAL.

**3. Shared services are built once, protocol-agnostic.** These are needed by Bluetooth, Zigbee and Thread
alike:

| Service | Why all three need it | Consequence today |
|---|---|---|
| timer (`SYSRTC`) | every stack schedules its own events | implement in the **HAL**, not as `sl_sleeptimer` glue |
| persistent store (`nvm3`) | pairing data, network keys, frame counters | one flash region, one API, three consumers |
| AES core | BLE CTR; 802.15.4 CCM* | `libraries/aes-ctr-radio` holds the block cipher; CCM* is a mode on top, not a second AES |

**4. One RAIL state buffer is a single-protocol assumption.** `RAIL_AddStateBuffer` takes one per protocol
instance; multiprotocol operation needs one per protocol. Do not hard-code a single global.

**5. RAM decides what can coexist, and it decides late.** This part has 256 KB. The Bluetooth stack plus an
802.15.4 stack do not fit comfortably together with a GATT database. Whether the product time-slices the
radio or runs one protocol per firmware image is a decision for later — but the boundary above is what keeps
that decision open.

## The question to ask of every new piece

> *Would this be identical for 802.15.4?*

- Yes → it belongs in the HAL or in `libraries/`, and it must be written so Zigbee and Thread can use it
  unchanged.
- No → it is a protocol module, and it must not be reachable from the HAL, the BSP or the other protocol.

## What this changes in the work in flight

- The sleeptimer about to be written must be a **HAL timer** over SYSRTC, not a `sl_sleeptimer` shim. If it
  were written as vendor glue, Zigbee would need a second timer driver for the same peripheral.
- The AES work stands as is: a block cipher plus CTR is a mode, and CCM* (802.15.4) is another mode over the
  same S-box and key schedule.
- The BLE examples stay in the BLE layer. A "radio" example that only works for one protocol is a BLE
  example with a misleading name.
