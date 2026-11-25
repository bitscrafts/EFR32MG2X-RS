# HAL maturity, and whether it can be published

**Assessment date: 2026-11-25.** Everything below was measured on the tree at `9857b26`, on the board, or read from the
registry — no item is repeated from a previous report without re-checking it. The operator's standing rule applies to
the end of this document too: **nothing is published without an express order, and never without this analysis being
current.**

## 1. Registry state, as of today

| crate | in the crates.io index | last version | published |
|---|---|---|---|
| `efr32mg2x-hal` | **no versions** (sparse index answers `NoSuchKey`; the API answers 403 to this container's client, which is bot protection, not existence) | — | withdrawn |
| `efr32mg2x-pac` | yes | `0.1.1`, `yanked: false` | 2026-11-18T18:38:46Z |

So the HAL is **not published**, and the repository says otherwise in two places: `CHANGELOG.md` ("Both crates
republished as 0.1.1") and `CLAUDE.md` ("[x] Published to crates.io — `efr32mg2x-pac` and `efr32mg2x-hal` (0.1.1)").
Both are corrected in the same change as this document. The registry also holds the name for a cooldown after a
deletion -- the PAC hit that on 2026-11-16 -- so the first publish attempt may be refused for that reason and not
because of the package.

## 2. What the package is, measured

```
cargo package -p efr32mg2x-hal   ->  94 files, packaged crate BUILDS (1m04s, from the tarball)
public modules                   26
public items                     446   (fn/struct/enum/trait/const)
unsafe occurrences               194   of which 88 carry a SAFETY note
#[test] inside the crate         58    (these are the cores; they RUN in their twin crates under libraries/)
host tests in the gate           157   across 18 core crates
examples that ship               20    of which 11 carry a dated hardware record
gate                             144 checks, VERDICT PASS
```

The HAL is structurally publishable: its only non-registry dependency is `efr32mg2x-pac = { path = "../efr32mg2x-pac",
version = "0.1.1" }` — a path+version pair, which resolves from the registry — and the PAC is published at that exact
version. There is no `#[path]` or `include!` reaching outside the package; the only `include_str!` is
`../docs/crate-doc.md`, which ships.

## 3. Evidence per peripheral, which is what "maturity" has to mean here

A driver's maturity in this project is not its line count; it is whether a record was read back off the part.

| peripheral | evidence | state |
|---|---|---|
| `pin`, `route`, `delay` | `hal_led_blink` `PASS` (2026-11-15), LED watched at 1 Hz; routing proved by every I2C example | **proven** |
| `bus` (bit-banged) | `hal_i2c_bitbang` `PASS scan n=3 3c 51 76` (2026-11-15) | **proven** |
| `i2c` blocking | `hal_i2c_recovery` `PASS ... depois=0x6a wr=1 rd6=1 nz=1 volta=0x40` (2026-11-23) | **proven** |
| `i2c` async | measured `who=0x6a` with no blocking call before it (2026-11-22), no dated marker yet | **measured, undated** |
| `dma` | `hal_dma_stop` `PASS ... stA=1 stS=0 stRe=0` (2026-11-23); LDMA streams the microphone (`hal_mic_level` `PASS`) | **proven** |
| `timer` | `hal_timer_compare` `PASS late=1 fired=1` and `hal_timer_ticks` `PASS fast=18961kHz` (2026-11-24) | **proven** |
| `sysrtc` | same two examples plus `hal_em2_sleep`/`hal_sysrtc_wide`/`hal_sysrtc_wrap` (undated) | **proven for count/compare; wide/wrap undated** |
| `wdog` | `hal_wdog_feed` `PASS fed 94 st80000000 sb0 lock1` (2026-11-24) | **proven** |
| `emu` | `hal_emu_voltage` `PASS 5/5 , 5/5`, three boots (2026-11-24) | **proven** |
| `adc`/`microphone` | `hal_mic_level` `PASS n=128 rms=7 pk=18` (2026-11-24) | **proven** |
| `sensors` (IMU) | `hal_imu_read` `PASS |a|=1028` (2026-11-24) | **proven** |
| `sensors` (BME680) | published to a phone as `0x181A` by `base_ble_env_gatt` (2026-11-14) | **proven, in the BSP** |
| `clock` (HFXO) | `hal_hfxo` undated; HFCLK rate cross-checked at 18.961 MHz by `hal_timer_ticks` | **partial** |
| `adc` (single-shot, `hal_adc_read`) | undated | **unproven** |
| `spi` | **no example at all** | **unproven** |
| `usart` | **no example at all** | **unproven** |
| `mvp` | `hal_mvp_bench` undated, and its numbers are `dwt`-based | **unproven, and suspect** |
| `dwt` | now known to count only while a debug session is attached (`DWT-NEEDS-A-DEBUG-SESSION.md`) | **a footgun as published API** |

Two of thirteen peripheral families have **no hardware evidence whatsoever** — `spi` and `usart` — and the README's own
standard is that a date means a record was read back. That is the single largest maturity gap, and it is cheap: both
drivers exist and compile; nobody has watched them move a byte.

## 4. Grades, with the reason

| dimension | grade | why |
|---|---|---|
| functional coverage | **B** | 26 modules, 446 public items, 15 peripheral families with drivers; SPI/USART unexercised |
| hardware evidence | **B−** | 11 of 20 examples dated, one of them (`hal_timer_ticks`) rebuilt twice before it measured anything real |
| documentation | **C+** | `deny(missing_docs)` is enforced — and bypassed on five modules with `#[allow(missing_docs)]`, so docs.rs would show 5 of 26 modules undocumented. 9 prose references point at `docs/0X-…` paths that do not ship in the package |
| API hygiene | **C** | `pub mod cores` and `cores::i2c::sim` put internals and a test double in the public API (review finding F5, still live); `Board` hands out unlimited aliases of its singletons (F10) |
| safety | **C** | 194 `unsafe` against 88 `SAFETY` notes: ~106 sites unannotated. Most are register writes, which is the cheap kind — but a published crate's auditability is the notes, not the kind |
| testing | **B** | 157 host tests across 18 core crates, run by the gate; the HAL's own `#[test]` blocks execute only through their twin crates |
| layering / packaging | **A−** | one registry-resolvable dependency, package builds from its tarball, no paths escaping the package, docs.rs metadata present |
| error handling | **B** | typed errors; bounded waits everywhere since 2026-11-17; but `wdog` still returns `Error::Locked` for a *timeout*, which is a semantic lie |
| observability | **A−** | the mailbox record pattern is better than most commercial HALs, and this week proved why: three of six "red" examples were the instrument, not the driver |
| publication hygiene | **D** | `check-publishable.py` only matches `include_str!` (not `include!`/`#[path]`); `pre-publish.sh` still runs `cargo publish --dry-run`, the command that certified a lying README once; and the CHANGELOG claims a publication that no longer exists |

**Overall: a prototype-grade HAL with an unusually strong evidence culture and an unusually weak publication surface.**
It is ahead of most hobby HALs on hardware proof and behind the bar on the things a stranger downloading a crate sees
first: documented public modules, no internals in the API, and a changelog that matches the registry.

## 5. What blocks publishing, in the order it should be closed

Each item is a document or API change; none needs the board.

1. **Five modules carry `#[allow(missing_docs)]`** (`cores::{adc,dma,i2c,sysrtc,usart}`). The crate advertises
   `deny(missing_docs)` and then exempts the modules a reader most wants documented. Write the docs; the behaviour is
   already covered by host tests, so this is writing, not proving.
2. **`cores::i2c::sim` is public.** A test double in the published API is a promise nobody wants to keep. Gate it
   behind a `sim` feature or move it to `#[cfg(test)]`.
3. **~106 `unsafe` sites without a `SAFETY` note.** Cheap to fix in bulk (most are `w.bits(x)` in a register write);
   worth doing before a stranger reads the crate.
4. **Nine prose references to `docs/0X-…`** in the shipped README/crate-doc. They resolve in the repository and not in
   the package — exactly the shape of the 0.1.0 failure, one notch quieter. Turn them into repository URLs.
5. **The registry lies in the repository**: `CHANGELOG.md` and `CLAUDE.md` both say the HAL is published at 0.1.1.
   Correct them, and decide the version — 0.1.x names a tree that no longer exists, so a fresh start at **0.2.0** is
   the honest number, with the CHANGELOG saying why.
6. **`dwt` must carry its condition in its own rustdoc.** A published `dwt::cycles()` that silently returns a frozen
   value once the probe detaches is a footgun; say so where the function is documented.
7. **`spi` and `usart` need one measured example each** before the README's evidence table can be read as complete.
   This is the only item here that wants the board.

## 6. Verdict

- **Publishable as a prototype crate (0.2.0) after items 1–6.** Those are documentation and API-surface changes; a day
  of work, no new hardware.
- **Not publishable as "stable" or 1.0.** The API will move (`Board` ownership, `transaction` semantics), two
  peripheral families have no hardware evidence, and the module documentation is incomplete by the crate's own lint.
- **The name may need a cooldown.** A deleted crate name is held by the registry for a period; the PAC waited until
  2026-11-16. Attempt one publish and read the refusal rather than assuming.

## 7. The publish checklist, when there is an express order

```bash
cd /projects/efr32mg24-rs && export TMPDIR=/projects/_tmp/tmp
bash scripts/gate.sh                                   # VERDICT: PASS, 144 checks
python3 scripts/gen-crate-readme.py crates/efr32mg2x-hal --check
python3 scripts/check-publishable.py                   # reads the package, not the tree
cargo package -p efr32mg2x-hal                         # builds from the tarball
# then, and only then, with the order in hand:
#   read README.md, docs/README.md, docs/crate-doc.md and every include_str! target AS THEY SHIP
#   confirm the version and the CHANGELOG entry match what the registry will show
```

A published version cannot be edited, only superseded — and this crate has already been withdrawn once.