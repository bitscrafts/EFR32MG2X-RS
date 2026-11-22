# HANDOFF — EFR32MG2X-RS (XIAO MG24 Sense + Expansion Base)

**State: green.** Branch `master`, pushed to `origin` (`bitscrafts/EFR32MG2X-RS`). The Expansion Base
was found faulty on 2026-11-15 and is away for replacement (about a month); that does not remove
anything already measured with it — those dates stand — and the examples that need the carrier say so
until another one is on the bench. The board left running `hal_led_blink`, its LED confirmed blinking.

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
3. **The on-chip I2C0 master cannot complete a read after a write.** Both buses are
   bit-banged (`docs/04-reference/I2C0-READ-DEFECT.md`).
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
