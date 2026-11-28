# Code written in Portuguese: two counts, and why the first one lied

The project's rule is that code is written in English. Measured on 2026-11-28 -- and the measurement itself is the
first thing to report, because the first version of it certified a false green.

## The count that lied

The first pass counted lines containing a **Portuguese accented character**, and after the translations below it
reported zero for every library and driver source. That was wrong: Portuguese written without accents -- `nao`,
`e'`, `so'`, `esta'`, `nucleo` -- is abundant here, and the regex saw none of it. `src/clock/clocks.rs` was declared
clean while lines 138-144 still read:

    // A ordem importa e nao e' negociavel: ligar (`FORCEEN`), esperar que o cristal estabilize

Both statements were "measured"; the one that mattered was measured by the weaker instrument. That is the failure
this whole repository is built against: a check whose input is not the real thing turns a defect into a certified
one.

## The count that is closer

Counting comment lines that contain **two or more Portuguese function words** (`que`, `para`, `com`, `dos`, `uma`,
`foi`, `esta`, `fica`, `tem`, `antes`, `depois`, `nucleo`, `silencio`, `contador`, `espera`, `volta`, `fase`, ...):

| File | Lines |
|---|---|
| `crates/efr32mg24-rail-sys/examples/rail_ieee802154_tx.rs` | 60 |
| `apps/dosimeter-ble/src/main.rs` | 42 |
| `crates/xiao-mg24-bsp/examples/embassy_driver_systrc.rs` | 20 |
| `crates/efr32mg2x-hal/examples/hal_em2_sleep.rs` | 13 |
| `crates/xiao-mg24-bsp/examples/embassy_task_systrc.rs` | 12 |
| `crates/efr32mg2x-hal/examples/hal_i2c_recovery.rs` | 7 |
| `crates/efr32mg2x-hal/src/cores/sysrtc.rs` | 6 |
| `crates/efr32mg2x-hal/src/sysrtc.rs` | 5 |
| `crates/efr32mg2x-hal/src/clock/clocks.rs` | 5 |
| `crates/efr32mg2x-hal/examples/hal_mic_level.rs` | 5 |
| `crates/efr32mg2x-hal/src/i2c/mod.rs` | 3 |
| `crates/efr32mg2x-hal/examples/hal_imu_read.rs` | 3 |
| `crates/efr32mg24-rail-sys/src/rx.rs` | 1 |
| **Total** | **182** |

A heuristic, not a detector: it will miss a short Portuguese line and it could catch an English line with two
unlucky words. It is labelled as a heuristic because the number it replaces was presented as a fact.

## Translated in this pass

| File | What it held |
|---|---|
| `src/i2c/async_impl.rs` | the `BUSHOLD` hang that cost six flashes, and the **44 647 591** handler entries at `ICSR = 0x82b` |
| `src/sleep.rs` | why EM2 needs an API rather than a `wfi`, and the `EM2DBGEN` consequence |
| `src/clock/clocks.rs` (`start_hfxo`) | the HFXO clock gate: `Error: Exception` 0.16 s after a read without `CMU_CLKEN0.HFXO0` -- **and this file still holds an accent-free second copy of the same reasoning at 138-144, found only by the second count** |
| `examples/hal_i2c_async.rs` | re-measured: `who=0x6a pending=23 irq=12 wfi=12`, `PASS` |
| `examples/hal_sysrtc_wide.rs` | re-measured: `sleep_ms=1000 wakes=10 spin_ms=1000 spin_wall=997`, `PASS` |
| `examples/hal_sysrtc_wrap.rs` | re-measured: `wraps=1 irq=1 before=4294963200 after=0 delta=4096`, `PASS` |
| `examples/hal_em2_sleep.rs` | header and the wake-check comments; **the rest of this file is still Portuguese** |
| `src/cores/sysrtc.rs`, `src/sysrtc.rs`, `crates/embassy-efr32/src/alarm.rs`, `src/blocking/stack.rs`, the radio's two BLE probes, `hal_mic_level.rs`, three lines of `apps/dosimeter-ble` | five small blocks and four short notes |

A translation is not finished when it compiles; it is finished when the instrument still measures the same thing.
The three examples above were booted afterwards and each recorded the same numbers as before.

The async I2C's markers were renamed with them -- `ESPERAS_*`/`ULTIMO_*` to `WAITS_*`/`LAST_*`, `Ate`/`espera` to
`Until`/`until` -- after checking that nothing outside that file referred to them. They are `#[no_mangle]` symbols read
over SWD, so the check mattered.

## The rule that comes out of this

Do not write a language check that looks for accents, and do not report its verdict as complete. Either detect the
language properly, or report the method and its blind spot beside the number -- which is what this file does now.
