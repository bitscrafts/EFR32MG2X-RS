# burtc

`BURTC`: the backup counter, the one that keeps time when the rest of the part is off.

## What it is

`BURTC`: the backup counter, the one that keeps time when the rest of the part is off.

`SYSRTC` is the counter this crate uses for ordinary timekeeping, and it stops in EM4. `BURTC` lives in the backup
domain, runs from the low-frequency branch, and is the counter a product reads after a hibernate to learn how long
the part was asleep. It is a separate block with its own clock mux (`CMU_EM4GRPACLKCTRL`), its own bus gate
(`CMU_CLKEN0.BURTC`) and its own enable/start sequence -- a driver copied from `sysrtc` compiles and counts nothing.

# What this driver encodes

The sequence is the one measured on the part, and each step is a line of [`Burtc::new`]:

1. **`LOCK` before `CFG`.** A wrong key leaves the configuration registers writable-or-not without saying so; the
unlock key is written first so that the `CFG` write that follows is not silently dropped.
2. **The bus gate is not the clock.** `CMU_CLKEN0.BURTC` lets the module answer on the bus; the counting clock is
`EM4GRPACLK`, selected separately. With the gate set and no source, the registers read back and the counter never
moves -- exactly the state [`remove_source`] puts the block in, and the control the instrument uses.
3. **`EN`, then `CMD.START`.** `CMD.START` is a command that crosses into the counter's clock domain, and
`SYNCBUSY.START` is the part of the crossing that can be seen. The driver waits for it, bounded, and then for
`STATUS.RUNNING`.
4. **The backup domain outlives a core reset.** The block can arrive already enabled and running from the firmware
that ran before, and `CFG` is not writable in that state -- the first boot of this driver died on a precise bus
fault at `BURTC_NS + 0x08` (`CFSR=0x8200`, `BFAR=0x50064008`). `new` and [`Burtc::set_prescaler`] therefore drop
`EN`, wait out `EN.DISABLING`, write `CFG`, and enable again.
5. **`CNTPRESC` is a power of two**, `0` = divide by 1 through `15` = divide by 32768. Written and read back:
[`Burtc::set_prescaler`] refuses rather than returning with a field that did not take.
6. **`COMPTOP` is left clear**, so the top is the whole 32-bit counter and nothing wraps early. The compare channel
is not driven here: there is no instrument for its interrupt in this crate, and a driver without one is a claim.

# What this driver does not do

EM4 entry and the retention of the counter across it belong to `emu`; the counter is only read here. The overflow
interrupt is not exposed for the same reason the compare channel is not.

The measured record, the control and what each run rules out are in `../../../../docs/../../../../docs/04-reference/BURTC-2026-11-27.md`.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Burtc` | struct | The backup counter, owned by one caller. |
| `Error` | enum | — |
| `SOURCE_HZ` | const | Ticks per second of the source this driver selects: LFRCO, 32.768 kHz, the same branch `SYSRTC` runs from. |
| `clock_readback` | fn | — |
| `counter` | fn | — |
| `flags` | fn | — |
| `is_running` | fn | — |
| `prescaler` | fn | — |
| `remove_source` | fn | Remove the source from `EM4GRPACLK` (`CLKSEL = 0`), leaving the module enabled and its registers reachable. |
| `select_lfrco` | fn | Point `EM4GRPACLK` at LFRCO, the 32.768 kHz branch `SYSRTC` also runs from. |
| `select_ulfrco` | fn | Point `EM4GRPACLK` at ULFRCO, the 1 kHz branch that keeps running with the rest of the part asleep. |
| `set_prescaler` | fn | Set the prescaler (`0` = divide by 1, `15` = divide by 32768) and check that the field took it. |
| `start` | fn | Start a counter that was stopped, with the same wait as [`Burtc::new`]. |
| `status` | fn | — |
| `stop` | fn | Stop the counter, waiting, bounded, for `STATUS.RUNNING` to go low. |
| `syncbusy` | fn | — |
| `tick_hz` | fn | — |

## Registers it touches

`BurtcNs`, `CmuS`

## What was measured

```
    burtc: ok=1 run=1 pre=0 d0=32901 s0=32901 r0=1000 pd=3 d1=4113 s1=32901 r1=125 ctl=0 cs=32902 bk=16448 bs=16451 rb=999 if=0x00000000 st=0x00000001 sb=0x00000000 ck=0x00000101
    burtc: ok=1 run=1 pre=0 d0=32900 s0=32900 r0=1000 pd=3 d1=4112 s1=32899 r1=124 ctl=0 cs=32899 bk=16446 bs=16450 rb=999 if=0x00000000 st=0x00000001 sb=0x00000000 ck=0x00000101
```

Controls and the full story: [BURTC-2026-11-27.md](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/BURTC-2026-11-27.md)

## What is not proven

From the record's own list: two counters agreeing on one LFRCO is not an absolute frequency -- the crystal was not
used; `CNT` retention across an EM4 entry is `emu`'s subject and has no instrument here; the compare/overflow path is
not armed (`COMPTOP` clear, `IF` only read); and the prescaler's phase at the moment a change takes effect is not
visible in a one-second window.

## Using it

See `examples/hal_burtc_backup.rs`, `examples/hal_cmu_sources.rs`
