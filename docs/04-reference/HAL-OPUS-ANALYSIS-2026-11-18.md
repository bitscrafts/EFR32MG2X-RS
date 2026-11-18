# HAL: how to publish without publishing the libraries, and without losing host testability

**Provenance.** An independent analysis, dispatched 2026-11-18 with a brief at commit `3bb82bd`,
run by a frontier model in read-only plan mode with the whole tree available. It answered one
question — *what structure lets the HAL be published while the `libraries/` stay unpublished and
everything stays testable on the host?* — and was asked explicitly to say where the brief was wrong.

**Why it is in the repository.** It was written to `_tmp/`, which is not in git, which is the same
mistake as the review report rescued to `04-reference` on the morning of 2026-11-18. An analysis
that lives in a scratch directory is an analysis nobody will read twice.

**What came of it.** Its recommendation — one copy, source in the HAL, library as a `#[path]`
harness — was implemented as `C1` in commit `efedb81`. Its finding that the HAL's "cannot build on
the host" premise was unproven turned out to be half right: it compiles, but its tests cannot link
because of inline `dsb`/`isb`. Its finding that the gate's "no hardware dependency" check checked
nothing was confirmed, and fixed.

---

# HAL publication without publishing `libraries/` — findings (read-only, commit 3bb82bd)

Note: plan mode allows writing only this file, so findings are appended here instead of `_tmp/opus-hal/findings.md`.
Copy verbatim once plan mode is lifted.

## finding F1 — the "no hardware dependency" gate check passes vacuously
where: `scripts/gate.sh:340-341` (vs `:330`, `:152-157`)
what: `cargo tree -p "$name"` uses the *directory* name (`adc-core`), not the package name (`efr32mg2x-adc-core`) that
`pkg_name` returns and that the test line uses at `:332`. `cargo tree -p adc-core` errors, stderr goes to `/dev/null`,
grep sees nothing, and the branch prints `pass "...: no hardware dependency"`. Independently, the pattern
`^(efr32mg24|motion-detection|cortex-m)` cannot match `efr32mg2x-pac` or `efr32mg2x-hal` — the two crates that matter.
why: AGENTS.md HARD RULE 0b; this is the check that protects the whole premise "libraries test on the host".
Only the `cargo test --target $HOST` line (`:332`) remains, and it would not catch a PAC dependency if the PAC
compiles on the host (F8). So today the invariant is unproven.
severity: high (it is the guard of the structure being redesigned).
fix: `cargo tree -p "$pkg" -e normal,build --prefix none` with `set -o pipefail`-style failure on a cargo error;
pattern `^(efr32mg2x-(pac|hal)|efr32mg24-|cortex-m|xiao-)`; near-miss to keep on the normal path: a library whose
*name* starts `efr32mg2x-` (all of them) must still pass — anchor on the dependency lines, skip line 1 (the root).
confidence: high for the name mismatch (`:330` vs `:340` read); medium that `cargo tree -p adc-core` errors rather than
fuzzy-matching (cargo's pkgid spec is exact on name) — measure first: `cargo tree -p adc-core; echo $?`.

## finding F2 — five of the eleven `-core` crates have no caller; their tests measure code the device never runs
where: `crates/efr32mg2x-hal/Cargo.toml:13-18` (6 cores) vs `Cargo.toml:16,17,18,22,24` (clock, emu, gpio, timer,
wdog); grep of `crates/**/*.rs` finds no `clock_core|emu_core|gpio_core|timer_core|wdog_core`.
what: `libraries/README.md:27-28` says "the HAL ... calls the core rather than reimplementing it". For five cores the
HAL reimplements: `pin.rs:141-148` does its own DOUT read-or-write (H17 still live), `clock/clocks.rs:28,32`,
`pwm.rs:121`, `board.rs:284` keep `19_000_000` (H21/H22 still live) while `clock-core` exists.
why: HARD RULE 0b — the gate prints `clock-core: host tests (N passed)` while the live clock path is the unfixed one.
severity: high for the publication question: 5 of the 11 cores are not part of the HAL at all, so "vendor the
libraries into the HAL" is about 6 crates, not 16.
fix: wire before you move. Each core either gets a caller in the HAL (one commit per core, with the defect it closes)
or is marked "not yet wired" in `libraries/README.md`. Moving unwired code into the HAL changes nothing.
confidence: high.

## finding F3 — `gpio-core::with_dout` cannot close H17 even when wired
where: `libraries/gpio-core/src/lib.rs:1-11` (claims H17), `:147-154` (`with_dout`), `crates/efr32mg2x-hal/src/pin.rs:141-148`,
`crates/efr32mg2x-hal/src/alias.rs:3-6` (SET/CLR windows at +0x1000/+0x2000).
what: H17 is a race between the read and the write of `DOUT`. `with_dout` is the same read-modify-write as pure
arithmetic; its test "setting one pin never touches another" is true by construction and says nothing about the window.
The fix is a single write to `DOUT_SET`/`DOUT_CLR` (the alias the HAL already uses for `IF`), which needs no core.
why: a host test that passes while the defect it names is live (0b).
severity: medium. fix: HAL `set`/`toggle` via `+0x1000`/`+0x2000`/`+0x3000`; the core keeps only `route()`/`field_of()`.
confidence: high.

## finding F4 — the brief's count is wrong: 15 crates, not 16
where: `Cargo.toml:29-30` lists `libraries/bt-heap` twice; `ls libraries/` has 15 directories.
what: the "16 crates" is the member-list line count. Of the 15: 11 `-core` (6 used by the HAL), 4 domain crates
(`motion-detection` used by both BSPs, `aes-*` + `bt-heap` used by `efr32mg24-bluetooth-sys:18-24`).
severity: low (but the duplicate member line should go). confidence: high.

## finding F5 — library types are in the HAL's public API
where: `crates/efr32mg2x-hal/src/i2c/mod.rs:63` (`pub use efr32mg2x_i2c_core as proto;`),
`dma/mod.rs:249,360` (returns `efr32mg2x_dma_core::Transfer`), `dma/mod.rs:592` (implements the core's `Channel` trait).
what: any option that keeps the cores as separate *published-or-not* crates leaks their identity into the HAL's semver
surface; any option that moves them inside the HAL makes `proto` and `Transfer` HAL types. A user who names
`efr32mg2x_i2c_core::Error` today breaks under (b)/(c); nobody outside does yet (HAL not on crates.io), so the change is
free *now* and costly after the first HAL release.
severity: medium. confidence: high.
Addendum: `libraries/i2c-core/src/lib.rs:17` has `pub mod sim;` unconditionally, so the host simulator is reachable as
`efr32mg2x_hal::i2c::proto::sim` — a test double in the HAL's public API.

## finding F6 — the radio chain is blocked before the HAL question: `emlib-sys` is `publish = false`
where: `crates/efr32mg24-emlib-sys/Cargo.toml:7` (`publish = false`), normal dependency of
`efr32mg24-rail-sys/Cargo.toml:20` and `efr32mg24-bluetooth-sys/Cargo.toml:23`; `efr32mg24-radio/Cargo.toml:16,18`
depends on both; `xiao-mg24-bsp/Cargo.toml:41` and `xiao-expansion-base-bsp/Cargo.toml:33` depend on the radio
unconditionally (inside `[dependencies]`, not optional).
what: cargo refuses to publish a crate whose normal dependency is not on the index, optional or not. So neither BSP
can publish until emlib-sys -> cmsis-stubs -> rail-sys -> bluetooth-sys -> radio are on crates.io, and bluetooth-sys
itself has four path-only `libraries/` deps (`:18,19,20,24`) — the same problem as the HAL, again, plus vendor blobs.
severity: high for part 3 of the brief (the premise "if the HAL publishes, the BSPs follow" is false).
fix: see "Publication chain" below. confidence: high.

## finding F7 — `sysrtc-core` has two consumers; vendoring it into the HAL alone creates two copies
where: `crates/efr32mg2x-hal/src/sysrtc.rs:316`, `crates/efr32mg24-bluetooth-sys/src/sleeptimer.rs:18,69`,
`crates/efr32mg24-bluetooth-sys/Cargo.toml:20`.
what: BT-sys calls `sysrtc_core::compare_target` and names `WideTick` directly. If the HAL gets a private copy and BT
keeps the library, two implementations of the same wrap arithmetic run on one device and can drift.
fix: the file lives in the lowest published crate that calls it (the HAL) and is re-exported; BT-sys already depends
on the HAL (`:17`) and uses the re-export. confidence: high.

## finding F8 — "the HAL does not build for the host" is unproven; the blocker is one line, not the dependencies
where: the only raw ARM asm in the HAL is `crates/efr32mg2x-hal/src/sysrtc.rs:233` (`asm!("dsb 0xf","isb 0xf")`);
the other sites call `cortex_m::asm::*` (`delay/mod.rs:107`, `emu/mod.rs:136,211`, `pwm.rs:102`, `microphone.rs:222`,
`bus/bitbang.rs:98`), which cortex-m 0.7.10 gates with `#[asm_cfg(cortex_m)]` (`cortex-m-0.7.10/src/asm.rs:40`); its
single-core critical section is `#[cfg(all(cortex_m, ...))]` (`src/lib.rs:115`). `libraries/README.md:25` gives
"it contains ARM asm!" as the reason the cores exist.
what: the HAL may compile on x86_64 with `--no-default-features --features mg24` (no `rt`), plus a host
critical-section impl in dev-deps. Two HAL files already carry `#[cfg(test)]` modules that never run
(`alias.rs:35-44`, `rtc/pcf8563.rs`) — tests nobody executes.
measure first (M1): `cargo check -p efr32mg2x-hal --target x86_64-unknown-linux-gnu --no-default-features --features mg24`.
severity: medium (it changes which options are open). confidence: medium — I read cortex-m's gating, did not build.

## finding F9 — the publishability check is green for a HAL that cannot be packaged
where: `scripts/check-publishable.py:49-57` runs `cargo package --list`; `scripts/gate.sh:113-117` turns it into
"crates.io: the package says what the tree does". `INCLUDE` regex (`check-publishable.py:46`) matches only
`include_str!`, not `include!` or `#[path = ...]`.
what: `--list` does not resolve the publish requirement that every dependency has a version (the dry-run error in the
brief), so the gate passes for the HAL while `cargo publish` fails. And a `#[path]`/`include!` that reaches outside the
package — the exact mistake options (a)/(b) invite — is not seen.
severity: high (0b). fix: in check-publishable, parse `[dependencies]`/`[build-dependencies]` of each publishable
crate and fail any entry with `path` and no `version`; extend `INCLUDE` to `include!`, `include_bytes!`, `#[path = "…"]`
and fail anything resolving outside the crate directory.
confidence: medium on `--list` skipping the version check (M2: `cargo package -p efr32mg2x-hal --list --allow-dirty; echo $?`);
high on the regex.

## finding F10 — `pre-publish.sh` runs `cargo publish --dry-run`
where: `scripts/pre-publish.sh:35`; CLAUDE.md "HARD RULE — no publication" names a `--dry-run` presented as a step
towards publishing as forbidden without an express order.
what: the sanctioned verification path calls the forbidden command. severity: medium. fix: replace with
`cargo package -p "$NOME" --allow-dirty` (verifies and builds the packaged copy, no index write) plus F9's version
check; keep the dry-run only behind an explicit `--with-dry-run` flag the operator passes. confidence: high.

## finding F11 — the generated HAL README lists dependencies that are not on crates.io
where: `scripts/gen-crate-readme.py:224-226` — a path dep with no version falls back to `ws.get(name, "workspace")`.
what: the published README's Dependencies table would show `efr32mg2x-adc-core | workspace` etc. Disappears with the
recommended structure. severity: low. confidence: high.

---

# Part 1 — options (scope: the **6 wired cores**, ~1,650 lines; the 5 unwired cores and 4 domain crates are separate, F2/F4)

| option | what breaks | host-testable? | contributor must know | gate can prove | split back out later |
|---|---|---|---|---|---|
| (a) copy `libraries/*/src` into HAL `src/vendor/` (committed), drift check | two copies; people fix the wrong one; a generated file in `src/` | yes (library copy) | "never edit `src/vendor/`, run the generator" | copy == source (byte compare) | cheap, but drift is the steady-state risk |
| (b) inline into HAL, tests in HAL `tests/` with `include!` | needs the HAL lib to build on host (integration tests link it; M1); PAC compile in every test run; per-library `examples/` lost | only if M1 passes | HAL host-build flags | tests run | expensive: tests must be pulled back out |
| (c) fold 16→1 unpublished crate, vendored | same as (a) for one crate; mixes BT crypto/heap with HAL cores | yes | as (a) | as (a) | worst: re-splitting a merged crate |
| (d) **source lives in the HAL, harness crate points in with `#[path]`** | `#![no_std]` must leave the moved file (crate attr in a module); `mod.rs` layout needed for `i2c/sim.rs` | yes, unchanged: `cargo test -p efr32mg2x-i2c-core` | "the core's file lives in the lowest published crate that calls it; `libraries/x-core` is its test harness" | harness compiles the *same file* the HAL compiles (one copy, nothing to drift) | cheapest: `git mv` + one `#[path]` line either way |
| (e) publish the 6 cores (or one `efr32mg2x-hal-core`) | contradicts the "not published" rule; names are permanent | yes | semver of each core | — | cheap technically, irreversible on the index |

## Recommendation: (d), "source in the published crate, harness in `libraries/`"

```
crates/efr32mg2x-hal/src/core/i2c/mod.rs      <- the code + its #[cfg(test)] mod (git mv from libraries/i2c-core/src/lib.rs)
crates/efr32mg2x-hal/src/core/i2c/sim.rs      <- behind #[cfg(any(test, feature = "sim"))], not in the default API
crates/efr32mg2x-hal/src/core/mod.rs          <- pub mod i2c; pub mod dma; ...
libraries/i2c-core/src/lib.rs                 <- #![no_std]  #[path = "../../../crates/efr32mg2x-hal/src/core/i2c/mod.rs"] mod imp; pub use imp::*;
libraries/i2c-core/Cargo.toml                 <- publish = false; features = { sim = [] }, enabled for its examples
```

Why: one copy (no generator, nothing to verify against a generator), the host test command and per-library examples do
not change, the HAL needs no host build (F8 becomes optional), the HAL package contains every byte it compiles, and the
direction of reach (unpublished -> published) is the one cargo tolerates. It generalises to bluetooth-sys's four domain
crates when the radio chain is taken on (F6) and to the 5 unwired cores at the moment each is wired (F2): **wiring and
moving become one step.**

Flexibility (part 2): split a core back out = `git mv` the dir to `libraries/x/src/`, replace `pub mod x` with
`pub use efr32mg2x_x_core as x` + a versioned dep (only if it gets published). New library = new dir under
`src/core/` + three-line harness. Move logic from HAL into a core = new file under `src/core/`, gate block 4 enforces
purity. None of these needs a generator or a re-vendoring step.

# Part 3 — publication chain

1. `efr32mg2x-pac` 0.1.1 — on the index. HAL keeps `version = "0.1.0"` (`efr32mg2x-hal/Cargo.toml:19`, resolves 0.1.1;
   raise to `"0.1.1"` if the HAL relies on anything new in it).
2. `efr32mg2x-hal` — after (d) it has zero path-only normal deps. Before the *first* publish settle the public surface:
   `i2c::proto` (+ `sim`), `dma::Transfer`, and the **H21 `Clocks::new` shape** — fixing that after release is a
   0.2 break; H17/H22/H28 are bug fixes and can ship as 0.1.x.
3. `efr32mg2x-xiao-bsp`, `efr32mg2x-expansion-base-bsp` — **cannot follow the HAL** (F6). Two exits: (i) publish the
   radio chain in order `cmsis-stubs` -> `emlib-sys` (drop `publish = false`, blobs fetched at build time) ->
   `rail-sys` -> `bluetooth-sys` (its 4 libraries moved by (d)) -> `efr32mg2x-radio` -> `xiao-bsp` ->
   `expansion-base-bsp`; or (ii) take the radio out of the BSPs' `[dependencies]` into a separate `-ble` crate, so
   the BSPs need only HAL + PAC (+ `motion-detection` moved into the BSP by (d)). Every surviving path dep gets
   `version = "<what is on the index>"`; path-only **dev**-dependencies are stripped by cargo and are fine
   (`rail-sys:29`, `emlib-sys:16`). Each step needs its own express order.

# Part 4 — the verification: `scripts/check-core-split.py`, in the gate

Compares, for every `crates/*/src/core/<x>/`:
1. a harness `libraries/<x>-core/src/lib.rs` exists whose only non-test item is `#[path = "<that file>"]` + `pub use`
   — resolved, canonicalised paths equal;
2. the owning crate declares `pub mod <x>` in `src/core/mod.rs` **and** references `core::<x>::` from at least one
   file outside `src/core/` (a core compiled but uncalled fails — the F2 pattern);
3. purity outside `#[cfg(test)]` and outside comments: no `pac`, `cortex_m`, `unsafe`, `asm!`, `crate::`;
4. F1 fixed: `cargo tree -p "$pkg" -e normal,build` on the package name, cargo error = FAIL, pattern includes
   `efr32mg2x-(pac|hal)`;
5. F9 fixed: no path-only normal dep in a publishable crate; no `#[path]`/`include!` in it resolves outside its dir.

It must fail on fixtures: a harness pointing at a copy in `libraries/`; a HAL `#[path]` into `libraries/`; a core with
`unsafe` in code. **Near-misses that must take the normal path:** a harness with extra `#[cfg(test)]` tests and its own
`examples/`; `unsafe` or `pac::` inside a doc comment or a `#[cfg(test)]` block; a library whose own name starts
`efr32mg2x-`; a core used by two crates (sysrtc) referenced via re-export from bluetooth-sys.

# Part 5 — where the brief is wrong

- "16 crates" — 15; `bt-heap` is listed twice (F4).
- "the HAL does not build for the host: it pulls the PAC and cortex-m" — unproven; the dependencies are not the blocker
  (F8). Measure M1 first.
- "the gate runs each library's host tests" — yes, but its paired no-hardware check is vacuous (F1) and 5 of 11 cores
  test code the HAL does not call (F2, F3).
- Implicit: "if the HAL publishes, the BSPs can follow" — false, `emlib-sys` is `publish = false` (F6).
- Implicit: the gate's publishability line covers the HAL — it is green while the HAL cannot be packaged (F9, M2).

# Least-cost order

0. Measure M1 (host check of the HAL), M2 (`cargo package --list` exit code), M3 (`cargo tree -p adc-core; echo $?`).
1. Gate fixes F1 + F9 + F10 (the HAL goes honestly red as "not publishable: N path-only deps" — list it as a known,
   printed blocker so the verdict is not masked).
2. `check-core-split.py` with fixtures, before any move.
3. Move the 6 wired cores, one commit each, gate after each; sysrtc last (two consumers, F7).
4. HAL public surface + H21 API shape; regenerate README (`gen-crate-readme.py --check`, F11).
5. Wire-and-move the 5 remaining cores as their defects are fixed (H17 via the SET/CLR alias, F3).
6. HAL publish — only on an express order. Radio/BSP chain is a separate decision (F6).

Not done: no build, no flash, no tree changes; F8 and F9 carry one measurement each before they are certain.
