# gpcrc

The general-purpose CRC accelerator, driven so that its answer can be compared with an independent one.

## What it is

The general-purpose CRC accelerator, driven so that its answer can be compared with an independent one.

## What this is for

A CRC is the cheapest way for firmware to notice that an image, a configuration blob or an over-the-air frame arrived
corrupted, and this device has an accelerator for it: `GPCRC`.

## Which CRC-32 this actually is — measured, not read

The reference manual describes the accelerator as MSB-first with polynomial `0x04C11DB7`, and this module's first draft
said so. The part disagreed: `hal_gpcrc` read `0x340BC6D9` for `b"123456789"` with seed `0xFFFFFFFF`, and
`!0x340BC6D9` is exactly `0xCBF43926` — the published check value of the **reflected** CRC-32, the one `zlib`, PNG and
Ethernet use. So the engine computes the reflected convention and stops one step short: the final exclusive-or is the
caller's.

Recorded in the README as `PASS gpcrc: poly=crc32 raw=340bc6d9 zlib=cbf43926 want=cbf43926 diff=1 seed=1`.

## The order the part expects

Measured 2026-11-25 on the XIAO MG24 (`hal_gpcrc`, record in the README): enable, then the polynomial and byte mode,
then the seed through `INIT` followed by `CMD.INIT` — the write to `INIT` alone does not load the engine, and a CRC
seeded with the previous seed is a CRC of the wrong message. Bytes then go to `INPUTDATABYTE`, one per call, and the
result is read from `DATA`.

## What it does not do

No DMA, no interrupt, no 16-bit polynomial. Those are in the PAC and in the reference manual; adding them here without
an instrument that measures them would be the kind of code this crate's README calls "compiled and unmeasured".

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Gpcrc` | struct | The CRC accelerator. |
| `crc` | fn | Reset, feed and read, which is what every caller actually wants. |
| `poly` | fn | — |
| `reset` | fn | Load a seed into the engine. |
| `result` | fn | — |
| `seed_changes_the_result` | fn | — |
| `update` | fn | Feed bytes through the engine. |

## Registers it touches

`GpcrcNs`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

See `examples/hal_gpcrc.rs`
