# Brief — the dosimeter crate: NR-15 by the book, and the Leq the docs promise

Work package for the `2 - UFC` window. Everything here is in `crates/dosimeter-mg24/`, which is pure Rust,
`no_std`, `deny(unsafe_code)` and testable on the host. No hardware, no HAL, no PAC.

## Why this is the right thing to do now

A review of the spec and the crate (2026-11-17) found two defects that make the numbers the app will show
wrong, and one of them is a correctness issue with the standard itself:

1. **The dose is computed with a continuous formula instead of the standard's table.** The crate uses
   `T(L) = 28800 · 2^(-(L-85)/q)` with a 5 dB exchange rate. NR-15's Annex 1 gives a *table* of maximum daily
   exposure per level, and the rule between two entries is "take the next higher level" — not interpolate, and
   certainly not extrapolate a curve. Measured effect: at 97 dB the continuous formula *understates* the
   exposure by up to 17,5 %, because the standard's table steps up while the curve slides between entries.
   A dosimeter that under-reports exposure is the one failure mode this device exists to prevent.
2. **`Leq` is promised in the crate's documentation and does not exist as a function.** The docs describe a
   continuous-equivalent level; the code has `Leitura`, `intervalo`, `tempo_admissivel_s`, `Dose`, `nova`,
   `acumula`, `percentagem`, `tempo_s`, `repor`, `tempo_restante_s`. Someone reading the docs and calling
   `Leq` gets a compile error — which is the good outcome; the bad outcome is that the *report* quotes an Leq
   that was never computed.

## What to build

```
1. NR-15 table      the maximum daily exposure per level, from the standard's Annex 1, as data. Between two
                    entries, the "immediately higher level" rule applies. Keep the table in one place, with
                    the standard's own wording quoted next to it, so the next person can check it.
2. Leq              Leq = 10·log10( (1/T) · Σ ti · 10^(Li/10) ), over (level, duration) pairs, in dB.
                    No libm on the target: use fixed-point or document exactly which libm function is
                    needed and why. The crate is no_std and must stay that way.
3. Dose from Leq    dose % = 100 · (T_measured / T_allowed(Leq)). Check it against the two cases everybody
                    knows by hand: 85 dB for 8 h = 100 %, and 90 dB for 4 h = 100 %.
4. Tests            at least three new host tests, and they must include both sides of the boundary:
                    - a level exactly on a table entry (no rounding up)
                    - a level between two entries (must round up, and the test must fail if it interpolates)
                    - the 85 dB/8 h and 90 dB/4 h pair, both to within 0,1 %
```

## Acceptance — the command, not the claim

```bash
cd /projects/efr32mg24-rs
cargo test -p dosimeter-mg24 --target x86_64-unknown-linux-gnu
```

**The `--target` is not optional.** The repository's `.cargo/config.toml` pins the embedded target, and a
`no_std` crate built as a *test* for `thumbv8m.main-none-eabihf` fails with `#[panic_handler] required`. That
is not a defect in the crate; it is the wrong command, and it has already cost time once.

The record to report back: the test names, the counts, and the two dose percentages against the hand numbers.
A "tests pass" without the names does not travel.

## Boundaries

```
touch      crates/dosimeter-mg24/**   only
never      crates/efr32mg2x-hal/**, crates/efr32mg2x-pac/**, crates/xiao-*/**   another window owns them
never      cargo publish / anything that writes to crates.io
never      moving or renaming files outside the crate (this is what broke HEAD on 2026-11-17: a board.rs
           was moved and the two lib.rs were not adjusted, and no gate was run)
commit     only after the test command above passes, and after `bash scripts/gate.sh` prints VERDICT: PASS
```

## What not to do

- Do not "fix" the discrepancy by tuning the exchange rate until the two cases match. The table is the
  standard; the formula was the shortcut.
- Do not add an `Leq` that returns a plausible number for an empty input. Empty means "no measurement", and
  the type should say so rather than returning 0 dB — 0 dB is indistinguishable from a real reading of silence
  on a slide, and that is exactly the kind of number this project has spent the week removing.
