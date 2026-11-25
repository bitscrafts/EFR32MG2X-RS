# The HAL, read by an outside reviewer — brief and the path to A

**Status: a working brief, not a review.** This document exists so a frontier model that has never seen this repository can
judge the crate in one sitting, and so the work that would move every dimension from B to A is written down somewhere
other than a chat log. It is **not** a publication decision: publishing needs an express order from the operator, and
nothing here changes that.

State at the time of writing: `master` = `cadfe80` (+ the `wdog` `SyncTimeout` change), gate `VERDICT PASS`, 108 examples
build across 9 crates, 197 `unsafe` sites all annotated.

---

## 1. What the crate is

`efr32mg2x-hal` is a `no_std` hardware abstraction layer for the Silicon Labs **EFR32MG24B220F1536IM48** as fitted to the
Seeed **XIAO MG24 Sense**. It sits on a hand-checked PAC (`efr32mg2x-pac`, published as 0.1.1) and is driven on a real board:
27 public modules, ~12 500 lines, 23 examples, each example a small instrument that writes a verdict into a mailbox in RAM
that a host reads back over the debugger.

The crate's own rule, which most of the design follows from: **a claim in a document must be a record read back off the
board.** A number with a date means somebody flashed that firmware and read the answer back. A number without a date is
code that compiles.

What is proven on the part today (records verbatim, 2026-11-23 → 11-25):

```
i2c-rec: antes=0x6a erro=1 depois=0x6a wr=1 rd6=1 nz=1 volta=0x40      PASS   I2C0 survives a NACK
dma-stop: st0=0 armed=1 stA=1 busyA=0 stS=0 ctl=1 stRe=0 stS2=0        PASS   stop() really stops
wdog: fed 116 st80000000 sb0 lock1                                     PASS   WDOG feeds across the domain crossing
emu: VS2->VS1 5/5 , VS1->VS2 5/5                                        PASS   voltage scaling both ways
cmp: start=…57 deadline=…77 reached=…78 late=1 fired=1 en=1 run=1      PASS   SYSRTC compare fires once, late=1
mic 4 rms=5 pk=21 n=128                                                PASS   IADC + analogue microphone
imu 6a x-351 226 940 |a|=1028                                           PASS   IMU over I2C
hfxo: sel=1 before=19M after=39M tick=32974 err=206                    PASS   crystal, and a 1000 ms delay = 1006 ms
adc: gnd=0 ch0=0 ref=VBGR                                              PASS   single-shot, ground channel
spi: n=8 ok=8 err=0 out=a5..34 in=a5..34                               PASS   SPI on a D10-D9 jumper
spi: n=8 ok=1 err=0 out=a5..34 in=ff..ff                               FAIL   …the same firmware with MISO moved off the jumper
usart: baud=115200 out=6 in=6 ok=6 45 46 52 33 32 21                   PASS   USART on a D6-D7 jumper
one-of-each: sysrtc=1 sysrtc2=1 adc=1 adc2=1 wdog=1                    PASS   the board refuses a second hand-out
buzzer tick=19000000Hz (sysclk=19000000) top=3518 tone=2700Hz          PASS   TIMER0 + CC0, calibrated
dwt: demcr=0x01100000 ctrl=0x40000001                                  probe  the counter runs only under a debug session
```

The two `spi` lines are one instrument and its **control**: they differ only in where MISO is routed, which is what makes
the PASS mean something. That habit — a near-miss that must fail — is the one this project holds highest.

## 2. The grading rubric, as applied

Ten dimensions, each graded A to F by the same standard: **A needs the property to be enforced by a check or by the type
system and demonstrated on hardware where hardware is involved; B means it is true and written down but a person has to
keep it true; C means it is true today and nothing keeps it true.**

| dimension | grade | the sentence behind it |
|---|---|---|
| functional coverage | B | 26 modules, 15 peripheral families with a driver, every one exercised by an example; GPIO interrupts, slave modes and DMA linked descriptors are not implemented |
| hardware evidence | B | 15 of 23 examples carry a dated record and every family has one; `hal_mvp_bench`'s numbers are `dwt`-based and only valid under a probe |
| documentation | B | `deny(missing_docs)` on all 26 modules, no prose pointing at files that do not ship, README generated from the tree and checked |
| API hygiene | B | the singleton claim is enforced at run time and measured; the pure layer is private; the compiler still cannot distinguish two handles |
| safety | B | 197 `unsafe` sites, each with the invariant it relies on, enforced by `scripts/check-safety-notes.py` in the gate |
| testing | B | 162 host tests across 19 core crates, and every example of every crate compiles in CI (107 → 108 after this week) |
| packaging | A− | one registry-resolvable dependency, package builds from its tarball, no path escaping the package, docs.rs metadata |
| error handling | B | typed errors, bounded waits; a timeout no longer reports itself as a lock (`wdog::Error::SyncTimeout`) |
| observability | A− | the mailbox record pattern, per-driver `State` snapshots; every example duplicates its own formatting |
| publication hygiene | B | `check-publishable.py` reads the package; `pre-publish.sh` packages instead of dry-running a publish |

## 3. What the path from B to A is, dimension by dimension

Each entry says what A means here, what it costs, and **what evidence would prove it**. A reviewer should feel free to
disagree with the ordering; the point of writing it down is that the argument can be had about a list instead of a memory.

### 3.1 Functional coverage: B → A

**A means**: the peripheral set a product on this board needs is complete and proven, and what is missing is missing on
purpose and written down.

Work, in the order that unblocks the most:

1. **GPIO interrupts** (`EXTI`): nothing in the crate can wake on a pin change. Needed by any button, any motion device,
   any low-power product. Cost: one module, the `EXTI`/`IF`/`IEN` registers, a debounce decision, an example with a
   JumperWire as the instrument. ≈ 2 days.
2. **I2C slave / SPI slave**: a product that is a sensor node needs at least one of them. Cost: a state machine on top of
   the existing register layer; the EUSART slave path has no measured example anywhere in this repository. ≈ 3 days.
3. **DMA linked descriptors** (Stage 3 of the DMA plan): the only way to chain transfers without the core. Cost: the
   descriptor list and the `LINK` field, plus an example that proves the chain. ≈ 1 day.
4. **Radio, honestly scoped**: BLE works (a phone reads the BME680 over GATT), but it lives in `efr32mg24-radio` and the
   Bluetooth stack, not in this crate. A is not "add Zigbee" — it is "say precisely where this crate stops".

**Evidence that would prove A**: one dated record per family *including the new ones*, and a table in the README whose rows
are generated from the examples themselves rather than written by hand.

### 3.2 Hardware evidence: B → A

**A means**: every claim in the package traces to a record, the instrument behind each record has a control that fails,
and the whole set is repeatable.

1. **A second, independent witness for the two loopback proofs.** A loopback proves the driver, not that a *different
   machine* can use the wire. An FT232 on D6/D7 (USART) and a real SPI device on D8-D10 would each turn a self-test into
   an external one. Cost: one afternoon with the bench, one example each.
2. ~~**`hal_mvp_bench` re-measured without `dwt`.**~~ **Done, 2026-11-25.** The clock is the SYSRTC and the measurement
   is a batch of 256 additions so one 30.5 µs tick is a fraction of a percent of the total: `PASS mvp: n=64 reps=256
   tick_mvp=123 tick_cpu=288 mvp_us_x100=1466 cpu_us_x100=3433 ratio_x100=234`, four boots within one tick, no probe
   attached. Worth noting against the old DWT record: it claimed 921 cycles for the Rust loop where the SYSRTC measures
   34.3 µs (~652 cycles at 19 MHz), so the probe-era number was not merely unrepeatable, it was 40 % high.
3. **Repeatability**: every instrument run N times (5 is enough) and the record carries `n/N`. `hal_emu_voltage` already
   does this (`VS2->VS1 5/5`); the rest do not. Cost: mechanical.
4. **Archive the logs, not just the verdict.** `_tmp/…` holds the flash and mailbox transcripts for this week; they are
   not in the repository. A published crate does not need them, but a review does.

**Evidence that would prove A**: an `EVIDENCE.md` in the package where every row is a record, a date, an example name and
a control, generated from the examples' own headers by a script (the same way the README is generated today).

### 3.3 Documentation: B → A

**A means**: a stranger can use the crate without reading its source, and the documentation cannot drift from the tree
because a check would fail.

1. **Doc-tests.** *Started, 2026-11-25.* The gate's host-test loop now runs `cargo test --doc` per crate, and a
   deliberately wrong assertion was measured turning it red. Three doc-tests exist (`record-core` 2, `slots-core` 1);
   the other eighteen crates are honest zeros. Finishing means writing the examples for the pure API of every core crate
   — the place a consumer copies from — which is content, not mechanism.
2. **Per-module "what it does not do"**. Several modules say what they do; the honest half — the limitations table — is
   only in this assessment. Move it into each module header.
3. **A generated evidence table**, as in 3.2.
4. **A migration note per API change** (`Board::x` now returns `Result`): a `docs/CHANGES-FOR-0.1.0.md` listing every
   signature that moved this week, with the one-line fix at the call site.

**Evidence that would prove A**: doc-tests in the gate, a generated README/evidence pair that `--check` verifies, and no
document referencing a path that is not in the package (already true; the check exists).

### 3.4 API hygiene: B → A

**A means**: the properties the crate claims are properties the compiler enforces.

1. **`Board` owns its peripherals and moves them into drivers.** This is the redesign F10 named: `Board::take()` destructures
   `pac::Peripherals` into `Option<T>` fields, and each `Board::x()` does `self.wdog.take()` — a second request is
   impossible to write, not merely refused at run time. Cost: every driver constructor plus every example; ≈ 2 days, and it
   is the single largest API break left. The claim table (`cores::slots`) is what makes the current state honest; this is
   what makes it unnecessary.
2. **`#[non_exhaustive]` on every public error enum** (`wdog::Error`, `usart::Error`, …) so adding a variant is not a
   breaking change.
3. **`cargo-semver-checks`** in the gate against the last published version of the PAC, and against the last tagged HAL.
4. **No panics in library paths**: `expect`/`unwrap` live in examples only; the crate's own code returns errors. Audit with
   `grep -n 'expect(\|unwrap(' src/` and justify each.

**Evidence that would prove A**: the `Option`-moving `Board` with a compile-fail test (`trybuild`) that a second
`board.sysrtc()` does not compile, plus semver-checks green in the gate.

### 3.5 Safety: B → A

**A means**: the invariants are enforced or, where they cannot be, they are written once in one place and mechanically
checked.

1. **No `steal()` outside `Board`.** Today `pwm.rs`, `sleep.rs`, `dwt.rs` and `dma` still reach for `core::Peripherals` and
   PAC blocks directly. Moving those behind the board removes the largest class of aliasing.
2. **Invariants per peripheral in one document**, and each `SAFETY` note cites it (today each note re-explains its field
   width — correct but repeated 197 times).
3. **A `miri`-style audit is not applicable** (bare metal), but `cargo geiger` is: it produces the unsafe count per
   dependency and per module, which is the number a reviewer wants next to the notes.

**Evidence**: a gate check that `steal()` appears only in `board.rs`, and the per-peripheral invariant document.

### 3.6 Testing: B → A

**A means**: every defect found becomes a test that fails if it comes back, and the hardware path is exercised without a
human.

1. **A regression test per defect found this week** (the buzzer's three, the `dwt` trap, the receiver overrun). The core
   crates are the right home: the overrun is a `usart-core` arithmetic property, the `CC0_CFG` order is a `timer-core`
   state-machine property.
2. **Hardware in CI**: a nightly job that flashes the instruments and reads the mailbox, with the failure mode stored. The
   bench is a laptop and a USB cable; the missing piece is that nobody has written the job.
3. **Coverage per core crate** (`llvm-cov`), not per line of the HAL (which is register writes).

**Evidence**: a `tests/` entry per defect with its ticket number in the name, and a nightly run with the mailbox records as
artifacts.

### 3.7 Packaging: A− → A

1. **MSRV pinned** in `Cargo.toml` (`rust-version`) and checked in CI.
2. **`cargo package` from a clean clone** as the gate's last step (today it runs in-tree).
3. **LICENSE files in the package** (the licence is declared, the text is not shipped).
4. **A version policy**: what 0.1.x means, what breaks a minor.

### 3.8 Error handling: B → A

1. **Every variant reachable and covered by a test.** `wdog::Error::AlreadyEnabled` — is it reachable? If not, it is a
   variant that lies in a different way than `Locked` did.
2. **`impl core::error::Error`** for each error type (stable since 1.81), so `?` composes across crates.
3. **Errors that carry the offending value**: `InvalidBaudrate` says nothing; `InvalidBaudrate { asked: u32, max: u32 }`
   says what to do next.

### 3.9 Observability: A− → A

1. **The mailbox record as a library facility** (`hal::record`), so 22 examples stop re-implementing `Line` (the same
   `u`, `s`, `h` helpers exist 22 times).
2. **A `defmt` feature** behind the same records, so RTT and the mailbox are two transports of one thing.
3. **A failure taxonomy**: what a reader should conclude from `last=00`, from `in=0`, from `ok=1`.

### 3.10 Publication hygiene: B → A

1. **Run the checklist once, end to end**, on a tree that is about to be published: `check-publishable`, `cargo package`,
   read every file that ships, `docs.rs` metadata.
2. **Verify the name cooldown with one attempt** (the PAC waited until 2026-11-16); the refusal is read, not assumed.

## 4. What a reviewer should be asked

1. Is the grading rubric honest, or is it grading the project on its own terms? Specifically: is "every family has one
   dated record" really a B, or is it a C dressed up?
2. Which of the ten dimensions would you withhold an A from a `0.1.0` for, and why?
3. The `Board` redesign in 3.4 — is moving `Option<T>` per peripheral the right shape, or is `&mut self` + a guard the
   cheaper honest answer for a crate this size?
4. Is the claim table (`cores::slots`) worth keeping once the `Option` move lands, or is it dead weight?
5. The examples double as tests. Is that a virtue (they run on hardware) or a trap (a test that needs a bench is a test
   nobody runs in CI)?

## 5. Files a reviewer wants open

- `crates/efr32mg2x-hal/src/board.rs` — every hand-out, and the claim table's entry points.
- `crates/efr32mg2x-hal/src/cores/slots.rs` — the claim table, with its tests.
- `crates/efr32mg2x-hal/src/pwm.rs` — three defects fixed in one week; the comments say which and why.
- `crates/efr32mg2x-hal/src/dwt.rs` — an instrument that refuses to lie.
- `crates/efr32mg2x-hal/examples/hal_spi_loopback.rs` and `hal_usart_link.rs` — the instruments, with their controls.
- `scripts/gate.sh`, `scripts/check-safety-notes.py`, `scripts/check-publishable.py` — the enforcement.
- `docs/04-reference/HAL-MATURITY-2026-11-25.md` — the long assessment this brief summarises.