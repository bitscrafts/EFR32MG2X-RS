# Code written in Portuguese: counted, and what is left

The project's rule is that code is written in English -- identifiers, comments, commit messages. It is not a style
preference: `deny(missing_docs)` means a module's documentation *is* part of its public surface, and the examples are
how this crate explains its traps to the next person.

Measured 2026-11-28 by counting lines containing a Portuguese accented character, over `crates/`, `libraries/` and
`apps/`:

| File | Lines |
|---|---|
| `crates/efr32mg24-rail-sys/examples/rail_ieee802154_tx.rs` | 70 |
| `crates/xiao-mg24-bsp/examples/embassy_driver_systrc.rs` | 21 |
| `crates/efr32mg2x-hal/examples/hal_sysrtc_wrap.rs` | 18 |
| `crates/xiao-mg24-bsp/examples/embassy_task_systrc.rs` | 13 |
| `crates/efr32mg2x-hal/examples/hal_em2_sleep.rs` | 11 |
| **Total** | **133** |

**Every library and driver source is clean.** What is left is documentation inside examples -- which is the worst
place to leave it, because an example's header is where this repository records the measurement that makes the code
look the way it does. Those 148 lines are worth a deliberate pass: bulk substitution would cost the measurements that
make the files worth reading.

The two examples translated here were re-measured on the board afterwards, because a translation that changes a
number is a defect: `i2c-async: who=0x6a pending=23 irq=12 wfi=12` and
`wide: sleep_ms=1000 wakes=10 spin_ms=1000 spin_wall=997 c0=138 c1=18946342 wraps=0`, both `status=PASS`.

## Done in this pass (2026-11-28)

| File | Lines before | Now |
|---|---|---|
| `crates/efr32mg2x-hal/examples/hal_sysrtc_wide.rs` | 37 | 0 |
| `crates/efr32mg24-radio/examples/ble_beacon.rs` | 3 | 0 |
| `crates/efr32mg24-radio/examples/ble_boot_probe.rs` | 2 | 0 |
| `crates/efr32mg2x-hal/examples/hal_mic_level.rs` | 5 | 0 |
| `apps/dosimeter-ble/src/main.rs` | 5 | 0 |
| `crates/efr32mg2x-hal/examples/hal_i2c_async.rs` | 30 | 0 |
| `crates/efr32mg2x-hal/src/i2c/async_impl.rs` | 68 | 0 |
| `crates/efr32mg2x-hal/src/sleep.rs` | 19 | 0 |
| `crates/efr32mg2x-hal/src/clock/clocks.rs` | 19 | 0 |
| `crates/efr32mg2x-hal/src/cores/sysrtc.rs`, `src/sysrtc.rs`, `crates/embassy-efr32/src/alarm.rs` | 5 | 0 |
| `crates/efr32mg24-bluetooth-sys/src/blocking/stack.rs` | 1 | 0 |
| the generated module READMEs | inherited | 0, after regenerating |

Translated with the numbers intact: the HFXO clock gate (`Error: Exception` 0.16 s after a read without
`CMU_CLKEN0.HFXO0`), the wrap arithmetic that must return `2^32 + 5` and the 36.4 h a wait straddling the wrap measures
too little, the `BUSHOLD` hang that cost six flashes and the **44 647 591** handler entries at `ICSR = 0x82b`.

The async I2C's markers were renamed with them -- `ESPERAS_*`/`ULTIMO_*` to `WAITS_*`/`LAST_*`, `Ate`/`espera` to
`Until`/`until` -- after checking that nothing outside that file referred to them. They are `#[no_mangle]` symbols read
over SWD, so the check mattered.

## Why a count, and not just the rule

A rule that lives only in a document gets broken, which this workspace has paid for more than once. A count with a
date is falsifiable: anyone can re-run one command and see whether the debt shrank.

    grep -rlc 'ã\|õ\|ç\|ê\|á\|í\|ó\|ú\|à' --include=*.rs crates/ libraries/ apps/
