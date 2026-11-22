# HAL: is it complete, and is it robust enough for a product?

**Provenance.** An independent review, dispatched 2026-11-22 at commit `f49e680`, run by a frontier model
(`claude-opus-5-5`) in read-only plan mode with the whole tree available and 110 turns. Brief:
`_tmp/opus-hal-robustness/brief.md`. Session: `37ee7a89-73ba-466b-bea1-06755fb8699d`. The brief carried the
author's own measurements and asked explicitly to attack them and to say where it was wrong.

**Why it is in the repository.** It was written to a plan file outside git, which is the same mistake recorded in
`HAL-OPUS-ANALYSIS-2026-11-18.md`: an analysis that lives in a scratch directory is an analysis nobody reads twice.

## What the author verified personally, the same hour (not taken on trust)

```
gate.sh:415        cargo tree -p <pkg> --edges normal,build -e no-dev  ->  rc=101
                   "normal dependency kind cannot be mixed with no-dev"
                   stderr discarded, grep finds nothing, the else branch prints
                   "pass: <pkg> no hardware dependency" -- for every crate, always.
                   CONFIRMED: the gate's own guard is dead. This is a false green
                   inside the gate, i.e. the HARD RULE 0b failure class.
dma/mod.rs:431     stop() writes 0 to CHEN.                        CONFIRMED in code
i2c/async_impl.rs  Ate::poll has no counter: it polls, writes markers, wakes and
                   returns Pending forever.                         CONFIRMED in code
i2c/mod.rs:618     the NACK clear inside the shared path uses I2c0S::ptr() hardcoded,
                   so I2C1 clears I2C0's flag.                      CONFIRMED in code
```

## Where the brief was wrong, and it was the author's error

| the brief said | measured |
|---|---|
| ~8,300 lines | **11,870** — the brief counted only `src/<dir>/`, missing the root modules (`board.rs`, `pin.rs`, `pwm.rs`, `microphone.rs`, `buzzer.rs`, `dwt.rs`, `alias.rs`, `lib.rs`) |
| 8 error enums | **21** — same counting error, plus enums not named `Error` |
| 56 panics in driver code | **4 outside tests** — the brief matched the inline `#[cfg(test)]` modules in the same files |
| 111 `unsafe` | **181 outside tests**, 71 with a SAFETY comment |

The pattern in those four rows is worth naming: every one is a grep that measured the shape of the text rather than
the thing asked about. It is the same defect as a gate check that passes while the live path is broken, one level
up -- in the analysis instead of in the code.

---

# mg24 HAL: completeness and robustness review, 2026-11-22

**Provenance.** Independent read-only review by Claude Opus 5.5, commit `f49e680`, 2026-11-22. What I ran is marked
**ran**: host `cargo check`/`cargo test` of the HAL, greps, `cargo tree`, SVD lookups, RM text pulled out of
`docs/vendor/efr32xg24-rm.pdf` with pypdf. What I only read is marked **read**. Nothing was flashed.

## 0. Where the brief is wrong (measured)

| brief says | measured | how |
|---|---|---|
| 16 modules, ~8 300 lines | 16 directories **plus 11 top-level modules** (`board pin pwm sysrtc microphone buzzer route sleep dwt alias` + lib), **11 870 lines** | `find src -name '*.rs' \| xargs wc -l` (ran) |
| 8 error enums; dma, clock, sensors, mvp, bus have none | **21** `pub enum *Error`. `DmaError`, `ClockError`, `SensorError`/`ImuError`, `mvp::Error` and `BusError` all exist. The real gap is a different one: **declared errors that hardware never produces** (F7) | `grep -rnE 'pub enum [A-Za-z]*Error'` (ran) |
| 11 unbounded waits | 11 confirmed, **plus** `usart/mod.rs:156,213` (2), **plus the async I2C future, which has no bound at all** (F3), plus 5 bounded waits that time out **silently** (F9) | grep + reading each `while`/`loop` (ran/read) |
| 56 panics in driver code | **4 outside `#[cfg(test)]`**: `pin.rs:46,69` (`debug_assert!`), `clock/clocks.rs:172` (`assert!`, F11), `dma/mod.rs:304` (`assert_eq!`, a layout check). The other 149 hits are tests. Not counted by your grep: **implicit** panics, e.g. division by zero in `timer/mod.rs:151,162` when `frequency == 0` | script that splits each file at `cfg(test)` (ran) |
| 111 `unsafe` sites | **181** outside tests, 71 with a `SAFETY` comment within 4 lines. No SAFETY at all in `mvp` (17), `pwm` (8), `spi` (7), `pin` (5), `usart` (4). Most are `w.bits()`, which is harmless. The risky ones are the peripheral `steal()` calls (F10) | script (ran) |
| 58 host tests | 58 confirmed. **None of them reaches `dma/mod.rs`, `timer`, `spi/mod.rs` or `i2c/mod.rs`**. They live in `cores/*`, `alias.rs`, `route.rs`, `rtc/pcf8563.rs` | `cargo test -p efr32mg2x-hal --target x86_64-unknown-linux-gnu --lib` → `58 passed` (ran) |
| 1 interrupt unmasked | Confirmed: `sysrtc.rs:496` is the only `NVIC::unmask` | grep (ran) |
| (implicit) the HAL can't build on the host | **It does build.** `cargo check -p efr32mg2x-hal --target x86_64-unknown-linux-gnu --no-default-features --features mg24` → `Finished`. The comment at `cores/spi.rs:4` ("cannot be compiled for the host (it contains ARM asm!)") is stale | ran |
| layering: the BSP never names the PAC | `xiao-mg24-bsp/Cargo.toml:19` declares `efr32mg2x-pac` as a dependency. The source doesn't use it today, but the manifest allows it | read |

## 1. Robustness findings (most severe first)

### F1 — `dma::Channel::stop()` is a no-op, so every guard that relies on it is broken
- **where:** `dma/mod.rs:431-437` (`ldma.chen().write(|w| w.bits(0))`); callers `dma/mod.rs:456-459` (`end_transfer`) and `cores/dma.rs:67,77` (`Transfer::drop`)
- **what:** RM §25.7.10 LDMA_CHEN: *"Setting one of these bits will enable the respective DMA channel, writing zeros has no effect."* The disable register is `CHDIS` (§25.7.11, at offset 0x028, which the PAC exposes). So `stop()` leaves the channel running. The `Channel` trait in `cores/dma.rs:26` promises "stop the channel and **wait until it has stopped writing**". The HAL neither stops the channel nor waits on `CHBUSY`.
- **why it matters:** this is a soundness bug reachable from safe code. When a `Transfer` is dropped, its borrow ends but the LDMA keeps writing into that memory. In circular mode it never stops. The host tests (`a_transfer_starts_and_is_stopped_when_it_is_dropped`) pass because the mock's `stop()` only increments a counter. That is the project's rule 0b exactly: the test passes while the live path is broken.
- **severity:** critical
- **fix:** `ldma.chdis().write(|w| unsafe { w.bits(1 << N) })`, then a bounded wait on `CHBUSY & (1<<N) == 0`, then `compiler_fence(SeqCst)`. Add a board check: start a circular transfer, drop the guard, read `CHSTATUS`, and require the bit to be clear.
- **confidence:** high. I read the RM and the SVD (`CHEN` is `write-only`, `CHDIS` exists). Not run on the board.

### F2 — `Channel::transfer` returns `Err(Timeout)` with the channel still writing into the caller's buffer
- **where:** `dma/mod.rs:568-584`
- **what:** on timeout the function returns, so `dst: &mut [T]` is released while CH0 is still enabled. Even with F1 fixed, this path never calls `stop()`.
- **why it matters:** the same use-after-borrow as F1, on the path that runs exactly when the hardware is misbehaving.
- **severity:** high
- **fix:** on timeout, `CHDIS`, then bounded `CHBUSY` wait, then return. If the channel still won't stop, `panic!`. A DMA that can't be stopped while it owns a borrowed buffer is not recoverable in safe Rust.
- **confidence:** high (read)

### F3 — the async I2C future has no bound and isn't cancel-safe; its doc says otherwise
- **where:** `i2c/async_impl.rs:96-132` (`Ate::poll`), `:220-238`
- **what:** the doc says the poll limit exists "so a lost interrupt becomes a visible error". `poll` has no counter. It returns `Pending` and self-wakes forever, and `const _: u32 = WAIT_POLLS;` at the end of the file only silences the unused import. On any error (`?`) and on drop, no STOP or `CMD.ABORT` is issued, so the peripheral is left mid-transaction. The obvious fix, `embassy_time::with_timeout(...)`, drops the future and hits exactly that state.
- **contract violations:**
  - Every `Operation` gets its own START+address, so `[Write(a), Write(b)]` becomes two transactions. embedded-hal requires adjacent same-type operations to be merged.
  - An empty `Write` returns `InvalidData`. That breaks the standard address-probe idiom.
- **why it matters:** this is the same class as today's NACK hang, moved into async: a stuck bus is now a task that never finishes, burning EM0.
- **severity:** high
- **fix:**
  1. Count polls in `Ate`, return `Err(Timeout)` above `WAIT_POLLS`.
  2. Add a drop guard that issues `CMD.ABORT` + `CMD.CLEARTX`/`CLEARPC` if the transaction didn't complete.
  3. Merge adjacent same-type operations; allow an empty write.
- **confidence:** high (read)

### F4 — the siblings of the NACK hang: no STOP or abort after an error, anywhere in hardware I2C
- **where:** `cores/i2c/mod.rs:53-63,72-77,86-92` (every `?` returns before `bus.stop()`); `i2c/mod.rs` `I2c0::write/read`, `I2c1::write/read` (same pattern); compare `bus/bitbang.rs:165-230`, which *does* stop on NACK
- **what:** a NACK or timeout leaves the master holding the bus with no STOP and no ABORT. Nothing ever checks or clears `STATE.BUSHOLD`, and there is no 9-clock bus-clear recovery.
- **why it matters:** one NACK from a sensor that's busy, absent, or in reset can poison the next transaction. Over days of operation that's a near-certainty.
- **severity:** high
- **fix:** in the core, `let r = body(); if r.is_err() { bus.abort() }`, adding `abort()` to the `Bus` trait (`CMD.ABORT`, then wait for `STATE` idle). On the next transaction's entry, if `BUSHOLD` is set, abort first. Add a sim test that the abort is issued after a NACK. That's the one host test here that would fail if the feature were broken.
- **confidence:** high (read)

### F5 — I2C1 clears its NACK on I2C0's register
- **where:** `i2c/mod.rs:619-621` (inside `impl I2c1`, which starts at `:431`): `clear_window(pac::I2c0S::ptr())`
- **what:** I2C1's NACK flag is never cleared, so after the first NACK every later I2C1 byte reports `Nack`. As a side effect, a pending I2C0 NACK gets cleared.
- **severity:** high for anyone using I2C1. Nothing uses it today.
- **fix:** `I2c1S::ptr()`, or better, derive the window from `self.i2c` the way `impl_hw_bus!` does (`:123`).
- **confidence:** high (read)

### F6 — blocking I2C0 reads NACK from the non-secure `IF` alias, which your own async work says reads zero
- **where:** `i2c/mod.rs:122` (`HwBus<I2c0Ns>`), `:419` (`I2c0::write_byte`); compare `async_impl.rs:149-163`
- **what:** the async file records, as a measurement, that `if_` on I2C0's NS view doesn't reflect NACK, and it reads the secure alias for that reason. The blocking path, called "board-proven", still reads the NS view. So a NACK there comes back as `Timeout` from the next wait, or not at all. Board-proven covers the happy path only.
- **severity:** medium (wrong error class, plus F4)
- **fix:** read `IF` through the same alias you clear it on. Then do a board check: a transaction to an absent address (e.g. 0x55) must return `Nack` within 1 ms, blocking and async.
- **confidence:** medium. I read it and rely on your measurement; I don't know why the NS alias reads zero. An unexplained SMU/alias behaviour is itself worth one measurement.

### F7 — declared errors that hardware never produces
- **where:**
  - `i2c::Error::Bus` is only produced by `cores/i2c/sim.rs:91,101`. No driver reads `BUSERR`/`ARBLOST`.
  - USART `Framing`/`Parity`/`Overrun`: no read of `FERR`/`PERR`/`RXOF` anywhere in `usart/`.
  - SPI `Overrun`/`FrameError`: never produced. Only `InvalidConfig` is (`spi/mod.rs:158,291,414`).
- **why it matters:** the error taxonomy looks complete and the host tests exercise `Bus`, but on the device those conditions turn into silent data corruption or timeouts.
- **severity:** medium
- **fix:** read the flags at the existing wait points. Then delete each variant that is still unreachable, so the type stops implying coverage it doesn't have.
- **confidence:** high (grep, ran)

### F8 — the remaining unbounded waits
- **where:**
  - `spi/mod.rs:203,211,336,344,457,461` and `spi/traits.rs:34` (USART `flush`)
  - `usart/mod.rs:156,213` (missing from the brief)
  - `delay/mod.rs:139,153,181,195`
- **what:** SPI and USART hang forever if the peripheral is disabled, unclocked, or used after EM2 (EM01-group clocks stop). Delay only hangs if SysTick is stopped. It also uses `has_wrapped()`, which clears COUNTFLAG, so two `Delay`s (F10) stretch each other's waits rather than hang.
- **severity:**
  - SPI/USART: medium. They aren't on the dosimeter's path (IMU on I2C, BLE on radio), but they are public traits.
  - delay: low
- **fix:** the same `WAIT_POLLS` pattern I2C uses. Return `Error::Timeout`, adding the variant to `spi::Error`.
- **confidence:** high (read)

### F9 — bounded waits that time out silently, or report the wrong cause
- **where:**
  - `sysrtc.rs:322-326,412-416,433-437` (`break` at 200 000 spins, then carries on writing)
  - `board.rs:533-537,574-578` (`break` on `HFRCOEM23` not ready)
  - `wdog/mod.rs:336-339` (`SYNCBUSY` timeout returns `Error::Locked`)
  - 6 `board.rs`/`pwm.rs` functions return `Option` and discard the cause
- **why it matters:** if the SYSRTC sync times out silently, a compare gets programmed while SYNCBUSY is still set. That's a missed alarm, which is exactly the "cadence stops after days" failure, and nothing records it.
- **severity:** medium
- **fix:** return `Err(Timeout)` (sysrtc needs an error type), add a `Timeout` variant to wdog, and replace the `Option`s with `Result`s that carry the cause.
- **confidence:** high (read)

### F10 — `Board` hands out unlimited aliases of singletons
- **where:** `board.rs:64` (`i2c0(&self)` → `I2c0Ns::steal()`), `:139` `emu`, `:146` `wdog`, `:177` `mvp`, `:438` `delay`, `:463/473` `output_on`/`input_on`, `:558` `adc`, `:594` `i2c_static` (free function, returns `'static`); `sleep.rs` (`cortex_m::Peripherals::steal()`); `async_impl.rs:112` (steals I2C0 on every poll)
- **what:** the PAC's `take()` gives ownership, and the `&self` methods throw it away. Safe code can hold two `I2c0`s or two `Wdog`s, and two tasks can drive the same pin.
- **why it matters:** with a single superloop this is just a footgun. With an executor and interrupts (the stated direction) it becomes a data race the type system was supposed to prevent.
- **severity:** medium
- **fix:** make these `&mut self`, or have them consume a per-peripheral token taken once from `dp` (`Option<I2c0Ns>` fields with `.take()`). Keep `steal()` only inside interrupt handlers, with a SAFETY note.
- **confidence:** high (read)

### F11 — the timer prescaler is computed as log2; the hardware divides by PRESC+1
- **where:** `timer/mod.rs:143-163` and `:105,113,123` (`w.presc().bits(prescaler)`, `prescaler ∈ 0..=10` treated as divider `1<<prescaler`)
- **what:** RM TIMER_CFG.PRESC (bits 27:18) says *"divided by PRESC+1"*. The PAC enum agrees (`Div4 = 3`, `Div8 = 7`, `Div1024 = 1023`). For any target below about hfclk/65536 (~1.2 kHz at 78 MHz), the timer runs `2^k/(k+1)` times too fast. Example: 100 Hz → k=4 → divides by 5 instead of 16 → ~320 Hz. Also `target_freq == 0` divides by zero, which panics.
- **why it matters:** "timer-driven sampling cadence" is the product. This module and `timer-core` have no caller today (`pwm.rs` writes `presc().div1()` directly), which is why nobody has seen it.
- **severity:** high if anyone adopts `timer::` for the cadence; latent today
- **fix:** search `presc ∈ 0..=1023` with the divider `presc+1`, reject `frequency == 0` with `Error`, and wire `timer-core` with a test that fails on the log2 formula.
- **confidence:** high (read RM §TIMER_CFG and the PAC)

### F12 — wrong hardware premises written into code comments
- **pin.rs:141-146:** says "DOUT has no atomic set/clear register on this part ... zero occurrences in both". The RM register map lists `GPIO_PORTA_DOUT_SET` at 0x1040 and `GPIO_PORTA_DOUT_CLR` at 0x2040 (§4.2.4.3: SET at +0x1000, CLR at +0x2000). The vendor `em_gpio.h:1277` uses `GPIO->P_SET[port].DOUT`. The search looked for the Series 1 names (`DOUTSET`). Consequence: H17 is still live (see 4.F3).
- **clock/clocks.rs:165-178:** `assert!(config.lfxo.is_none())`, on the claim that "this SVD has no ... LFXOEN bit, so the crystal cannot be started". Series 2 starts the LFXO through its own peripheral, `LFXO_S`, whose `CTRL` (FORCEEN/DISONDEMAND) and `STATUS` (RDY/ENS) are in the PAC (`efr32mg2x-pac/src/lfxo_s/`). A caller passing `Some(lfxo)` panics at runtime over a non-problem.
- **severity:** medium. The comments steer future work away from the right fix.
- **confidence:**
  - DOUT_SET/CLR: high. I grepped the RM text.
  - LFXO is startable: high. Whether the XIAO MG24 has a 32 kHz crystal fitted: **not verified**.

## 2. Completeness against the part

From the SVD's secure peripherals, the count of HAL references to each (grep, ran):
- **zero:** `ACMP BURAM BURTC DCDC DPLL0 GPCRC ICACHE0 KEYSCAN LETIMER0 LFRCO LFXO PCNT0 PRS SMU SYSCFG ULFRCO VDAC WDOG1 SEMAILBOX`
- **partial:** GPIO (no EXTI), TIMER1-4 (macro only), EUSART (SPI only), LDMA (CH0 only, no IRQ), I2C (master only, no IRQ)

**Needed by the dosimeter (IMU continuously, RMS/RMSSD, BLE, cadence for days):**
1. **GPIO EXTI / EM2 wake on PA/PB.** IMU data-ready or FIFO-watermark interrupt. The RM (51394, 51466) says only ports A and B work as interrupts in EM2, so the IMU INT pin must be on PA/PB. Check the XIAO schematic.
2. **LFXO, or LFRCO precision mode.** SYSRTC currently runs from LFRCO without precision mode (RM 15468: precision mode gives ±500 ppm, and EM0-EM2 only). This sets cadence accuracy over days, and BLE sleep clock accuracy needs it too.
3. **SYSRTC sharing.** The HAL drives `GRP0_CMP0` + `SYSRTC_APP` (`sysrtc.rs:442,496`), and so does the BLE sleeptimer (`efr32mg24-bluetooth-sys/src/sleeptimer.rs:25-45`, by raw address). An Embassy time driver plus BLE in one image means two owners of one comparator and one IRQ symbol. Assign CMP1 to the HAL or Embassy, and dispatch both from a single handler. Confidence: read, not run together.
4. **WDOG running in the product with a fed-only-when-healthy policy.** The module exists. What's missing is a policy, plus BURAM to record the reset cause/count across resets (RMU/EMU `RSTCAUSE`), which today has zero references.
5. **DMA that stops (F1), plus LDMA IRQ.** Only if mic or IMU bulk transfer moves to DMA.
6. **DCDC.** Average current over days. Zero references today; check whether the XIAO routes the DCDC inductor.

**Merely absent (not needed for this product):** ACMP, VDAC, PCNT, KEYSCAN, LETIMER (SYSRTC covers it), GPCRC (useful for NVM integrity later), I2C slave, USART async/IrDA, TIMER capture/DTI, PRS (useful later for ADC triggering), SMU/TrustZone, SEMAILBOX/Secure Vault (pairing, D2f).

**States the present modules don't reach:** SPI and USART after EM2 (no re-init path); I2C after BUSHOLD (F4); DMA after timeout (F2); clocks after EM2 exit with HFXO/DPLL selected (no restore or verify path in `sleep.rs`); EMU EM4 wake source configuration.

## 3. Concurrency model

**Verdict:** cooperative polling with no driver interrupts is defensible for this product, but only if the cadence comes from the **IMU's own clock and FIFO**, not from the MCU.

- **Wake latency:** if the MCU timer paces each sample, every sample costs an EM2→EM0 wake and an I2C transfer. EM2 wake is tens of µs, and hundreds more if HFXO has to restart (datasheet figures from memory, not measured here). Jitter then depends on whatever else the executor is running.
- **Polling cost:** the self-waking async I2C keeps the core in EM0 for the whole transfer. That's about 1 ms per ~10 bytes at 100 kHz (your own note), or about 0.35 ms for a 12-byte burst at 400 kHz. Per sample at 100 Hz, that's roughly 3.5% EM0 duty, which dominates the EM2 floor.
- **What makes it work:** with the IMU FIFO batching samples and the IMU ODR setting the timebase, the MCU wakes once every ~0.5-1 s, drains the FIFO in one burst, and polling costs ~100× less. Then wake latency is irrelevant to the signal: RMSSD and RMS only need the sample spacing, which the IMU's oscillator provides.

**Minimum interrupt set to call this product-grade:**
1. SYSRTC, which you already have. Resolve the conflict with BLE first (§2.3).
2. **One GPIO EXTI** for the IMU FIFO watermark, on PA/PB.
3. Nothing else is mandatory. The I2C IRQ is an optimisation.

**The 44 647 591 re-entries:** they're evidence of a handler that doesn't disarm `IEN`, not evidence that an interrupt-driven I2C can't work. `TXBL`/`RXDATAV` are level conditions. Your own comment in `async_impl.rs:41-45` reaches the same conclusion. The standard pattern: arm only the bit for the current wait, and have the handler clear that bit in `IEN` before waking.

**One more race:** `sleep.rs` `enter()` doesn't take PRIMASK around the "is there work?" check plus `WFI`. An interrupt that arrives between the caller's check and `WFI` is a lost wakeup that lasts until the next tick. Embassy's executor already avoids this (`WFE`/`SEV`). Don't call `Sleep::enter` from under it.

## 4. The 2026-11-18 review: which of its 11 findings are still live

| # | status | evidence |
|---|---|---|
| F1 gate `cargo tree` passes vacuously | **LIVE, and worse than reported** | `gate.sh:415`. `cargo tree -p adc-core --edges normal,build -e no-dev` → **exit 101**, *"`normal` dependency kind cannot be mixed with ... no-dev"* (ran). The flag combination itself errors, before the name mismatch matters. stderr goes to `/dev/null`, so it always prints `pass`. The pattern still can't match `efr32mg2x-` |
| F2 five cores have no caller | **LIVE, as you measured.** clock/emu/gpio/timer/wdog have 0 callers in `crates/`; the 6 wired cores have 1-3 real callers each | grep (ran). New consequence: F11 above sits in exactly one of those unwired pairs |
| F3 `with_dout` can't close H17 | **LIVE.** `pin.rs:154-162` now does the read-modify-write under a critical section, but `bus/bitbang.rs:101-113` (`set`), `pin.rs:126-131` (`Output::new`), `:205` (`Input::with_pull_up`) and `set_mode` (MODEL RMW) don't. A critical section only protects code where every writer takes it. The correct fix exists in hardware (`DOUT_SET`/`DOUT_CLR`, §1 F12) | read |
| F4 duplicate `bt-heap` member | **LIVE** | `Cargo.toml:33-34` (ran) |
| F5 library types in HAL API | **Changed.** The cores are now `crate::cores`, so the crate-identity leak is gone. Still live: `pub mod sim` (`cores/i2c/mod.rs:15`) puts a test double in the HAL's public API, as `efr32mg2x_hal::i2c::proto::sim` and `efr32mg2x_hal::cores::i2c::sim` | read |
| F6 `emlib-sys` `publish = false` | **LIVE** | `efr32mg24-emlib-sys/Cargo.toml:7` (ran) |
| F7 sysrtc-core has two copies | **FIXED.** bluetooth-sys uses `efr32mg2x_hal::cores::sysrtc` (`sleeptimer.rs:18,69`). Note the new, different SYSRTC issue in §2.3 | read |
| F8 HAL won't build on host: unproven | **ANSWERED: it builds** | ran (see §0) |
| F9 check-publishable misses path-only deps and `include!`/`#[path]` | **LIVE as a check gap** (`check-publishable.py:46`: `INCLUDE` matches only `include_str!`). Latent for the HAL, whose only dependency now has a version (`Cargo.toml:13`) | read |
| F10 `pre-publish.sh` runs `cargo publish --dry-run` | **LIVE** | `scripts/pre-publish.sh:35` (ran) |
| F11 README generator shows `workspace` for path deps | **LIVE in code, latent for the HAL** | `gen-crate-readme.py:226` (read) |

**Where you are wrong about it:** nowhere on F2 or F10. On F1, the cause is the flag combination, not only the name. On F3, the critical section was presented as closing H17, and it doesn't.

## 5. Smallest set before I'd call it "complete", ordered by risk × cost

1. **F1 + F2 (DMA stop/timeout).** About 15 lines, and it's a soundness bug. Board check: `CHSTATUS` is clear after drop.
2. **F4 + F3 + F5 (I2C error recovery: abort on error, async bound, drop guard, I2C1 window).** About 80 lines plus one sim test that fails without the abort. Board check: absent address → `Nack` within 1 ms, then a good transaction succeeds, on both the blocking and async paths.
3. **Gate F1 (2026-11-18).** One line. Until it's fixed, the "libraries are pure" invariant is unverified.
4. **§2.3 SYSRTC ownership between HAL/Embassy and BLE.** A design decision plus about 30 lines. A product that has both can't have two comparator owners.
5. **GPIO EXTI on PA/PB plus the IMU FIFO cadence (§3).** The one new feature the product needs.
6. **F9 silent timeouts → `Err`, F6 NACK alias.** Small.
7. **F10 ownership.** Mechanical, but it touches the `Board` API. Do it before anything publishes the HAL again.
8. **WDOG policy + BURAM reset log.** This is what makes "runs for days" observable.

**What I would *not* do:**
- I wouldn't make I2C/SPI/USART interrupt-driven now.
- I wouldn't add the absent peripherals from §2's second list.
- I wouldn't chase the `unsafe` count, which is mostly `w.bits()`.
- I wouldn't wire the five unwired cores for their own sake. Wire `timer-core` only when F11 is fixed, and `gpio-core` only for `route()` after DOUT_SET/CLR lands.
- I wouldn't move more code into `cores/`.
- I wouldn't publish anything.
