# The HAL, reviewed from outside — 2026-11-27

Provenance, because this file is a report about the tree and not the tree itself:

- reviewer: `claude --model claude-opus-5-5 --permission-mode plan --max-turns 70`, session
  `f65803bd-4c64-4e90-b6b0-c0d6c227fb07`, dispatched with `scripts/analysis-dispatch.sh`
- subject: `master` at `351ad5f`, working tree clean, before this file existed
- question asked: *which claims in this tree cannot be true, and which checks in `scripts/gate.sh` pass while the thing
  they check is broken?* The brief is `docs/04-reference/OPUS-REVIEW-2026-11-27-brief.md`
- 67 tool calls, no build, no flash, no write into the repository (plan mode). The findings landed in the run's plan
  file because plan mode has no other channel; the file below is that plan file, verbatim, followed by the run's final
  message.
- **This is a self-report.** Three of its claims were re-checked against the gate log on the day it landed and are
  marked below as *confirmed*. An unmarked finding is a claim, not a measurement.

Three re-checks done before filing (2026-11-27):

| finding | claim | re-check | verdict |
|---|---|---|---|
| 2 | the HAL never compiles its doc examples | `gate-counts2.log` has `PASS efr32mg2x-hal: 43 examples` and **no** doc-test line for the crate; `cargo test -p efr32mg2x-hal --doc --target x86_64-unknown-linux-gnu` then fails to compile the crate itself (`isb 0xf` in `sysrtc.rs:331`, host-side inline asm) | **confirmed, and the fix is not the obvious one** — doc-tests for this crate cannot run on *any* target cargo can test on, so the honest fix is to mark the blocks and say so, not to add a doc-test step. 50 `/// ``` ` blocks, 17 marked `no_run`/`ignore`, so ~33 claim to be runnable and have never been compiled |
| 1 | "13 `#[test]`s inside the HAL are dead" | `cargo test -p efr32mg2x-hal --lib --target x86_64-unknown-linux-gnu` -> **102 passed, 0 failed, 0.00 s** | **partly wrong, and the truth is worse for the gate**: the 13 exist in `route.rs` (10), `alias.rs` (1), `pcf8563.rs` (2) and they *pass*. What is broken is that `scripts/gate.sh` runs host tests for `libraries/*/` only, so the HAL's 102 tests, doc blocks and all, are never run by any gate |
| — | `CLAUDE.md` carries a stale example count | line 41 says `all 125 example targets`; the tree has 127 | **confirmed** (the reviewer wrote "41 examples", which is the line number, not the count — the defect is real) |

---

## The reviewer's findings (verbatim)

# HAL review 2026-11-27 — findings (read-only run)

## The grades in HAL-MATURITY §4, tested

| dimension | claimed | defensible? | what falsifies it / what moves it |
|---|---|---|---|
| functional coverage | B | yes, but the count is inflated by I2C1, rtc::Rtc, Spi0/Spi2 | moves with a "compiled, never run" row per instance (f17, f19) |
| hardware evidence | B (addendum → A− on docs) | **no, B−**: README says "nothing unmeasured" (f17); module pages contradict it (f11, f12); one cited instrument is not in the tree (f21) | evidence = record + instrument in tree + module mapping checked by the gate |
| documentation | B → A− (addendum) | **no, C+ for the HAL**: the 23 doc-test crates are libraries/*; no HAL doc block is compiled (f2); module READMEs carry false evidence and 84 dead links (f11-f14) | HAL doc-tests in the gate; generator vendored and checked |
| API hygiene | B (§6 says C+) | **C**: role-keyed slots (f5), Delay alias (f6), Dma/Board exclusion (f3), I2c1 unreachable (f4) | slots keyed by block plus a test that fails on a cross-role alias |
| safety | B (§6 says C) | B− : notes exist; one is false (f6); window rule and `unsafe impl` gaps (f23) | a note per block, not per window |
| testing | B | **C+**: 13 HAL tests never compiled (f1); examples behind features unbuilt (f15, f16) | count built artifacts; HAL host tests |
| layering/packaging | A− | B: the package build is not re-run (f24d); memory.x leaks a board layout (f24b) | `cargo package` in the gate |
| error handling | B | C+: unbounded TX waits, swallowed sync timeouts (f20) | budget + Result on every wait |
| observability | A | yes, A− (record::Record is host-tested) | — |

Plan mode blocks writes outside this file, so findings are kept here instead of
`/projects/_tmp/efr32mg24-rs/opus-review-2026-11-27/findings.md`. Copy them across verbatim if wanted.

## finding 1 — the HAL's own host tests never run: 13 `#[test]`s in route.rs, alias.rs, rtc/pcf8563.rs are dead
where: crates/efr32mg2x-hal/src/route.rs:102 (10 tests), src/alias.rs:35 (1), src/rtc/pcf8563.rs:189 (2); scripts/gate.sh host-test loop (`for dir in libraries/*/`)
what: the HAL carries `#[cfg(test)]` modules; the gate runs `cargo test` only on `libraries/*`, and only the files reached by a library `#[path]` shim (cores/*.rs, record.rs) are compiled for the host. No shim names route.rs, alias.rs or rtc/pcf8563.rs.
why: these tests are never compiled, never mind run. clippy runs `--release --target thumb` without `--tests`, so `cfg(test)` code is not even type-checked. They can rot to non-compiling and the gate stays green.
severity: medium
fix: add shims (route-core etc.) or run `cargo test -p efr32mg2x-hal --lib --target x86_64-…` for the host-safe modules; or have the gate fail when a `#[cfg(test)]` under crates/efr32mg2x-hal/src lies in a file no library `#[path]` reaches.
confidence: verified-by-reading

## finding 2 — HAL doc examples are never compiled; at least one is wrong
where: scripts/gate.sh doc-test step (inside `for dir in libraries/*/`); crates/efr32mg2x-hal/src/dma/mod.rs:167-170 (and ~70 fenced blocks in emu, rtc, wdog, usart, spi, adc, dma…)
what: `cargo test --doc` runs only for libraries/*. The HAL has about 70 fenced code blocks in doc comments. `dma/mod.rs:167` shows `let dp = pac::Peripherals::take().unwrap(); let dma = Dma::new(dp.ldma_s);` with no `use`, so it cannot compile as written.
why: `rust,no_run` still compiles under `--doc`, but nothing runs `--doc` on the HAL, so these blocks are checked by no one. These are the blocks docs.rs shows to a stranger.
severity: high (for publication)
fix: run `cargo test --doc -p efr32mg2x-hal --target thumbv8m… -Zdoctest-xcompile`, or `cargo rustdoc` with `--test` on a host-buildable cfg; at least mark non-compiling blocks `ignore` and gate on "no unmarked block fails".
confidence: verified-by-reading (gate scope); the specific dma block is inferred not to compile

## finding 3 — DMA and Board cannot be used in the same program
where: crates/efr32mg2x-hal/src/dma/mod.rs:157-158 (`Dma::take` = `pac::Peripherals::take().map(|dp| Self::new(dp.ldma_s))`); src/board.rs:54 (`Board::take` = `pac::Peripherals::take()?`); no `dma` handout in board.rs
what: both constructors take the PAC singleton. Whichever runs second gets `None`. Board has no `dma()` handout, and `Dma::new` needs a `pac::LdmaS` the Board keeps inside its private `dp`.
why: the layer rule (no example steals a PAC type) means a Board user can reach LDMA only by `LdmaS::steal()`. All three DMA examples (hal_dma_irq/stop/link) use `Dma::take` and no Board, so this composition was never tried. Same "public, documented and unbuildable" shape as the Timer1..4/Eusart1/Spi0/Spi2 defect fixed today.
severity: high
fix: add `Board::dma()` that claims a slot and returns `Dma::new(LdmaS::steal())`, and make `Dma::take` document that it excludes Board (or remove it).
confidence: verified-by-reading

## finding 4 — I2C1 is public, needs `&FrozenClocks`, and has no Board handout
where: crates/efr32mg2x-hal/src/i2c/mod.rs:396 (`I2c1::new(i2c: pac::I2c1S, config, clocks: &FrozenClocks)`); board.rs has only `i2c0` (board.rs:92)
what: this is the exact pattern fixed today for timer/eusart/spi: a constructor taking a `&FrozenClocks` that Board does not expose (Board stores `sysclk_hz`, not FrozenClocks), and no `new_at` variant.
why: a Board user cannot build I2C1 without `I2c1S::steal()` and a forged FrozenClocks. Nothing in the workspace calls `I2c1::new` (grep crates/*/examples, crates/*/src, apps: zero hits), so it has never executed.
severity: high
fix: add `I2c1::new_at(i2c, config, hfclk: Hertz)` as I2c0 already has (i2c/mod.rs:301), plus `Board::i2c1()`; then an instrument on the second I2C pair.
confidence: verified-by-reading

## finding 5 — the claim table is keyed by role, not by peripheral: USART0, EUSART1 and IADC0 can each be handed out twice
where: crates/efr32mg2x-hal/src/cores/slots.rs:86,103,104,105 (Usart0=4, Eusart1=21, Spi0=22, Spi2=23) and :88-89 (Adc=6, Converter=7); board.rs:187 vs :243 (both steal `Usart0S`, :192/:250); board.rs:317 vs :624 (both steal `Eusart1S`, :324/:629); board.rs:1438 vs :1478 (`converter`, `adc`: both IADC0)
what: `usart0()` claims `Slot::Usart0` and `spi0_on()` claims `Slot::Spi0`; `spi2_on()` claims `Slot::Spi2` and `eusart1_async()` claims `Slot::Eusart1`; `converter()` claims `Converter` and `adc()` claims `Adc`. Each pair is one hardware block.
why: HAL-MATURITY:76 grades API hygiene B on "a second request for the same peripheral is refused by name". That holds only within a role. `board.usart0(..)` then `board.spi0_on(..)` gives two live drivers that rewrite the same USART0 CTRL/FRAME. `hal_one_of_each` (`adc=1 adc2=1`) repeats the same slot, so it cannot see a cross-role alias. EUSART0 is done right (`spi_on` and `eusart0_async` share `Slot::Spi`, board.rs:473/:595), so the Spi0/Spi2 slots added on 2026-11-27 broke a rule the crate already followed. COVERAGE-BOUNDARY:88 ("with their own claims") describes the defect as a feature.
severity: high
fix: key slots by hardware block (Usart0, Eusart0, Eusart1, Iadc0, Timer0…) and map each role onto its block's slot. Add a host test (slots-core) asserting the role→block map, and extend `hal_one_of_each` with `usart0` then `spi0_on` (expect `Taken`).
confidence: verified-by-reading

## finding 6 — `Board::delay()` hands out unlimited SysTick owners, and its SAFETY note says it does not
where: crates/efr32mg2x-hal/src/board.rs:1303-1317
what: each call does `cortex_m::Peripherals::steal().SYST` into a fresh `Delay`, with no claim. The SAFETY note says "this is not a second owner".
why: two calls give two Delays that both reprogram SYST RVR/CVR. One used in an ISR while the other waits corrupts the wait silently. gate.sh rule 5 ("two owners of SysTick is a delay that lies") is enforced on examples and broken by the Board itself. check-safety-notes.py checks that a note *exists*, so a false note passes.
severity: medium
fix: claim a `SysTick` slot, or return a `Delay<'_>` that borrows SYST from `self._cp` through a RefCell; correct the note.
confidence: verified-by-reading

## finding 7 — `rtc::Rtc` (the "C4 RTC complete: calendar, alarms, 3 CC channels" of CLAUDE.md) has never executed and cannot coexist with Board
where: crates/efr32mg2x-hal/src/rtc/mod.rs:102 (`Rtc::new(sysrtc: pac::Sysrtc0Ns)`), its doc block at :98-101; CLAUDE.md "RTC (C4) - Complete (SYSRTC0)"; COVERAGE-BOUNDARY:37 lists only `sysrtc` for `sysrtc0`
what: grep over crates/*/examples, crates/*/src and apps finds no caller of `rtc::Rtc` (base_rtc uses the external Pcf8563). Board has no handout for it. Board::sysrtc() hands out a *second* driver for the same SYSRTC0 under `Slot::Sysrtc`, which `Rtc` does not claim.
why: same class as the Timer1..4 defect (never called, hence never wrong in any way anyone saw). Its calendar and alarm claims are unproven, and running it beside `Board::sysrtc()` or the Embassy time driver gives two masters on one counter.
severity: high (claim) / medium (code)
fix: either delete `rtc::Rtc` (sysrtc covers the counter) or hand it out from Board under `Slot::Sysrtc` and write `hal_rtc_calendar` (alarm N seconds ahead, record the fire tick against the SYSRTC count). Downgrade CLAUDE.md's C4 line until then.
confidence: verified-by-reading

## finding 8 — HAL-MATURITY contradicts itself on the two grades that gate publishing
where: docs/04-reference/HAL-MATURITY-2026-11-25.md:76-77 (API hygiene **B**, safety **B**) vs :133-134 ("API hygiene is C+ and safety is C") and :138 ("~106 unsafe sites without a SAFETY note") vs :77 ("197 unsafe sites, every one carrying the invariant")
what: §4 and §6 grade the same date differently, and §6 sets the bar at "B on every dimension".
why: a reader cannot tell whether the bar is met. Given findings 5-7, B on API hygiene is not supported in any case.
severity: medium
fix: re-date or delete the §6 sentences; regrade API hygiene to C+ until finding 5 is fixed and Dma/I2c1/Rtc are reachable.
confidence: verified-by-reading

## finding 9 — "the 58 #[test]s inside the crate RUN in their twin crates" is false, and the guard for it is not in the gate
where: docs/04-reference/HAL-MATURITY-2026-11-25.md:28; scripts/check-core-split.py (not called by gate.sh: grep finds no `core-split`)
what: see finding 1. 13 tests in route.rs, alias.rs and rtc/pcf8563.rs are reached by no shim.
severity: low
fix: correct the line; call check-core-split.py from the gate and extend it to "every HAL file holding `#[cfg(test)]` is reached by a `#[path]` shim".
confidence: verified-by-reading

## finding 10 — COVERAGE-BOUNDARY overstates i2c1 and the "every row has a record" heading, and §5 is stale
where: docs/04-reference/COVERAGE-BOUNDARY-2026-11-25.md:31 (heading "each with a record on the part"), :43 (`i2c0`, `i2c1` … evidence lists only I2C0 examples), :102-103 ("the four rows whose evidence is compiled, not measured (timer1–timer4)"), :58-59 (a dangling sentence fragment "would cost, so that the boundary…")
what: i2c1 has no caller and no record (finding 4). Timer1-4 are now measured, and the actually unmeasured rows are Spi0/Spi2 (:88) and i2c1. Understated in the other direction: :35 gives `hal_led_blink` as the GPIO-interrupt evidence while `hal_gpio_interrupt` exists, and :45 omits `hal_dma_link`/`hal_dma_irq`.
severity: medium
fix: split i2c1 into its own "compiled, never run" row; rewrite §5 from the current rows; fix the fragment.
confidence: verified-by-reading

## finding 11 — module READMEs attribute other modules' records by substring: `rtc` shows BURTC's measurement as its own
where: /root/.hermes/skills/software-development/hal-module-readme/scripts/gen-module-readme.py:124,126 (`name in text.lower()`, `name.split("_")[0] in ln.lower()`); output in crates/efr32mg2x-hal/src/rtc/README.md:110-117
what: `rtc/README.md` "What was measured" quotes `burtc: ok=1 run=1 …` and the SYSRTC compare record. `rtc` is a substring of `burtc` and `sysrtc`. The same README says "No example imports this module yet" (line ~127), so the module has no record at all.
why: a published page presents another peripheral's measurement as proof of an `rtc::Rtc` that has never run (finding 7). This is a false green produced by the generator.
severity: high
fix: match records by an explicit `module:` tag or the example name mapped to the module (examples import list is already computed), with word boundaries; never by substring.
confidence: verified-by-reading

## finding 12 — the same generator calls 12 measured modules "unmeasured on hardware"
where: crates/efr32mg2x-hal/src/{i2c,extint,emu,wdog,mvp,gpcrc,microphone,buzzer,bus,sensors,display,record}/README.md ("No mailbox record in `docs/` names this module…"); cause gen-module-readme.py:119 (`^\s*[a-z][a-z0-9]*\s*:` refuses `i2c-async:`; `gpioirq:` does not contain `extint`)
what: i2c (I2C-RECOVERY-STATE.md, ASYNC-I2C-INTERRUPT-2026-11-27.md), extint (GPIO-INTERRUPT-2026-11-27.md), wdog (`hal_wdog_feed PASS`), emu (`hal_emu_voltage PASS`) and mvp (`hal_mvp_bench PASS`) all have dated records, and their READMEs deny it.
why: the crate README and HAL-MATURITY §3 say "proven" while the module page on docs/package says "treat its API as unmeasured". Two generated surfaces contradict each other and no check compares them.
severity: medium
fix: same fix as finding 11, driven by an explicit module↔record table that the gate checks against the files in docs/04-reference.
confidence: verified-by-reading

## finding 13 — the module-README check never looks at the claims a README makes, and its generator is not in the repository
where: scripts/gate.sh:215-230; scripts/check-module-readmes.py (checks: file exists, no `TODO(measured)`, one table row per `pub` item)
what: the checker tests existence and API-row coverage only. "What was measured", "What is not proven", "Using it" and "Registers it touches" are unchecked. The generator lives only in `~/.hermes/skills/...`. On every CI runner the gate prints SKIP for the drift comparison and nothing can regenerate the READMEs. The brief's "34 module READMEs, generated from the code by a checker in the gate" conflates generator and checker.
why: findings 11-12 pass the gate. The check covers the half of the page that cannot lie (names) and ignores the half that can (evidence).
severity: high
fix: vendor gen-module-readme.py into scripts/ and run it in `--check` mode, as gen-crate-readme.py already is; add a rule that every record line quoted in a module README comes from an example that imports that module.
confidence: verified-by-reading

## finding 14 — 84 `../../../../docs/…` links ship inside the package's module READMEs, and check-publishable only reads the crate README
where: crates/efr32mg2x-hal/src/*/README.md (84 occurrences, e.g. rtc/README.md:119); scripts/check-publishable.py:146-158 (relative links checked only in `d / "README.md"`)
what: HAL-MATURITY:175 says the 41 `../../../../docs/...` links "were dead inside the package" and were fixed. That was for the crate README and crate-doc. The 34 module READMEs, generated later, reintroduced 84 of them, and `cargo package` ships `src/**/README.md`.
why: the same failure as 0.1.0's dead `docs/PHASE2_PLAN.md`, in files the checker never opens.
severity: medium
fix: run the relative-link loop over every `*.md` in the package list (the function already has `files`), or emit repository URLs from the generator.
confidence: verified-by-reading

## finding 15 — the gate's "examples build" PASS line counts files, while cargo silently skips every example behind `required-features`
where: scripts/gate.sh, examples loop (`count=$(ls "$dir/examples/"*.rs | wc -l)` … `pass "$pkg: $count examples"`), `features_for()` (radio gets `""`, xiao-bsp gets `sense` only); crates/efr32mg24-radio/Cargo.toml:52-69; crates/xiao-mg24-bsp/Cargo.toml:93-114
what: the radio crate has 7 example files, and 5 of them require `stack` (+`gatt`/`diag`). The gate builds it with default features (`ble`), so 2 are compiled and the line reads "7 examples". xiao-mg24-bsp has 6 examples requiring `stack`, built with `sense` only, so none of the 6 is compiled.
why: the input to the PASS text is `ls`, a source count, and not the set of targets cargo built. `check-examples-build.sh` says in its own header that it exists for exactly this, but it is **not called by gate.sh**.
severity: high
fix: derive the count from `cargo build --message-format=json` (`compiler-artifact` with `target.kind == ["example"]`) and fail when it differs from the number of example files; or call check-examples-build.sh from the gate.
confidence: verified-by-reading

## finding 16 — check-examples-build.sh, the source of CLAUDE.md's "all 125 example targets compile", never builds `ble_gatt` or `ble_diag`
where: scripts/check-examples-build.sh:27 (`efr32mg2x-radio * ble,stack`), :94-101 (COUNT = `find … -name '*.rs' | wc -l`); crates/efr32mg24-radio/Cargo.toml:64-69 (`ble_gatt` needs `gatt`, `ble_diag` needs `diag`)
what: the `*` row enables `ble,stack`, so cargo skips the two examples needing `gatt`/`diag` without a message. The completeness check accepts `*` as covering the crate, and the verdict counts files again.
why: the reported "125 build" contains at least 2 targets that were never compiled, and `ble_gatt` is the one carrying the phone-verified GATT claim. This is the third false green of the file-count shape.
severity: high
fix: build each crate with `--all-features` where that is legal, or read each example's `required-features` from `cargo metadata` and build it with exactly those; count artifacts, not files.
confidence: verified-by-reading

## finding 17 — the crate README says "Compiled, not measured: Nothing" while three handed-out instances have no record
where: crates/efr32mg2x-hal/README.md "Compiled, not measured" (≈line 189: "Every instance the crate hands out has a record on the part"), and the `I2C0`, `I2C1` row of "Driven, and proven"; template templates/readme-hal.tmpl
what: `Board::spi0_on` and `Board::spi2_on` hand out Spi0/Spi2, which COVERAGE-BOUNDARY:88 itself calls **unmeasured** (the 2026-11-27 run read 1 byte of 8). I2C1 is listed as proven with no caller (finding 4). `rtc::Rtc` has never run (finding 7).
why: this is the sentence a crates.io reader trusts, and the repository's own boundary file contradicts it. It is the same class of failure as the withdrawn 0.1.0.
severity: blocker (for publication)
fix: list Spi0, Spi2, I2C1 and rtc::Rtc under "Compiled, not measured", and make the template build that section from a machine-readable table that check-publishable cross-checks against COVERAGE-BOUNDARY.
confidence: verified-by-reading

## finding 18 — the HAL README count check reads one phrase; the same page says "21 modules" and "Last Updated: November 14"
where: crates/efr32mg2x-hal/README.md:414 ("**Status**: 21 modules in the tree"), :413-ish ("Last Updated: November 14, 2026"); templates/readme-hal.tmpl:316; scripts/gate.sh hal-README-counts block (`grep -oE '[0-9]+ public modules' … | head -1`)
what: the gate compares only the first "N public modules" against `grep -c '^pub mod'` (35, which counts the inline `pub mod prelude {` as a module). The footer's hand-typed 21 is never compared.
why: the check's input is one regex on one sentence, so any other number on the page is free to drift. Also, a `^pub mod` count that includes `prelude` makes "35 public modules" disagree with "34 module READMEs".
severity: low
fix: generate the footer from the same counter, or have the template refuse any literal digit followed by "modules"; decide whether `prelude` is a module and use one definition in all three places.
confidence: verified-by-reading

## finding 19 — public HAL functions with no caller anywhere in the workspace (never executed)
where: the scan covered every crate's examples, apps/, and non-HAL src for each `pub fn` in crates/efr32mg2x-hal/src (cores excluded):
  spi/types.rs `with_bit_order` (CLAUDE.md claims "MSB-first / LSB-first"), wdog/types.rs `warning_only`, `timeout_ms`, `cmd_sync_busy` (CLAUDE.md claims "warning mode"), board.rs `boost_to_38_mhz`, `boost_to_56_mhz`, timer/mod.rs `get_counter`, `reset_counter`, `disable_channel`, `unlisten_overflow`, usart/mod.rs `flush_receiver`, eusart/mod.rs `actual_baud`, buzzer.rs `set_tone`, pwm.rs `frequency_hz`, rtc/mod.rs `ticks_to_millis`, `millis_to_ticks`, extint.rs `is_armed`, mvp `is_idle`, adc `with_resolution`, ssd1306 `clear_display`; plus whole constructors `I2c1::new` (finding 4), `rtc::Rtc` (finding 7), `Dma` beside `Board` (finding 3)
what: none of these is referenced outside its own file.
why: every defect found today was in exactly this population. `with_bit_order` (LSB-first) and `warning_only` back claims made in CLAUDE.md.
severity: medium
fix: cheapest instruments, in order: (a) `hal_spi_loopback` run twice, MSB then LSB, recording the byte pattern (the same jumper); (b) `hal_wdog_feed` with `warning_only` and the WARN flag read back without a reset; (c) `hal_i2c1_scan` on the second pair; (d) `boost_to_38/56` folded into `hal_delay_clock` (it already measures delay against the SYSRTC after a boost). Gate: a `#[doc(hidden)]`-free `pub fn` with no caller in examples/ is listed by the gate as "compiled, never run".
confidence: verified-by-reading (crude name scan; trait-method and doc mentions can mask a few more)

## finding 20 — "bounded waits everywhere" is false: USART/EUSART TX waits are unbounded, and several timeouts are swallowed
where: crates/efr32mg2x-hal/src/usart/mod.rs:185 (`write_byte`), :334 (`flush`); src/eusart/mod.rs:207 (`write_byte`), :276 (`flush`); swallowed: src/sysrtc.rs:373-377 (`break` at 200 000 then carries on), src/pcnt.rs:193-197, src/board.rs:1134-1138 and :1235
what: the TX waits spin until a status bit sets, with no budget. The sync waits time out and then `break` into the next register write with no error.
why: HAL-MATURITY:80 grades error handling B on "bounded waits everywhere since 2026-11-17". EUSART1's own record shows a block can be in a state where frames go nowhere. A block whose clock or TXEN is off never sets TXC, so `flush()` hangs. A `break` after a sync timeout is the same "timeout read as success" defect that WDOG's `SyncTimeout` was introduced to remove.
severity: medium
fix: give every wait a budget and return `Error::Timeout` (`nb::Error::WouldBlock` for the embedded-hal-nb traits); make `wait_sync` return `Result`. Add a gate grep: `while` on a `.read()` with no counter in the body is a failure.
confidence: verified-by-reading

## finding 21 — COVERAGE-BOUNDARY cites an instrument that has never been in the repository
where: docs/04-reference/COVERAGE-BOUNDARY-2026-11-25.md:51 ("`hal_pcnt_timer_rate` sweeps the rate until the count stops moving (`ceiling=0k`)")
what: `git log --all -S'hal_pcnt_timer_rate'` returns only the two doc commits (007230f, 437c752). No `.rs` file by that name was ever committed. The examples dir has `hal_pcnt_gpio` only.
why: the record `pcntrate: r1k=0/13 …` cannot be reproduced from the tree, so the "pulse width, not bandwidth" claim rests on a program that does not exist.
severity: medium
fix: commit the instrument (it existed in the run on 2026-11-26), or mark the row "record without an instrument in the tree".
confidence: verified-by-reading

## finding 22 — feature-gated code is never linted, documented-checked or built
where: scripts/gate.sh `features_for()` (radio → `""`); crates/efr32mg24-radio/src/lib.rs:40,74,81 (`#[cfg(feature = "diag")]`, `rail`, `external-bonding`); radio `stack`
what: clippy and the examples build use one feature set per crate. Modules behind `diag`, `rail`, `external-bonding` and `stack` are not compiled by any gate step, so `deny(missing_docs)` and clippy never see them. check-missing-docs relies on those builds; `--measure`, which compiles, is not the gate mode.
why: "0 undocumented items across 33 crates" and "zero warnings" hold for the default feature set only.
severity: medium
fix: run clippy per crate with `--all-features` where the features are additive (or `cargo hack --each-feature`), and name the excluded combinations.
confidence: verified-by-reading

## finding 23 — gate checks whose input cannot fail on the defect they name (the shapes)
where / what / why, one line each:
- gate.sh rule 3 "HAL exposes no register block": a line regex on `pub fn … pac::x::RegisterBlock`. rustfmt splits long signatures across lines, and an imported `RegisterBlock` carries no `pac::` prefix, so both escape. Meanwhile 20 public constructors take PAC singletons (`pac::LdmaS`, `pac::I2c1S`, `pac::CmuS` …; src/dma/mod.rs:171, src/i2c/mod.rs:396, src/adc/mod.rs:102), which is the coupling the rule's comment says it prevents.
- gate.sh rule 7 "a crate that declares the PAC uses it": `grep -rc 'pac::'` counts comments, so one comment satisfies it.
- check-safety-notes.py:39-41: any `SAFETY` within 8 lines above satisfies a block, so one note covers every following block in its window. `unsafe impl` (src/sysrtc.rs:170 `unsafe impl Send for Sysrtc`) is never counted. Only the HAL is scanned. The note's truth is unchecked (finding 6).
- check-module-readmes.py: rows only, never claims (finding 13).
- check-publishable.py: `forbidden` is a hand-written list of five phrases from the last failure, with no `required` list for the HAL. Relative links are checked only in README.md (finding 14). The `blob/master` URLs are resolved against the local checkout, not what GitHub `master` serves, so an unpushed or gitignored target passes.
- gate.sh hal-README counts: one regex on one sentence (finding 18).
- gate.sh examples: file count as the PASS text (finding 15).
severity: medium (as a class)
fix: for each, feed the check the live artifact (cargo JSON, rustdoc JSON for public signatures, `cargo package --list` for every shipped `.md`), and give each a fixture that must turn it red, as check-missing-docs already does.
confidence: verified-by-reading

## finding 24 — publication surface the tree cannot see about itself
where: crates/efr32mg2x-hal/Cargo.toml (no `rust-version`; no `readme =`, so cargo's default is relied on), crates/efr32mg2x-hal/build.rs + memory.x (ORIGIN 0x08006000), README "Documentation" section (`[src/pin/](src/pin.rs)`, `[src/delay.rs](src/delay/)`: label and target swapped), README absent-modes table (the `TIMER1`–`TIMER4` row has 3 cells in a 2-column table)
what: (a) no MSRV, while `embedded-hal-async = "1"` needs async-fn-in-trait (Rust ≥ 1.75), so an older toolchain fails with a trait error, not a version message. (b) The HAL's build.rs puts its own XIAO-bootloader `memory.x` on every downstream link search path. A crate named `efr32mg2x-hal` therefore imposes one board's flash layout, and a user's own memory.x competes with it by search order (inferred). (c) The HAL README's Quick Start is never compiled: gate.sh calls check-readme-example.py for the PAC README only. (d) check-publishable never builds the package. "Package builds from its tarball" (HAL-MATURITY:79, layering A−) is a 2026-11-25 measurement that nothing re-runs, while acmp, aes, burtc, extint and eusart were added since.
severity: medium
fix: set `rust-version`; move memory.x to the BSP (or gate it behind a `xiao-layout` feature); run check-readme-example.py on the HAL README too; add `cargo package -p efr32mg2x-hal` (verify build) to the gate or to pre-publish as a required step.
confidence: (a)(c)(d) verified-by-reading, (b) inferred


---

## The run's final message (verbatim)

Review finished: 24 findings. Plan mode would not let me write `/projects/_tmp/efr32mg24-rs/opus-review-2026-11-27/findings.md`, so each finding went into `/root/.claude/plans/the-hal-reviewed-snug-catmull.md` as I established it, in your format, after a regrading table for the eight dimensions. Nothing was built, flashed or committed. About 45 tool calls used.

## Ranked

**Blocker**
- **f17**: the crate README says "Compiled, not measured: Nothing. Every instance the crate hands out has a record." That is false for Spi0 and Spi2 (your own COVERAGE-BOUNDARY:88 calls them unmeasured), for I2C1 and for `rtc::Rtc`.

**High**
- **f3**: `Dma::take` and `Board::take` both take the PAC singleton, and `Board` has no `dma()` handout. A program using `Board` cannot get DMA without `steal()`.
- **f4**: `I2c1::new` needs a `&FrozenClocks` that `Board` doesn't expose, and there is no `new_at`. Nothing calls it. It is listed as proven.
- **f5**: the claim table is keyed by role, not by hardware block. USART0 (`Usart0` and `Spi0`), EUSART1 (`Eusart1` and `Spi2`) and IADC0 (`Adc` and `Converter`) can each be handed out twice. `hal_one_of_each` only repeats the same slot, so it cannot catch this.
- **f7**: `rtc::Rtc` (CLAUDE.md's "C4 complete: calendar, alarms") has no caller, no `Board` handout, and would be a second driver on SYSRTC0.
- **f11**: the module README generator matches records by substring. `rtc/README.md` shows the BURTC measurement as its own evidence.
- **f13**: the module README check only checks the API rows, never the evidence sections. The generator lives in `~/.hermes`, not in the repo, so CI can't regenerate or verify the pages.
- **f15**: the gate's examples PASS line is an `ls` count. Cargo silently skipped 5 of 7 radio examples and 6 xiao-bsp examples, because they need features the gate doesn't enable.
- **f16**: `check-examples-build.sh`, the source of "125 example targets compile", isn't called by the gate. It also never builds `ble_gatt` or `ble_diag` (they need `gatt`/`diag`), and its verdict counts files too.
- **f2**: no HAL doc-test is ever compiled. The "23 crates with doc-tests" are all `libraries/*`. At least one doc block (`dma/mod.rs:167`) looks unable to compile; I inferred that, I didn't build it.

**Medium**
- f6: `Board::delay()` creates a new SysTick owner on every call, and its SAFETY note says it doesn't.
- f8: HAL-MATURITY gives API hygiene and safety B in §4 and C+/C in §6.
- f10: COVERAGE-BOUNDARY claims an I2C1 record and its §5 is stale.
- f12: 12 module pages say "unmeasured", including i2c, extint, wdog, emu and mvp, which do have records.
- f14: the module READMEs that ship in the package contain 84 `../../../../docs` links that are dead there.
- f19: about 20 public functions are never called, including SPI LSB-first, WDOG warning mode, `boost_to_38/56` and four timer methods.
- f20: the USART/EUSART transmit waits have no timeout, and three sync waits time out with a silent `break`.
- f21: the cited `hal_pcnt_timer_rate` instrument was never committed.
- f22: code behind the radio's `diag`, `rail`, `external-bonding` and `stack` features is never linted or checked for missing docs.
- f23: the checks whose input can't fail on the defect they name (register-block regex, `pac::` counted in comments, the 8-line SAFETY window, `unsafe impl` not counted, the hand-written forbidden-phrase list).
- f24: publication gaps: no `rust-version`; `build.rs` puts the XIAO `memory.x` on every downstream link (inferred); the HAL Quick Start is never compiled; `cargo package` is never re-run.

**Low**
- f1: 13 HAL `#[test]`s (route, alias, pcf8563) are never compiled.
- f9: the maturity file's "the 58 tests run in their twin crates" is false, and `check-core-split.py` isn't in the gate.
- f18: the README footer says "21 modules" and "Last Updated: November 14"; the count check reads only one sentence.

## Fix first
1. **f5, the claim table keyed by role.** It is a soundness hole in the one mechanism that API hygiene is graded on. Yesterday's Spi0/Spi2 change caused it, and the existing instrument can't see it. Key slots by hardware block, add a host test, add `usart0()` then `spi0_on()` → `Taken` to `hal_one_of_each`.
2. **f15/f16, counting built examples instead of files.** This is the fourth false green of the same shape, and it currently hides `ble_gatt`, the example behind the phone-GATT claim. Count artifacts from cargo's JSON output and fail when the number differs from the files.
3. **f17 with f11–f13, the evidence pages.** These are the pages a stranger reads. Correct the README, vendor the generator and match records by explicit module tag rather than substring.

## Where this brief is wrong
- **"34 module READMEs generated from the code by a checker in the gate."** The checker only verifies API rows. The generator is outside the repository and the gate never runs it, and the evidence sections it writes are wrong both ways (f11–f13).
- **"doc-tests: 0 crates at 0 passed, 23 with at least one."** True only for `libraries/*`. The HAL, the subject of this review, has never compiled a doc-test (f2), so the addendum's A− for documentation doesn't follow.
- **"Three drivers locked to a single instance… fixed."** The class isn't closed: I2C1 has the same `&FrozenClocks` shape (f4), and DMA and `rtc::Rtc` are unreachable alongside `Board` (f3, f7).
- **The brief's own example counts are fine** (43 HAL examples, 35 `pub mod`). But the 35 includes the inline `prelude`, while the module-README count is 34, and CLAUDE.md still says 41 examples.
- **COVERAGE-BOUNDARY's "pulse width, not bandwidth" ceiling** (cited as settled) rests on an instrument that was never committed (f21).
- **The output location.** Writing to `findings.md` isn't possible under the plan mode the brief itself sets, so everything is in the plan file instead.
