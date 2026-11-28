# HANDOFF — EFR32MG2X-RS (XIAO MG24 Sense + Expansion Base)

**State: green.** Branch `master`, pushed to `origin` (`bitscrafts/EFR32MG2X-RS`). The Expansion Base
was found faulty on 2026-11-15 and is away for replacement (about a month); that does not remove
anything already measured with it — those dates stand — and the examples that need the carrier say so
until another one is on the bench. The board left running `hal_wdog_feed`, its record `PASS`.

## Session 2026-11-27 — TIMER1–TIMER4 measured, and the constructor nobody could call was wrong twice

**State: green.** Branch `master`, pushed. Board left running `hal_timer_instances`, `status=PASS`
(`timers: ms=50 o1=48 o2=48 o3=48 o4=0 stopped=0 top=18999 want=1000 clean=1 refused=1`, three boots identical).

The four rows in the coverage boundary that read "compiled, not measured" for `TIMER1`–`TIMER4` were not laziness: the
driver's only constructor took a `&FrozenClocks`, and `FrozenClocks` owns the CMU that the `Board` owns, so **no
`Board` hand-out could build one**. Making them reachable took:

- `TimerN::new_at(timer, config, hfclk_hz)` — the shape `usart` and `i2c` had already adopted; `new` now delegates.
- `Board::timer1()` … `timer4()`, one claim each (`Slot::Timer1`…`Slot::Timer4`, `SLOT_COUNT` 17 → 21).
- `Board::open_timer_branch()` — `CLKEN0.HFRCOEM23`, `FORCEEN`, wait `STATUS.RDY`, `EM01GRPACLK = HFRCOEM23`. Gating
  `CLKEN0.TIMEn` alone is not enough: the first register write faults.

**Two defects came out of the first run, both in code that had never executed.** The constructor enabled the block
*before* writing `CFG`, which faults (`Escalated BusFault ... at 0x4004c004` = `TIMER1`'s `CFG`); with the order fixed,
`TOP` turned out to fault the other way round, with the block disabled (`... at 0x4004c01c`). And `clear_overflow()`
cleared **`TIMER0`'s** flag from every instance, because the base was hard-coded in the macro that generates all five
drivers. A crate with one measured timer cannot see either.

The measurement counts **overflows**, not the raw counter: `CNT` advances at the oscillator and wraps at `TOP`, so
counting `CNT` measures the clock while counting overflows measures the period the driver was asked for. 48 overflows
against a 50 ms SYSRTC window at 1000 Hz is a measured ~960 Hz; the two oscillators are not the same to 1 %, and the
tolerance (±3 in 50) still refuses a stopped counter or a prescaler off by two.

**Uncommitted by design:** nothing. `scripts/check-missing-docs.py` and the 41-file `missing_docs` sweep were committed by
the night cron at 15:31 (`e878719`) after this session's edit to the coverage boundary was already in the tree; that
edit went in separately as `437c752`, and this session's work follows it.

**Next:** `EUSART1` is the last "compiled, not measured" row; then the PRS consumers beyond the IADC trigger; then the
modes in section 3 of the boundary (SPI/I2C slave, 9-bit, IrDA/ISO7816, I2S, IADC scan, timer input capture).

## Session 2026-11-25 — SPI and USART measured, the CI that built a quarter of the tree, and a safety sweep that found the buzzer dead

**State: green.** Branch `master`, pushed. Board left running `hal_usart_link` (jumper D6–D7 fitted, record `PASS`).

This session closed the four items that stood between the tree and a publishable 0.1.0, and each one turned out to be
more than a documentation fix.

- **SPI**: `hal_spi_loopback` drives the board's own pads (D10 = PA5 MOSI, D9 = PA4 MISO, D8 = PA3 SCLK) with a jumper
  between MOSI and MISO, and carries a **measured control**: moving MISO to a pin no jumper reaches gives
  `in=ff..ff` and one accidental match out of eight. Routing for USART0 and EUSART0 had to be written first — the HAL
  had `route_i2c0` and nothing else, which is why `spi` and `usart` were drivers that reached no pad.
- **USART**: `hal_usart_link`, jumper D6–D7, `PASS out=6 in=6 ok=6 45 46 52 33 32 21`. Two defects fell out of
  writing it: a six-byte burst overruns the two-deep receiver (`in=3 ok=2 45 46 21`), and a floating RX line reports
  `0x00` often enough that a byte-counting echo test passes on a board with nothing attached.
- **The `dwt` trap**: `dwt::start()` now proves the counter moves and returns `DwtError::NotRunning`, so a frozen
  counter cannot be read as zero.
- **CI**: `check-examples-build.sh` carried a hand-written list of 26 examples while the workspace had 107. The HAL
  alone shipped 22 and the script compiled 4 — every instrument added since mid-November was outside CI, and the green
  verdict was about a quarter of the tree. It now builds every example of every crate and refuses a tree where an
  example cannot be reached. **107 examples, 9 crates.**
- **Safety sweep**: 197 `unsafe` sites annotated, `scripts/check-safety-notes.py` added to the gate. Writing the notes
  found three stacked defects on the buzzer path (`CC0_CFG` written with the counter enabled → bus fault at
  `TIMER0 + 0x60`; `EN` crossing the clock domain; the calibration reading `CYCCNT` that was never enabled). Result:
  `PASS buzzer tick=19000000Hz top=3518 tone=2700Hz`.

Then F10, the last item that kept the API below B: `Board` handed out unlimited aliases of its singletons, because every
hand-out built a driver from a `steal()`. It now goes through a claim table (`cores::slots`, host-tested in
`libraries/slots-core`), a second request is refused by name, and `hal_one_of_each` measures it on the board:
`PASS one-of-each: sysrtc=1 sysrtc2=1 adc=1 adc2=1 wdog=1`. `mod cores` also stopped being public API; what a consumer
needs from it is re-exported by `sysrtc`.

Documentation grades moved: evidence B− → **B**, safety C → **B**, publication hygiene D → **B**, API C+ → **B**. What is still below B
is API hygiene (C+: `Board` hands out unlimited aliases, `pub mod cores` is public) and error handling (`wdog` returns
`Error::Locked` for a timeout).

## Session 2026-11-24 — the six red examples, all closed, and three of them were the instrument lying

Every example the HAL README listed as "ran, and the record says it did not work" now has a dated PASS. Half of them
were not driver defects at all: the record itself was broken.

1. **`hal_timer_compare`** — nothing was wrong. `late=1 fired=1`, one tick after the deadline, with the stale flag
   drained first. It had simply never been re-measured after that fix.
2. **`hal_imu_read`** — a `copy_from_slice` of 7 bytes into `buf[..8]` panicked before the record was written, so the
   example was filed as "writes nothing the harness can read". A silent panic in an instrument reads as missing data.
   `PASS imu 6a x-351 226 940 |a|=1028`.
3. **`hal_timer_ticks`** — two failed instruments before a working one. See below: it is the DWT finding.
4. **`hal_emu_voltage`** — the example rewrote its own verdict as `info` every second, so a reader over SWD never saw a
   verdict; the two historical readings (`ok , ok` and `ERR , ERR`) cannot be compared because neither had a status.
   Five round trips per boot now, three boots: `5/5 , 5/5` every time.
5. **`hal_mic_level`** — already fixed on 2026-11-21 (the LED PWM was fighting the microphone for TIMER0) and the
   README kept the old failure. `PASS mic 8 rms=7 pk=18 n=128`.
6. **`hal_wdog_feed`** — a real driver defect, and the note on file blamed the wrong register.

### The DWT cycle counter only runs while a debug session is attached

Two probes, both anchored outside the firmware: **detached**, two readings 10.8 s apart on the host clock gave the same
`c=29605` while the SYSRTC advanced 346 091 ticks (32 060 Hz, correct); **attached** under `probe-rs run`, DWT/SYSRTC
came out at 578.4 against the 579.8 that 19 MHz / 32.768 kHz predicts. So the counter is right while attached and
frozen while detached -- it lives in the debug power domain, and `DEMCR`/`CYCCNTENA` read back set while it stands
still. The tick example now uses **TIMER0** (HFRCOEM23, nominal 19 MHz) as the fast-clock reference and reads `cyc=1896280 fast=18961kHz`.
Record: `docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md`. **Any `dwt` number in this repository is only valid if the
probe was attached when it was taken**; `hal_mvp_bench`, `hal_sysrtc_wide`, `module_clock_probe`, `module_mic_rate`
and the dosimeter's cycle budget are all in that class.

### The watchdog: `feed()` wrote `CMD` without waiting for the previous clear

`probe-rs run` named the frame: `BusFault (Precise data access error) at 0x5b00400c`, `Wdog::feed`, the `CMD` write --
not `CFG`, which the older note blamed (`0x0c` is `CMD`). Ruled out one boot each: the window alias (same fault at both),
the configuration lock (same with `lock_config()` removed), the LF clock source (same with `HCLKDIV1024`), the pending
PRS flags (same after clearing them). While faulted the block reads `IPVERSION=1 EN=1 CFG=0x000a0020 SYNCBUSY=0` and
`CMU_CLKEN0` bit 13 is set: alive, configured, and only `CMD` refusing.

**`SYNCBUSY` has one bit on this part -- `CMD` -- so the driver's `wait_syncbusy()` after the `EN` write waited for
nothing.** Two feeds back to back were therefore the failing case: the second `CMD` landed while the first was still
crossing into the WDOG's clock domain (LFRCO, 30.6 us a cycle). `feed()` now waits for `SYNCBUSY.CMD` before and after
the write, bounded; the control (both waits removed) puts the fault back, and the fix reads `fed 95 st80000000 sb0
lock1`. Also measured: with `CMU_CLKEN0.WDOG0` clear -- its reset state -- **any** access to the block faults, reads
included (`IPVERSION` at `0x5b004000`), which is why the example reads its registers only after `board.wdog()`.

## Session 2026-11-23 — the I2C bus survives a refusal, the DMA stop is proven, and yesterday's diagnosis was wrong

**Closed: the I2C bus survives a NACK.** The record, read back off the part:

```
i2c-rec: antes=0x6a erro=1 depois=0x6a wr=1 rd6=1 nz=1 volta=0x40      status=PASS
```

The cause was **not** a held bus. `IF` is RW on this part and its flags are sticky, cleared only through the
`IF_CLR` alias at `+0x2000` (RM 22.3.8.2; `I2C_IntClear` in `em_i2c.h`); the `HwBus` written the day before wrote 1 to
the **plain** register, so the NACK of transaction 2 was still asserted when transaction 3 began, and `write` — which
checks the flag before loading a byte, to tell a refusal from a timeout — refused transaction 3 on the strength of
transaction 2. The previous session's conclusion, "`BUSHOLD` is a level, the bus is genuinely held", was wrong:
`STATE.BUSHOLD` means "the bus is currently held by **this** I2C module", the module pausing for software.

Attributed with a control, not with a pass: with only the clear put back on the plain `IF` (everything else kept) the
record returns to `depois=0x00`, FAIL; restored, PASS. `abort()` is now `ABORT|STOP` in one command, wait for `BUSY`,
`CLEARPC`, `CLEARTX`, then all sticky flags through `IF_CLR`, in the order RM 22.5.4 gives. `I2c0::write` and
`read` were rerouted through `cores::i2c` so every blocking entry point aborts on error; `I2c1`'s `write_read` became
one transaction with a repeated START; 372 lines left `src/i2c/mod.rs` and 146 arrived in their place.

**The instrument was extended, because the reuse needed its own proof**: `wr=1` is a plain `write` read back
(`CTRL1_XL` = 0x40, then restored), and `rd6=1 nz=1` is a **six-byte** burst of accelerometer data that is not zero —
which is also what closes `I2C0-READ-DEFECT.md`, whose fourteen measurements all shared the same blind spot (the
"clear all 22 IF bits" attempt wrote to the plain register and cleared nothing).

**Docs corrected where they asserted the old defect**: `I2C0-READ-DEFECT.md` (status closed, rows 8/9 annotated),
`I2C-RECOVERY-STATE.md` (rewritten), the HAL and PAC crate docs and the HAL README template — the generator checks the
README against the tree but cannot see a hand-written number, and "twenty-one modules, ten examples" was stale on both
counts (26 `pub mod`, 20 examples).

**Still open, named rather than implied**: a *follower* that genuinely holds SDA low needs nine clocks on SCL and a
STOP, with the pins taken from the peripheral; nothing in this repository produces that condition, and the criterion
that would recognise it is written in `I2C-RECOVERY-STATE.md`. The blocking `embedded-hal` `transaction` still emits
one STOP per operation (the async path does not).

### The same session, second item: `dma::Channel::stop()` is proven, and the instrument had been reading the wrong bit

**Closed: `stop()` disables the channel.** Record from the part:

```
dma-stop: st0=0 armed=1 stA=1 busyA=0 stS=0 ctl=1 stRe=0 stS2=0      status=PASS
```

The old instrument read `CHBUSY`, got `busy_armed=0` and could not answer — not a race but the wrong register:
`CHBUSY` (RM 25.7.13) means "this channel is transferring", and an armed channel can sit waiting. `CHSTATUS`
(RM 25.7.12, 0x02C) is the per-channel **enabled** state, and it is readable at any time. With it: enabled while the
guard holds the buffer (`stA=1`), disabled after dropping the guard (`stS=0`), and a second `stop()` idempotent
(`stS2=0`). Control in the other direction — `stop()` put back to `chen().bits(0)`, which the manual says has no
effect — flips exactly those three: `stS=1 stRe=1 stS2=1`, FAIL.

**A second defect the measurement found**: the blocking `transfer()` returned with the channel **still enabled**
(`stRe=1` before the fix) — transfer complete, borrow over, descriptor still owned by the hardware and reachable by a
later request. `transfer()` now calls `stop()` before `Ok`, and the record shows `stRe=0`. Full record, including why
`CHBUSY` could not answer: `docs/04-reference/DMA-CHANNEL-STOP.md`.

## Session 2026-11-22 — the async I2C works, three guards were lying, and two fixes are not proven

Intervention paused here. Everything below is committed and pushed; `master` is at `edd5acd`, gate green, tree clean.

### Closed, each with the measurement that closed it

1. **The async I2C reads the IMU alone.** `i2c-async: who=0x6a pending=145 irq=0 wfi=0`, no blocking call before it.
   The bug was a missing **NACK on the last byte**: every byte was read with ACK, so the slave was never told to
   stop driving data, the master never closed the reception, and what the markers showed was a held bus with no
   error. The correct sequence was already in the crate -- `cores::i2c::write_read` calls `read(i != last)` -- and
   the first version of the async file was written from memory instead of from it. Six flashes.
2. **The gate's own guard was dead.** `cargo tree -p "$name" --edges normal,build -e no-dev`: the directory name
   instead of the package name, two flags cargo refuses together (exit 101), stderr to `/dev/null`, and the `else`
   branch printing "no hardware dependency" for all 18 libraries, always. Fixed, with controls in both directions
   against real `cargo tree` output. It never ran before today.
3. **`dma::Channel::stop()` disabled nothing.** It wrote zeros to `CHEN`, a write-1-to-set register; the disable
   register is `CHDIS`. A `Transfer` could end the borrow of a buffer the DMA was still writing into, from safe
   code. The timeout path had the same defect. Both fixed; `is_busy`/`channel_busy`/`trigger` added so a caller can
   tell a working `stop()` from the one that did nothing.
4. **DMA could never be armed at all.** `CMU_CLKEN0` gates LDMA and LDMAXBAR on **separate** bits and only LDMA was
   set, so the first write of the peripheral-to-memory path (`LDMAXBAR->CH0_REQSEL`) was a bus fault. Found by
   elimination with one marker per register write. With the clock on, the arm returns: `armed=1`.
5. **SYSRTC channel 0 ownership decided** (`SYSRTC-OWNERSHIP.md`): it stays with the Bluetooth sleep timer; the
   application takes 1 or 2. Enforcement (a token the compiler checks) is named and not written.
6. **The IMU FIFO measured and rejected** (`IMU-FIFO-MEASURED.md`): the driver already bursts 12 bytes per sample,
   so the FIFO saves 1.8 points of bus occupancy for a 384-byte buffer and a new API. It would buy contiguity, which
   is a product question, and that is written as the condition that would change the answer.
7. **The GPIO wake item was withdrawn**: INT1/INT2 of the LSM6DS3TR-C are no-connects on this board (schematic sheet
   06), so there is nothing to interrupt. The review's item 5 is half impossible here, and the postscript in
   `HAL-OPUS-ANALYSIS-2026-11-22.md` says so.

### Open, with the exact next step

**The I2C bus does not survive a NACK.** Measured, in this order:

```
antes=0x6a  erro=1 (Nack)  depois=0x00          hal_i2c_recovery, the instrument
IF = 0x109f9 before the last START:  bit 7 NACK set, bit 11 BUSHOLD set
START_INICIOS = 4, START_EXPIRADOS = 0         so the START is not the culprit
```

Transaction 3 is refused by a **ghost**: the NACK flag from transaction 2 is still asserted, and `write` reads it
inside its `txbl` wait, so the first byte returns `Nack` without touching the bus. `abort()` clears `NACK` and
`BUSHOLD` and both are set again at the next read -- clearing the flag does not clear the condition, because
`BUSHOLD` is a level: the bus is genuinely held.

Next step, and it is a **design decision before it is code**: nine clocks on SCL, then a STOP, then hand the pins
back to the I2C peripheral. That needs the pins taken away from the peripheral and returned, which is who owns the
pins during recovery. The cheap alternative -- re-initialising the peripheral after an abort -- hides the condition
instead of clearing it and should be recorded as such if it is ever taken.

`dma::Channel::stop()`'s effect is still unproven on hardware: a software trigger drains 64 bytes between two
statements, so `CHBUSY` reads idle again before it can be sampled. The instrument that would prove it is an
IADC-driven transfer or a DMA interrupt reading `CHBUSY` from inside the handler.

### Environment facts for the next session

- The container root filesystem is **full** (a few hundred MB). `TMPDIR` must point at the big volume:
  `export TMPDIR=/projects/_tmp/tmp`, or `cargo` fails with no explanation. `/projects` has ~77 GB free.
- `/tmp` holds ~70 GB across ~181 000 entries, much of it `/tmp/orki-test-*` from another session's test runs. It
  is not this project's, and it has not been touched.
- The board is on `10.10.10.217` running `hal_i2c_recovery`, not the product. **The operator asked that the product
  not be restored until he says so.**

### Process lessons this session paid for

- A count of steps is not an attribution of the failing one: "13 passos, last = abort" was read as "fails on the
  repeated START", and the next instrument disproved it. Two commits were spent writing the wrong answer down and
  then correcting it -- cheaper than building a fix on the wrong register, which is what almost happened.
- `.gitignore` swallowed two real files in one day (`*recovery*.rs` an example, `*RECOVERY*.md` a document), both
  times with a hint nobody read and a commit that exited 0. Both rules now exclude `docs/` and `examples/`.
- A `str.replace` that does not match applies nothing and says nothing: one measurement was concluded from a
  firmware that never contained the code under test.

---

## Session 2026-11-17 (evening) — BLE measured encrypted end to end, and five HAL defects closed

1. **The link is encrypted, and it is the device that says so.** Pairing once, then after a reset of the MCU with no
   pairing, the host reports `Connection successful / Bonded: yes / Connected: yes` and the device reports `sec=1`,
   `mtu=247`, `windows=3780`, `published=60`, `dfail=0`, `disc=0`, `adv=0`. The load-bearing number is `sec=`, not the
   host's `Bonded:` line: the host prints that from its own bond store whether or not the link is encrypted, and this
   project made exactly that wrong claim once. What was missing was one vendor instruction —
   `sl_bt_sm_increase_security` on connect — and nothing ever issued it.
2. **The product refuses to be built without the external bonding database.** `module_ble_dosimeter` declares
   `required-features = ["stack", "sense", "external-bonding"]`, so a plain build stops with a clear message instead of
   producing a device that pairs, stores a bond and cannot find the key after a reset.
3. **Five review findings closed, each measured or bounded.** H1 (DMA linked to `LDMA->IF` at 0x50 instead of
   `CH0_CTRL` at 0x64, confirmed against the PAC), H3 (eight sites "cleared" flags by writing 1 to `IF`; this family
   clears through the `_CLR` window at +0x2000), H14 (the microphone's level was measured about a fixed 2048 while the
   converter's bias is 2391 — it published 343 for a signal of 8), H16 (FIFO overflows are now counted: `ovf=0` over
   3780 windows, so latent, and visible from now on), H20 (fourteen I2C waits bounded, `Error::Timeout`).
4. **Two defects the day's own checks caught, both mine.** The first H14 fix computed the variance in closed form and
   reported `rms=51` for a window whose extremes allowed at most 25.5 (Bhatia-Davis): the invariant written to keep the
   numbers honest rejected my arithmetic. And the report's source checker treated a row with no openable source as a
   *skip*: it is now a failure, and it immediately found a row citing "this repository".
5. **What blocks the rest of the HAL queue.** The I2C transaction model (repeated START) and the ADC's analogue bus both
   need things this project does not have: pin routing for the hardware I2C (`I2c0::new` configures no pins; the board
   exposes only the bit-banged bus) and the reference manual's pin-to-analogue-bus table for `GPIO->ABUSALLOC`. Both are
   named in the code where the gap is, rather than guessed: a plausible number from the wrong pin is the failure mode
   this queue exists to prevent.
6. **Where the work continues.** `_tmp/ble-plan/PLAN.md` (12 steps, `file:line`), `_tmp/ble-plan/IMPLEMENTED.md` (what
   the implementation pass completed and what it could not verify), `docs/04-reference/BLE-OPERATIONAL.md` (what is
   measured and what is open), and the open-items table in §7.6 of the dosimeter report.


## Session 2026-11-15 — the board learns to talk back, and the carrier steps out for a while

1. **The Expansion Base is faulty and away for replacement.** Seated in it the module's debug port is
   silent from both probes and both read paths, the application never starts, and a component on the
   base heats with USB power alone — reproduced on a second host, so host, cable, programmer and
   firmware are all excluded. With the base removed the module flashes, verifies by read-back and
   runs. Full evidence and the rail-by-rail procedure: `docs/01-getting-started/EXPANSION-BASE-SWD-FAULT-2026-11-15.md`.
2. **The board can be watched while it runs.** `defmt` + `defmt-rtt` + `panic-probe` are wired for
   both crates (`-Tdefmt.x` from each crate's own `build.rs`, `DEFMT_LOG=info`), and the shared mailbox
   module now sends the same record over RTT — so every example streams live and a fault arrives with
   a backtrace instead of silence.
3. **Three causes behind the SYSRTC fault, each with an exact address.** Writing the group registers
   with the SYSRTC's clock gate still off (`0x500a8068`), then STOP+EN_CLR followed by a CFG write
   (`0x500a800c`, even with SYNCBUSY polled); the second clock selection is now made with the counter
   running. Measured: LFRCO 32800 ticks over 19000025 cycles, ULFRCO 992 over 19000020.
4. **Three examples were reporting another program's result, and are fixed.** `pac_minimal`,
   `pac_sysrtc_rate` and `pac_sysrtc_trace` never touched the diagnostic slot, so their rows carried
   whatever the previous binary had written. Every example now invalidates the slot in the first
   instruction of `main`, and `hal_timer_compare` drains its stale compare flag before claiming a pass.
5. **One name per crate, and the board hands out its own peripherals.** The PAC and HAL libraries now
   derive from their package names (`efr32mg2x_pac`, `efr32mg2x_hal`), and the gate's layering rule —
   no PAC type named outside the PAC crate — is satisfied by `Board::emu()` and `Board::wdog()`.
6. **Published: not yet.** `crates.io` refuses the name until the deleted-crate cooldown ends
   (2026-11-16T03:57:22Z); a job is queued for 04:05Z. The HAL stays unpublished by decision.

## Session 2026-11-14 — Bluetooth closes, and what closed it

1. **The Bluetooth stack works on hardware.** Beacon advertising with a name, scanning that decodes a known address, a
   connectable GATT server with read, write and notify, and the firmware echoing written bytes in its notifications --
   so the proof cannot be answered by a host cache.
2. **The BME680 reaches a phone.** `base_ble_env_gatt` publishes the carrier's sensor as the standard `0x181A`
   Environmental Sensing service with `0x2A6E` temperature, `0x2A6F` humidity and `0x2A6D` pressure, read and notified.
   Verified on the user's phone with nRF Connect, and byte for byte by `scripts/host-env-verify.py`.
3. **The heap defect that broke a second connection is contained.** A vendor-blob write of about 136 bytes into a
   64-byte allocation destroyed the block header that followed it, dropping 57 KiB of a 64 KiB arena out of reach.
   Blocks now carry 256 bytes of slack: with a client connected the arena keeps 53720 bytes instead of 72.
4. **The PSA layer for pairing exists and is verified on the board** -- P-256 generation, public-key export, ECDH,
   AES-CMAC, AES-CTR, AES-ECB -- behind a catalogue switch that is off in the product build.
5. **Documentation and hygiene for publication.** No references to the internal bench host or account remain; the
   vendored OpenOCD binary and three third-party `.hex` files left the tree (13 MB); the internal book tooling left the
   repository; CI now runs the same gate as the bench instead of a step that could not fail, and
   `scripts/check-examples-build.sh` builds all 26 examples with the features they declare.

**The one open question:** with the security manager enabled a pairing request is accepted and then fails at a DHKey
check with `pg=0 pe=0 pa=0` -- no cryptographic call was made -- so the fault is before the key exchange. The component
stays out of the product build until that is answered.

## What runs today, and the evidence

| example | layer | evidence |
|---|---|---|
| `pac_env_display` / `hal_env_display` | PAC / HAL | `T=31.9C P=1015hPa H=68.3%` on the panel |
| `pac_bme680` | PAC | Bosch compensation, `par_t1=25975` |
| `pac_rtc_read` | PAC | real calendar read |
| `pac_oled_hello` / `hal_oled_hello` | PAC / HAL | text on the 128×64 panel |
| `pac_gpio_button` / `hal_button` | PAC / HAL | `PC01` pull-up, `DIN=1` at rest |
| `pac_buzzer_pwm` | PAC | `tone=2700Hz`, intermittent, measured |
| `pac_mic_vu` / `hal_mic_vu` | PAC / HAL | `rms≈550`, 128 samples, LED follows level |
| `pac_imu` / `hal_imu` | PAC / HAL | `id=6a a=970,79,142` (one g) |
| `pac_pullup_probe` | PAC | found the IMU bus by external pull-ups |
| `pac_imu_probe` | PAC | `sda=p1.2 scl=p1.3 a=6a id=6a` |
| `hal_rtc` | HAL | `rtc 2026-11-10 12:00:57 kept` (ticks across resets) |
| `base_fall_detector` | app | `fall selftest real=clean synth=detected` |
| `bsp_env_clock` | BSP | `bsp dev=111 T=31C P=1017hPa H=67` |

## Layering, as built (2026-11-12)

```
crates/efr32mg2x-pac              registers
crates/efr32mg2x-hal              drivers + pin/bus primitives; no board pins anywhere
crates/xiao-mg24-bsp              the module: LED; feature "sense": IMU + microphone
crates/xiao-expansion-base-bsp    the base: OLED, BME68x, RTC, buzzer, button
libraries/motion-detection        domain policy: [i32;3] + dt_us -> Event, zero dependencies
```

Rules the gate enforces: nothing in `libraries/` may pull the hardware stack; every
`*-bsp` crate must have examples; every example living in a directory must be declared in
its manifest. `docs/03-development/EXAMPLES.md` lists each example with the hardware it
needs, and `docs/02-architecture/CRATE-ORGANISATION-RESEARCH.md` explains why the split is
this one (and how rp-hal, nrf-hal, stm32f4xx-hal and esp-hal each do it differently).

## Layering, as agreed

- **HAL = drivers.** A driver answers "what did the device say". Accelerometer range
  and ODR are driver concerns and are configurable; *deciding that 2.5 g is an
  impact* is not.
- **`motion-detection` = algorithm crate.** Hardware free, host tested, no
  dependencies. The fall detector lives here, not in the HAL: an earlier version put
  it in `hal::sensors::fall` and that was wrong.
- **BSP = this board.** Addresses, pins, buses, power gates, conflicts.
  `crates/xiao-mg24-bsp/src/expansion_base.rs`.
- **Examples are applications.** They compose; the dependency runs application →
  algorithm → driver, never back.

## Facts that cost time, do not relearn them

1. **The IMU is on PB2/PB3 (SDA=PB2, SCL=PB3), not on the base bus**, and the
   Arduino variant has the two lines swapped. `PD5` powers it. See
   `docs/04-reference/IMU-INVESTIGATION.md`.
2. **The microphone is analogue**: PC8 enables it, PC9 is the input, IADC0 samples
   it. `IADC.SINGLE` can only be written while the converter is **enabled**; writing
   it disabled faults and wedges the engine until a power cycle
   (`docs/04-reference/IADC-MIC-NOTES.md`).
3. **The on-chip I2C0 master reads and writes** (since 2026-11-23; the fourteen "read defect" measurements were two
   driver defects: NACK on the last byte, and flags cleared through the plain `IF` instead of `IF_CLR`). It reaches one
   pin pair, PB2/PB3 for the IMU; the Expansion Base's PC4/PC5 bus is the bit-banged one
   (`docs/04-reference/I2C0-READ-DEFECT.md`, `docs/04-reference/I2C-RECOVERY-STATE.md`).
4. **`format_fixed` takes value×100.** Passing milligrams raw prints the reading
   divided by 100 — it looks exactly like a dead sensor.
5. **The MVP is compiler-programmed**, not hand-programmable in practice: the
   reference manual removed its register chapter in Rev 1.1 and the SDK compiles
   `.tflite` into register programs. `pac_mvp_add` builds and flashes but does not
   execute yet; next step is a known-good SDK program as an oracle
   (`docs/04-reference/MVP-NOTES.md`).
6. **Flash only through `scripts/flash.sh`** (dry-run by default, refuses
   `load_min < 0x08006000`, verifies by read-back). The bootloader at
   `0x08000000` has been intact after every flash.

## Next, in order

1. Retire the June BSP modules (`board.rs`, `pins.rs`, radio/sensor modules) — they
   compile but were written against an older HAL API and are not what the facade
   uses.
2. microSD (SPI, chip select shares a pin — the facade is where that conflict should
   be refused), then Grove UART/ADC and the servo.
3. A real fall dataset: `hal_imu_monitor` records 15 s at 200 Hz and `pac_imu_capture`
   10 s at 100 Hz; the classifier needs several genuine drops, not shakes.
4. MVP: run a Silicon Labs sample as the oracle, dump the register block before
   `CMD`, diff against `pac_mvp_add`.
5. Radio last, as agreed.

## Fall detector — state at the end of 2026-11-11 (read this before touching it)

**On the board:** `base_fall_detector`, self tests `real=clean synth=ok 200Hz=2/2 400Hz=4/4`.
It classifies **and** records while it classifies, and it beeps three times on a fall.

**Proven with real drops:**

| round | what | result |
|---|---|---|
| 1 | 2 drops, 200 Hz | 2/2 |
| 2 | 5 drops, 200 Hz | 3/3 detectable (two lost their free fall between samples) |
| 3 | 4 drops, 400 Hz | 4/4 |
| 4 | 4 drops onto a bed, real timing | 4/4 — free fall 176–252 ms, landing 11.7–13.8 g, settle 1.04–1.17 s |
| control | handling recording | 0 false alarms **on a time base now known to be wrong** |

**Open, and it is the only thing between "promising" and "validated":** the shaking control
has to be re-run with the current firmware. The old one was measured while the loop claimed
400 Hz and ran at 158, and the round that did get recorded filled the log 22 times over
because every sample above 2 g was its own "impact". Both are fixed; the measurement is not
made yet.

Read it with `scripts/read-fall-log.sh` (dumps and decodes; addresses come from the ELF, not
from this file). Three counters decide it: `FALLS` must be 0 after shaking, and equal to the
number of drops after dropping.

**Facts that cost time in this session, not to be relearned:**

- The loop **cannot** be trusted to run at the rate it aims for. It measured **235 Hz** with a
  mean interval of 4249 µs and stalls of **63.75 ms** when the display flushes. Always pass
  the measured interval, never the intended one.
- An impact is **one event per episode**, reported when the bang ends with the peak it
  reached. One event per sample turned three seconds of shaking into 1430 log entries.
- A recording must carry its own timing (`recording::DTS`). The optimiser **deleted** that
  array until the writes became volatile — the recording would have claimed to be self-timed
  and carried nothing.
- The ring's unwritten slots read as zero, and a zero sample looks exactly like free fall.
  Analyse only `TOTAL` samples.
- A replug resets the board and loses the RAM log. A log that must survive belongs in BURAM.

## 2026-11-17 (night) — the bond across a reset, and where it stands

**State: pairing green; persistence green; reconnect after a reset still open.** Tree clean, gate PASS,
six commits. Full chain with every number: `docs/04-reference/BOND-ACROSS-RESET-2026-11-17.md`.

Fixed and measured tonight (all in `crates/efr32mg24-bluetooth-sys/src/psa.rs` unless noted):

1. `nvm3_readData` refuses a length larger than the object; the restore asked for header+MAX_KEY (73) against
   40-byte objects, so every read failed. Read `nvm3_getObjectInfo` first.
2. The restore took its "done" flag before checking the NVM3 handle — the first PSA call of a boot arrives
   before the stack opens the instance, so one silent attempt per boot and never again.
3. The retry flag was a function-local `static mut`: a separate variable per function in Rust, so the helper
   cleared nothing. Module scope now.
4. The stack names the key it wants (`bondingdb_psa_add_ecb_key` writes `0x48000 | type | (handle << 8)` into
   the attributes at offset 8, then imports); we handed out our own id. Honour the requested one.
5. An identifier names one key: a re-import under an existing id now replaces the slot instead of appending a
   second one (the lookup was answering with a stale key and the host cancelled the pairing).
6. The key id was stored inside the NVM3 object key, which is 20 bits: `0x48003` came back as `0x8003`. The id
   now travels in the object's payload.

Instrument fixes: the mailbox record was capped at 200 bytes and cut the fields that explained the failure
(512 now), and the host pairing step retries.

**Open:** after a reset the stack destroys two keys during initialisation (`del=2`, `dl=1`) and the lookups then
ask for `0x48003`, which the restore did not put back in the last runs. Next two measurements, one cycle each:
does that run's pairing ever import under `0x48003` (`latr=` shows the attributes of the last 16-byte import),
and what exactly does the boot delete (`dl=`)?

**Unattended work in place:** `scripts/ble-bond-cycle.sh` reproduces the whole cycle and logs to
`_tmp/ble-bond-cycle.log`; a cron runs it every three hours; an analysis dispatch for the identifier question
runs after the account window reopens (`scripts/run-bond-id-review.sh`, brief in `_tmp/review-bond-id/brief.md`).


---

## 2026-11-27 (noite) — duas revisões do Opus, e nove commits a fechá-las

**Estado:** `master` = `origin/master` = `2aca49a`, árvore limpa, gate PASS.

**O que as revisões trouxeram.** Duas passagens, arquivadas com proveniência em
`docs/04-reference/OPUS-REVIEW-2026-11-27.md` (24 achados, sessão `f65803bd`) e
`docs/04-reference/OPUS-REVIEW-2-2026-11-27.md` (17 novos, B1–B17, sessão `052d4f81`, mais as três respostas:
production-ready, publicação, próximos periféricos).

**Fechado hoje** (cada um com controlo medido, não afirmado):

| o que estava partido | commit |
|---|---|
| o gate corria testes de host só em `libraries/*/` — os 102 da HAL nunca correram | `a84d777` |
| o passo dos exemplos imprimia `N examples` contando ficheiros e construía menos: rádio 7→2, xiao 27→19, 15 no total | `863866e` |
| a feature `diag` não compilava (6 wrappers sem docs, sob `deny(missing_docs)`) | `863866e` |
| `i2c::I2c1` era público e inconstruível (4ª instância da classe) | `60a3b48` |
| o gerador arquivava o relatório do Opus como se fosse medição de um módulo | `60a3b48` |
| `--resume` era deitado fora em silêncio pelo dispatch | `ceb9182` |
| README dizia `I2C1` medido e "Nothing" não-medido; doc de outro item sobre o `i2c_on`; nota do gate a mentir sobre `ble_gatt` | `96c00b4` |
| um pin fora de gama reconfigurava **PA12** em release (`debug_assert`) | `2aca49a` |

**Aberto, por ordem de risco×custo (da 2ª revisão):**
1. **B1** dois `memory.x` — HAL 1472K, PAC 1512K; o consumidor fica com o que o linker encontrar primeiro. Um layout, no BSP.
2. **B3** a HAL impõe `critical-section-single-core` como dependência normal (o PAC acerta: dev-dependency).
3. **B2** `publish = false` existe numa crate só; as outras podem sair sem querer.
4. **B13** sem `rust-version`, sem toolchain pin, sem `Cargo.lock` — o "constrói do tarball" não é reproduzível.
5. **f5/B5/f6** a tabela de posse é por papel (USART0/EUSART1/IADC0 entregues duas vezes) e não há tipo de pin.
6. **f20** esperas de TX sem limite; timeouts engolidos.
7. **B7** o `TIMER4` só tem o controlo (não arrancado); falta um boot com ele a contar.
8. **f3/f7** `DMA` e `rtc::Rtc` continuam inalcançáveis a partir do `Board`.

**Duas lições de método, medidas hoje e registadas no código:**
- um check cujo input é uma constante escrita à mão é um verde falso — o meu próprio check dos contadores passou
  com "999 example targets" ao lado do nome do script até o controlo o apanhar;
- um documento *sobre* a árvore não é um registo *da* árvore — o gerador dos READMEs de módulo citava a revisão
  como medição; agora recusa documentos cujo nome os marca como sendo sobre a árvore.

**Pendente do operador:** o jumper **D9 (MISO, PA4) – D10 (MOSI, PA5)**, sem o qual os três masters SPI não se ouvem
(`hal_spi_masters` fica sem registo e `hal_spi_loopback` mostra `in=00..00`).

### 2026-11-27, madrugada — B1, B2, B3 e B13 fechados

    ed3a61b  um só `memory.x`, no PAC, com 40 KB reservados ao NVM3 (antes: quatro fornecedores, e o do PAC a ganhar,
             portanto a reserva ignorada em todas as imagens). Check novo no gate: `scripts/check-one-layout.py`.
    0293cfc  `publish = false` em nove crates; 380b8a6  também no PAC e na HAL, e o check passou a exigi-lo em todas.
             Ordem do operador: «nao ha publicação ate que eu diga». Tirar a linha é o acto de publicar.
    187c75b  a HAL deixa de escolher o provider de `critical-section`; a escolha passou para as aplicações (dev-deps
             dos exemplos e testes, dependency do firmware). O gate recusou três vezes e cada recusa nomeou quem vivia
             da escolha alheia.
    B13      `rust-version = "1.95"` em todos os pacotes (o que compila hoje, não uma suposição) e
             `rust-toolchain.toml` a fixar 1.95.0 + rustfmt/clippy/llvm-tools + thumbv8m.main-none-eabihf.
             `Cargo.lock` fica ignorado de propósito: é um workspace de bibliotecas, e o `cargo package` não o usa.

**Continua aberto:** f5/B5/f6 (posse por papel, pin sem tipo), f20 (TX sem limite), f3/f7 (DMA e `rtc::Rtc`
inalcançáveis), f2 (~33 blocos de doc não compilados), B4 já fechado mas o tipo de pin continua a ser a solução de
fundo, e os 6 avisos do firmware do produto.

### f6, opção B — plano decidido, não começado (2026-11-28)

Decisão do operador: **B**. O `Delay` deixa de ser dono do countdown; o board configura **um** SysTick e `delay_ms`
espera nele. Nenhuma assinatura pública muda, e o hazard morre na raiz em vez de ser proibido à porta.

Porque não foi feito nesta sessão: é uma reescrita de `crates/efr32mg2x-hal/src/delay/mod.rs` (210 linhas, três
caminhos — `delay_ns`, `delay_us`, `delay_ms`) e ainda carrega um defeito conhecido e documentado (`with_sysclk`
diz, medido 2026-11-12, que os delays saem ~2,3× mais curtos; o experimento que o resolveria está escrito em
`docs/04-reference/DELAY-CLOCK-DEFECT.md`). Fazer as duas coisas ao mesmo tempo, no fim de uma sessão longa, é como se
deixam drivers a meio — e o repositório já pagou por isso.

O plano, para a próxima sessão:

1. `delay::init(syst, hz)` — configura o SysTick **uma vez** (fonte = core, reload de 1 ms; a 19 MHz são 19 000 ticks,
   a 39 MHz 39 000, ambos dentro dos 24 bits) e marca-o com um `AtomicBool` para que um segundo `init` seja um no-op
   verificado, não uma reconfiguração silenciosa.
2. `Board::take()` chama `delay::init` uma vez. `Board::delay()` passa a devolver um `Delay` **sem** `SYST`: só o
   `Hertz`. `with_sysclk` fica como caminho de quem não usa o `Board`, com a mesma garantia de idempotência.
3. `delay_us`/`delay_ms` passam a esperar em ticks de 1 ms (`COUNTFLAG`), em vez de recarregarem o `LOAD` por chamada.
   `delay_ns` continua no `cortex_m::asm::delay` para < 1 ms.
4. Instrumento: um exemplo que crie **dois** `Delay` e meça o mesmo intervalo pelos dois, contra o SYSRTC como
   testemunha; antes do B, o segundo volta cedo; depois do B, os dois medem o mesmo. Controlo: o mesmo exemplo com um
   só handle.
5. Gate verde, e o cabeçalho do exemplo com o registo de dois boots.

### f20 — os quatro sítios exactos, e o desenho que precisa (2026-11-28)

Levantado, não corrigido. São quatro esperas sem limite, todas no caminho de transmissão:

    crates/efr32mg2x-hal/src/eusart/mod.rs:207   write_byte  `while !status.txfl()`      -- o FIFO de TX não esvazia
    crates/efr32mg2x-hal/src/usart/mod.rs:185    write_byte  `while !status.txbl()`      -- idem, USART0
    crates/efr32mg2x-hal/src/usart/mod.rs:334    flush       `while !status.txc()`       -- o último bit nunca sai
    crates/efr32mg2x-hal/src/usart/mod.rs:289    flush_receiver                              -- mesma forma, do lado da RX

Um periférico com o clock desligado, um pino mal encaminhado ou um `TXENS` que ninguém ligou deixa aí o programa para
sempre, e o chamador não tem como saber: `write_byte` devolve `()`, `write` devolve `()`.

O desenho que isto pede, e porque não é uma linha:

1. `Error` ganha `Timeout` (o enum da USART já existe; o da EUSART precisa do mesmo);
2. `write_byte`/`write`/`flush` passam a devolver `Result<(), Error>`, com o limite derivado do baud: ciclos por frame
   vezes o tamanho do FIFO, com uma folga declarada, contada em `spin_loop` -- não em tempo, para não depender do
   SysTick que o `delay` agora segura;
3. os exemplos que escrevem passam a tratar o erro (a maioria pode usar `let _ =`, mas os instrumentos têm de o pôr no
   registo -- um timeout silencioso é o defeito outra vez);
4. instrumento: limpar `CLKEN0.USART0`, escrever 8 bytes, exigir `Err(Timeout)` com os ciclos gastos no mailbox; o
   controlo é a mesma escrita com o clock ligado, que tem de devolver `Ok` e os 8 bytes no fio.

Passo 2 muda API pública em dois módulos e mexe em todos os exemplos que escrevem -- é do mesmo tamanho do `f5`, e por
isso não se faz a seguir ao `f6` na mesma sessão.

### f20, o instrumento: uma tentativa, um resultado negativo medido (2026-11-28)

    tx-timeout: control=240 written=240 refused_at=240 of=240      FAIL

A ideia era fechar o bloco em si mesmo com `CFG0.LOOPBK` e não drenar o receptor: o FIFO de RX enche, o transmissor
fica sem sítio para o frame seguinte, `TXFL` deixa de limpar, e `write_byte` devolve `Timeout`. Medido: **não**. 240
bytes atravessaram sem uma única recusa, com e sem drenagem. O loopback desta parte não faz back-pressure no
transmissor — o RX transborda (é o que o `hal_usart_link` mede do outro lado) e o TX continua a aceitar.

Consequência para o instrumento: o caminho do `Timeout` só é alcançável desligando o relógio do bloco ou não pondo
`TXENS`, e nenhum dos dois é alcançável por API pública. Ficam as duas opções já escritas:

  a) o exemplo escreve no PAC directamente — quebra a regra de que nenhum chamador nomeia `pac::*`;
  b) a HAL ganha um gancho explícito, `#[cfg(feature = "test-hooks")]`, que entrega um EUSART/USART com o relógio por
     ligar. Feio, honesto, e fora de qualquer binário de produto.

O exemplo da tentativa está em `_trash/tx-timeout-instrument-2026-11-28/hal_tx_timeout.rs` — não fica na árvore a
falhar, mas também não se apaga: é a prova de que o caminho óbvio não existe.
