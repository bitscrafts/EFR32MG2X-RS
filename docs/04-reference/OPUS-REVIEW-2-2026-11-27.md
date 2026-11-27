# The HAL, reviewed from outside — second pass, 2026-11-27

Provenance:

- reviewer: `claude --model claude-opus-5-5 --max-turns 70`, session `052d4f81-5a75-4e6f-a367-f53db6b97a62`,
  dispatched with `scripts/analysis-dispatch.sh`, subject `master` at `ceb9182` (tree clean)
- brief: `/projects/_tmp/efr32mg24-rs/opus-review-2026-11-27/brief-2.md` — three questions: is the HAL production
  ready; what stands between the tree and an honest 0.1.0 on crates.io; which five peripherals next and at what cost
- it found 17 new findings (B1–B17) and did not repeat the first pass, which it read from
  [`OPUS-REVIEW-2026-11-27.md`](OPUS-REVIEW-2026-11-27.md)
- **the findings are in the message, not in the file**: the brief said `findings.md`, the dispatch allowed
  `findings2.md`, and the reviewer — correctly — refused to guess and wrote nothing. The mechanism worked (probe:
  `Edit(//…/file/**)` appends), the brief named the wrong file. A brief that names an output path must name the real
  one, or the dispatch must inject it.
- **this is a self-report.** It corrects this session's own claims in its closing section, and one of those corrections
  is accepted below.

Corrections to this session's work that it established, accepted and fixed on the spot:

| its finding | what was wrong | fixed |
|---|---|---|
| B6 | the HAL README still lists `I2C1` as measured and says "Nothing" is unmeasured, while `board.rs` says the bus is unmeasured | template corrected, README regenerated |
| B10 | a stray `///` line and `#[must_use]` belonging to another item sat on top of `i2c_on`'s docs | removed |
| B11 | the gate's closing note still says `ble_gatt` is not built, while the examples step now builds it | note removed |
| — | `board.rs` called the I2C1 fix "the third of its class"; the commit says the fourth | the file now says the fourth |

---

## The reviewer's answer (verbatim)

I couldn't write `findings.md`: the brief gives no path, this session can only touch the repository, and `analysis-dispatch.sh:38-39` refuses a findings file inside it. The findings are below in your format instead. Nothing was built, flashed, edited or committed, and I used about 50 tool calls.

# Findings, second pass (new only; f1–f24 are not repeated)

## B1 — the HAL and the PAC each put a different `memory.x` on every downstream link
where: `crates/efr32mg2x-hal/build.rs:14,20`, `crates/efr32mg2x-pac/build.rs:11-21`. The HAL's `memory.x` has `FLASH … LENGTH = 1472K`; the PAC's has `LENGTH = 1512K`.
what / why: both build scripts add their own `OUT_DIR` to the link search path, and both write a file called `memory.x`. A consumer gets whichever one the linker finds first. One layout reserves the top 40 KB of flash and the other doesn't, which is the difference between leaving NVM3 alone and overwriting it. Both crates also force the XIAO/Arduino bootloader offset `0x08006000` onto every MG24 user. `gate.sh:191` only checks that the origin is at or above `0x08006000`.
fix: one layout, kept in the BSP (or behind a `xiao-layout` feature on the PAC). The HAL ships none.
confidence: verified-by-reading that the two files differ; *inferred* which one the linker picks.

## B2 — `publish = false` protects one hardware crate, not the vendor-blob ones
where: `crates/efr32mg24-emlib-sys/Cargo.toml:7` is the only one in `crates/`.
what / why: `rail-sys`, `bluetooth-sys`, `radio`, both BSPs, `cmsis-stubs`, `embassy-efr32` and the bootloader crates can all reach `cargo publish`. Only a missing token stopped the 2026-11-15 attempt.
fix: set `publish = false` on every crate not listed in `check-publishable.py:CRATES`, and have the gate fail any that lacks it.
confidence: verified-by-reading.

## B3 — the HAL library chooses the critical-section implementation for its users
where: `crates/efr32mg2x-hal/Cargo.toml:14` enables `features = ["critical-section-single-core"]` as a normal dependency. The PAC gets this right and puts the same line under dev-dependencies (`efr32mg2x-pac/Cargo.toml:45-47`).
why: an application that brings its own implementation (an RTOS, a TrustZone setup) gets a duplicate-symbol link error.
fix: move it to `[dev-dependencies]`, as the PAC does.
confidence: verified-by-reading; the link error is inferred from how `critical-section` works.

## B4 — an out-of-range pin configures a different pin in release builds
where: `src/pin.rs:46-49,71`, reached from `board.rs:1358-1370`.
what: the only guard is `debug_assert!(pin < 16)`, and every gate and example build is `--release`. `output_on(Port::A, 20, …)` goes to `MODEH` with shift `4*(20%8)` and rewrites **PA12's** mode. `Port::B` pin 9 is silently ignored.
fix: return `Err`, or take a checked pin type.
confidence: verified-by-reading.

## B5 — pins have no owner, and CLAUDE.md's type-state pin does not exist
where: `board.rs:1353-1404` (`i2c_on`, `output_on`, `input_on`, `watch_edges`, `count_edges` take any `(port, pin)` with no claim). `grep 'struct Pin<'` over the HAL finds nothing.
what: CLAUDE.md says *"Type-safe pin modes (compile-time enforcement) `Pin<PORT, PIN, Input<Pull>>`"*. Nothing stops one pin being handed out as an output, an I2C line and an edge watcher at once. This is f5's defect at the pin level.
confidence: verified-by-reading.

## B6 — the HAL README still lists I2C1 as measured, and says "Nothing" is unmeasured
where: `crates/efr32mg2x-hal/README.md:177` (`I2C0`, `I2C1` | … | `hal_i2c_hardware_imu`…) and `:193` ("Nothing. Every instance the crate hands out has a record on the part").
what: `board.rs:117` says I2C1 *"is **unmeasured**"*. Commit 60a3b48 changed 2 lines of this README and left both of these. f17 is still open, and is now wider.
confidence: verified-by-reading.

## B7 — TIMER4 has no positive measurement
where: `README.md:173` and `COVERAGE-BOUNDARY:39` say *"each built at 1000 Hz and its **overflows** counted"*. `TIMER-INSTANCES-2026-11-27.md:56`: *"`o4=0` | **control**: `TIMER4` was constructed and never started"*.
what: TIMER4 has only been shown not to count when it isn't started.
fix: run a second boot with TIMER4 counting and another instance as the control.
confidence: verified-by-reading.

## B8 — `Board::i2c0`/`i2c1` ask the caller for a clock the board already knows
where: `board.rs:86-89`: *"the `Board` … does not own a `FrozenClocks`, so it would have to guess the frequency"*. The same file has `board.rs:1319 clock_hz()`, and `delay()` uses it at `:1339`. That fix was made precisely because a guessed clock was wrong by the boost ratio.
why: a wrong `hfclk` gives a wrong bus rate, which shows up as a device that never acknowledges. That is the failure mode the doc itself describes.
fix: drop the parameter. If I2C runs off the peripheral clock rather than the system clock (I didn't check this), add a `pclk_hz()`.
confidence: verified-by-reading.

## B9 — secure and non-secure aliases are mixed, and I2C0 is reached through both
where: `board.rs:103` (`I2c0Ns`), `i2c/async_impl.rs:153,174` (`I2c0S`). Across the HAL, `steal()` goes through the non-secure alias for I2C0, SYSRTC0, BURTC, HFXO0, GPCRC, LETIMER0, MVP, PCNT0 and WDOG0, and through the secure alias for everything else.
why: this is harmless until the SMU is configured, which Phase E's Secure Vault work implies. From then on, every non-secure-alias driver is a likely bus fault.
fix: one alias policy, enforced by the gate.
confidence: inferred (the risk); the mix is verified.

## B10 — `i2c_on`'s docs.rs summary is another item's sentence
where: `board.rs:1347-1349`: *"The GPIO register block this board's pins live on."* plus a stray `#[must_use]`, sitting directly on top of `i2c_on`'s own doc comment.
confidence: verified-by-reading.

## B11 — the gate's closing lines contradict its examples step
where: `gate.sh:627` prints *"efr32mg24-radio/ble_gatt -- … not gated on purpose"*. The examples step at `:390-416` now builds every example with its declared features (the brief reports radio 7/7).
fix: delete the line, or name what is actually still excluded.
confidence: verified-by-reading. The 7/7 figure comes from the brief; I didn't build.

## B12 — nothing fails on broken intra-doc links or rustdoc warnings
where: no `cargo doc` call in `scripts/`. `.github/workflows/docs.yml:41-50` sets `continue-on-error: true`, and `:60-68` passes no `-D warnings`. Both use `--features rt`, while docs.rs builds the default features (`mg24` only).
confidence: verified-by-reading.

## B13 — no MSRV, no toolchain pin, no tracked `Cargo.lock`
where: no `rust-version` in the workspace, HAL or PAC manifests. `git ls-files` returns neither `rust-toolchain*` nor `Cargo.lock`. Every workflow uses `dtolnay/rust-toolchain@stable`.
why: HAL-MATURITY's "packaged crate BUILDS from the tarball" (`:24`) can't be reproduced, because the dependency versions float.
confidence: verified-by-reading.

## B14 — the in-tree PAC says 0.1.1 but is no longer the 0.1.1 on crates.io
where: `git diff --stat 3bb82bd HEAD -- crates/efr32mg2x-pac` shows README, `docs/crate-doc.md` and `src/lib.rs` (+`deny(missing_docs)`) changed since the release commit.
why: the HAL's `version = "0.1.1"` will verify against the registry copy, not this one. Today the API is the same; nothing checks that it stays so.
confidence: verified for the diff; *inferred* that 3bb82bd is the uploaded tree.

## B15 — the coverage boundary leaves out two families that matter to a product
where: `COVERAGE-BOUNDARY:56-69` (§2) and §3. SEMAILBOX is in the PAC (18 matches) and isn't listed. Flash programming (MSC write/erase) isn't listed either; `:68` mentions MSC only for wait states.
confidence: verified-by-reading.

## B16 — `i2c::on_interrupt` acts on I2C0 for anyone, and its public docs are in Portuguese
where: `i2c/async_impl.rs:167-181`.
what: it is a safe free function that steals I2C0 and writes `IEN = 0` whether or not the caller owns the bus. Its doc comment is Portuguese, so docs.rs readers get mixed languages.
confidence: verified-by-reading.

## B17 — `board.rs` is 1573 lines against the project's 400-line rule
where: `crates/efr32mg2x-hal/src/board.rs`.
why: every handout, route and clock boost is in one file, which is where f3, f5, f6 and B8 all live.
confidence: verified.

# 1. Is the HAL production ready?

**No.** The bar I'm applying is the brief's six points:
1. A stranger's crate builds, links and runs.
2. Every public driver can be obtained from `Board`.
3. Every README claim has a record behind it.
4. The coverage boundary neither overstates nor understates.
5. Every gate check can fail on the defect it names.
6. Misuse produces an `Err`, never a hang or a silent wrong configuration.

How the tree fails each one:
- **1:** B1, B3, B13.
- **2:** f3 (DMA) and f7 (`rtc::Rtc`) are still unreachable.
- **3:** B6, B7, f17.
- **4:** B7, B15.
- **5:** f23, B11, B12.
- **6:** f5, f6, B4, B5, f20.

**Ranked by risk to a consumer × cost to close (cheapest high-risk first):**

| # | gap | risk | cost |
|---|---|---|---|
| 1 | B4 out-of-range pin configures the wrong pin | high (silent) | hours |
| 2 | B1 two `memory.x` files, B3 forced critical-section impl | high (wrong image, link failure) | ½ day |
| 3 | f5 claim table keyed by role + B5 no pin ownership + f6 SysTick owners | high (aliased peripherals) | 1–2 days |
| 4 | f20 unbounded transmit waits, swallowed timeouts | high (hang) | 1 day |
| 5 | B6, B7, f17 README claims | medium (trust) | hours |
| 6 | f3 DMA beside `Board`, f7 `rtc::Rtc` | medium (feature unusable) | 1 day |
| 7 | B8 caller-supplied clock, B9 alias mix | medium, latent | ½–2 days |

**The three that would turn "no" into "yes", each with a one-board instrument:**
1. **Ownership is sound (f5, B4, B5, f6).** Extend `hal_one_of_each` to try every cross-role pair (`usart0` then `spi0_on`, `adc` then `converter`, `eusart1` then `spi2_on`), plus pin 20 and a pin claimed twice. Each must return `Taken` or `Err`, recorded in the mailbox. Add a host test that every slot maps to exactly one hardware block.
2. **Nothing hangs (f20).** A fault-injection example: clear the EUSART clock-enable bit, then `write` and `flush`; route I2C1 to unconnected pads and do a read. Both must return `Err(Timeout)`, with the elapsed SYSRTC ticks recorded. The control is the same calls with the clock on.
3. **A stranger can link it (B1, B3, B13).** Add a gate fixture: a consumer crate outside the workspace that depends on the `cargo package` tarballs and brings its own `memory.x` with a distinctive length. The gate asserts from the ELF section headers that its layout won, and that it links with a non-default critical-section implementation. Flash that image once to show a blink.

# 2. Between this tree and an honest 0.1.0

In the order a publisher hits each item:

| # | item | status |
|---|---|---|
| 1 | Only the intended crates can be published (B2) | **neither** |
| 2 | Version and name are free (HAL deleted, registry cooldown) | **documented** (HAL-MATURITY:12-19). Whether 0.1.0 can be reused after a deletion can't be settled from the tree; the PAC's move to 0.1.1 suggests caution |
| 3 | Package contents listed | **checked** (`check-publishable.py:55-63`, `gate.sh:176`). No allow-list of what may ship |
| 4 | Package builds from its tarball | **documented** once (HAL-MATURITY:24, 2026-11-25), **not re-run** |
| 5 | Relative links in the README | **checked** |
| 6 | Relative links in other shipped `.md` files (f14) | **neither** |
| 7 | Intra-doc links, rustdoc warnings (B12) | **neither** |
| 8 | README generated from its template | **checked** (`gate.sh:147-153`) |
| 9 | README claims match records (B6, B7) | **neither** |
| 10 | HAL Quick Start compiles (f24c) | **neither** |
| 11 | docs.rs build (default features, thumb target) | **documented** (`[package.metadata.docs.rs]`), never simulated |
| 12 | Feature matrix | **partly checked.** The HAL has only `mg24` (default) and `rt`. Tested: `rt` on thumb, default on the host. Not tested: default on thumb (what docs.rs builds); `--no-default-features` should fail and nothing asserts it |
| 13 | MSRV / `rust-version` (B13) | **neither** |
| 14 | Licence files | present in both crate directories; **not checked** that they ship |
| 15 | Metadata (repository, keywords ≤ 5, categories) | present; **not checked** before upload |
| 16 | Path dependency matches its registry version (B14) | **neither** |
| 17 | No linker or critical-section policy imposed (B1, B3) | **neither** (`gate.sh:191` checks only the origin) |
| 18 | Semver story | **documented** (CHANGELOG; HAL-MATURITY:102 "first publication is 0.1.0"); no `cargo-semver-checks`, no stated policy for 0.x |
| 19 | Lockfile, toolchain pin (B13) | **neither** |

# 3. The next five peripherals or modes

Ranked by value to a real product on this part × how cheap the proof is.

1. **LDMA handed out by `Board`, with EUSART RX/TX through it.**
   - Gain: serial transfers without the CPU, in a program that also uses `Board`.
   - One-board proof: EUSART1 internal loopback (routed, since the loop closes through the pad path) into a 64-byte buffer via LDMA, byte-compared. The control is the same with the DMA request unselected.
   - Needs a jumper: nothing.
   - Defect class closed: "public driver nobody can construct" (f3).
   - Size: 2 days. Not split: the slot for the channel and the handout.
2. **EM4 hibernate, woken by BURTC, with BURAM retention.**
   - Gain: months on a coin cell.
   - One-board proof: write BURAM, enter EM4, BURTC compare wakes the part. Record the reset cause (EM4) and the BURAM word that survived. The control is a pin reset.
   - Needs an observer: current draw needs a meter (deferred).
   - Defect class closed: BURAM is an absent family, and "survives a reset" is currently proved only for a core reset.
   - Size: 2 days.
3. **SE mailbox: TRNG and SHA-256.**
   - Gain: real entropy for keys and nonces, and hashing.
   - One-board proof: the SHA-256 known answer for "abc", plus TRNG health checks (no repeated word in 4 KB, monobit count within bounds). The control is a bad command code, which must come back as a mailbox error.
   - Needs an observer: nothing.
   - Defect class closed: absent from the boundary (B15), and errors surfaced as values rather than hangs.
   - Size: 4 days. Not split: the mailbox transport.
4. **Timer input capture, and PWM on TIMER1–TIMER4.**
   - Gain: frequency and pulse measurement, more PWM outputs.
   - One-board proof: TIMER0 PWM routed over PRS into TIMER1's capture input, with the measured period equal to TOP+1. The control is PRS off, giving no capture. This also gives TIMER4 the positive record B7 lacks.
   - Needs an observer: PWM on a pad, checked with a scope (deferred).
   - Defect class closed: "decision layer host-tested, register path absent", and "one instance hard-coded".
   - Size: 2 days.
5. **MSC flash page program/erase.**
   - Gain: storage without the NVM3 blob.
   - One-board proof: unlock, erase and write a page, read it back across a reset. The control is a write without the unlock, which must be refused.
   - Size: 2–3 days.

**The one I would not do: #5 (MSC).** Its only spare pages are the region B1 shows the two layouts disagree about, which is where NVM3 keeps bonds. One wrong address bricks the bootloader or wipes pairing state. It also duplicates storage the product already has. Fix B1 first; after that it becomes a decision rather than a risk.

# Where this brief is wrong

- **The first pass isn't in this session.** I read `docs/04-reference/OPUS-REVIEW-2026-11-27.md` instead.
- **"findings.md in this run's directory":** no path was given, and this session can't write outside the repository, where the dispatch script refuses the file.
- **"`cargo package --list` is not run by any check here":** it is, at `check-publishable.py:55-63`, called from `gate.sh:176`.
- **"`efr32mg2x-hal` and `efr32mg2x-pac` are private today":** the PAC is published (0.1.1, HAL-MATURITY:13). Neither crate has `publish = false`.
- **The feature matrix "`rt`, `sense`, `ble`, `diag`, `gatt`":** only `rt` (and `mg24`) are HAL features. `sense` belongs to the XIAO BSP; `ble`, `diag` and `gatt` to the radio crate.
- **"I2C1 … Unmeasured bus, said so":** said in the method's doc comment only. The README still claims it as measured (B6).
- **TIMER4 listed as driven:** its only reading is the not-started control (B7).
- **The boundary's §2 and §3 as the list of what's absent:** they omit SEMAILBOX and flash programming (B15).
- **"Four instances of one class":** `board.rs:115` calls the I2C1 fix "the third"; the commit message says "the fourth".
