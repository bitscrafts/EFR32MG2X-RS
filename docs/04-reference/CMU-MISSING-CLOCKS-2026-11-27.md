# The clocks nobody had measured: `FSRCO`, `HFRCOEM23`, `ULFRCO`

**Measured 2026-11-27 on the XIAO MG24 (EFR32MG24B220F1536IM48-B).** Eight flashes, eight boots, `status=PASS` on all
eight, and the two frequencies that matter repeat to 0.03 % and 0.11 %. The instrument is `examples/hal_cmu_sources.rs`
on top of `clock::sources` (`em01_select`, `hfrcoem23_band`, `hfrcoem23_cal`), `burtc::select_ulfrco` and
`Board::timer0_free_running`.

Three numbers for `HFRCOEM23` were in circulation before this run -- 18.5 MHz from a blink, 18.961 MHz from the tick
timer, "19.6 MHz" in a register note -- and none of them could be reproduced from a record. `FSRCO` and `ULFRCO` had no
measurement at all and no driver that could select them.

## The record

`FLASH_HOST=lnx FLASH_USER=mvcorrea bash scripts/flash.sh <elf> --confirm --run`, then
`read-mailbox.sh lnx mvcorrea`. Boot 7:

```text
cmu: ok=1 hx=38773318 hx2=38774405 hb=19065632 bnd=8 fs=20101417 ul=993 bu=988 tu=38774566
lf=32959 bl=32768 tl=38773363 ul0=0 ur=988 dr=0 em=2 bc=769 hs=0x00010001 hc=0xd868a141
```

Boot 8, the same firmware flashed again:

```text
cmu: ok=1 hx=38773399 hx2=38774757 hb=19064923 bnd=8 fs=20098902 ul=993 bu=988 tu=38774744
lf=32958 bl=32768 tl=38774102 ul0=0 ur=987 dr=0 em=2 bc=769 hs=0x00010001 hc=0xd868a141
```

The record carries the raw counts, not only the frequencies, so every number in the table below can be recomputed from
the line: `f = 39 MHz x counts / reference counts`.

| boot | `HFRCOEM23` | `FSRCO` | `ULFRCO` | `LFRCO` |
|---|---|---|---|---|
| 1 | 19.1757 MHz | 20.2278 MHz | -- | -- |
| 2 | 19.1756 MHz | 20.2063 MHz | -- | -- |
| 3 | 19.1732 MHz | 20.2153 MHz | -- | -- |
| 4 | 19.1752 MHz | 20.2148 MHz | -- | -- |
| 5 | 19.1770 MHz | 20.2182 MHz | 993.8 Hz | -- |
| 6 | 19.1787 MHz | 20.2191 MHz | 993.8 Hz | -- |
| 7 | 19.1771 MHz | 20.2184 MHz | 993.7 Hz | 32 959.5 Hz |
| 8 | 19.1763 MHz | 20.2157 MHz | 993.7 Hz | 32 958.9 Hz |

`ULFRCO` and `LFRCO` only appear from boot 5: the first four boots used an instrument that computed them but did not
put their raw counts in the record, and a rate nobody can recompute is the defect this file exists to avoid. Boots 1-4
are quoted for the two frequencies whose counts *were* in their records.

## What each number is judged against, and the verdict

| clock | measured | datasheet | verdict |
|---|---|---|---|
| `HFRCOEM23`, band 8 | 19.1732..19.1787 MHz | Table 4.27 note 6, band 8 = 16.4..24.7 MHz | inside |
| `FSRCO` | 20.2063..20.2278 MHz | Table 4.28 = 17.2 / 20 / 21.2 MHz | inside, +1.1 % from the 20 MHz typical |
| `ULFRCO` | 993.7..993.8 Hz | Table 4.30 = 0.944 / 1.0 / 1.095 kHz | inside, -0.63 % from the 1 kHz typical |
| `LFRCO` | 32 958.9..32 959.5 Hz | Table 4.29 = 32 768 Hz, normal mode +-3 % | inside, +0.58 % |

`bnd=8` is `HFRCOEM23.CAL.FREQRANGE` read back after `STATUS.RDY` came up (`hs=0x00010001`: `RDY` and `ENS`), and the
reference manual is explicit that the minimum and maximum for each `FREQRANGE` are "listed in the device data sheet" --
so band 8 is judged against the datasheet's band limits and not against a nominal. **This is not the +-3 % accuracy
row** of Table 4.27: that row is for production-calibrated frequencies, and 19.177 MHz is 4.11 % below the 20 MHz
calibrated point and 0.93 % above 19 MHz, so nothing here supports calling this oscillator either of those by spec.
What it supports is that the die is inside the band the datasheet gives for the trim it is running on.

That last qualification is why `hc` is in the record: `CAL` reads `0xd868a141`, not the reset word the reference manual
prints (`0xa8689f7f`: `FREQRANGE=8`, `TUNING=0x7F`, `FINETUNING=0x1F`). The die's word differs in `IREFTC` (`0xd` vs
`0xa`), `TUNING` (`0x41`) and `FINETUNING` (`0x21`). This crate never writes `CAL` -- no code path does -- so the trim
is whatever the part arrived with; the band is 8 either way, which is what the check needs. What is *not* established
is which frequency that trim was produced for.

## The referee, and why the obvious one is wrong

The queue for this item said "measured against the SYSRTC by window counting". That is the right shape and the wrong
referee: `SYSRTC` runs from `LFRCO`, which the datasheet specifies to +-3 %, so judging another oscillator's percent
against it would be judging one ruler by another of the same kind. The only traceable clock on the board is the 39 MHz
crystal, and every number above is a ratio to it:

- **`EM01GRPACLK` sources** (`HFXO`, `FSRCO`, `HFRCOEM23`) are counted by **TIMER0** free-running at 65536-count wraps,
  with the same timer pointed at the crystal before and after each source. The window is 32 768 `SYSRTC` ticks, and it
  cancels: `f = 39 MHz x counts(source) / counts(crystal)`. That is the one place the 3 %-accurate part is allowed to
  appear -- as a duration, never as a rate.
- **`ULFRCO` cannot clock `EM01GRPACLK`** at all (the selector has no entry for it), so it clocks **BURTC** through
  `EM4GRPACLK`, and TIMER0 on the crystal counts the window: `f = ticks(BURTC) x 39 MHz / counts(TIMER0)`. Here the
  window *is* the crystal, so nothing cancels and nothing has to.

`DWT` was not used: it freezes when the probe detaches (`DWT-NEEDS-A-DEBUG-SESSION.md`), and these records are read
from the mailbox after the run. `SysTick` was not used either: it is the core clock, and the core clock was one of the
things that had to stop being the explanation.

## The controls

1. **The instrument on its own reference.** `hx` and `hx2` are the crystal counted before and after the two sources,
   with `FSRCO` and `HFRCOEM23` in between: 38 773 318 against 38 774 405, `dr=0` per mille. The bracket is what says
   the window did not drift while the sources changed, and the tolerance in the code is 5 per mille -- 50x the observed
   value, because a check that is as tight as the measurement is a coin.
2. **The selector took.** `em01_select` refuses when `CLKSEL` reads back what was not written; the run only reaches a
   record if both selections read back, and `em=2` at the end is `EM01GRPACLK` left on `HFXO`.
3. **`ULFRCO` is not `LFRCO`.** The same BURTC is measured on `LFRCO` first (`bl=32768` ticks, 32.96 kHz) and then on
   `ULFRCO` (`bu=988` ticks, 993.7 Hz). A selector written and quietly ignored would show 32.9 kHz twice; a factor of
   33 between the two runs is not something a stuck register produces.
4. **The counter really is clocked -- both directions.** With `EM4GRPACLK` pointed at nothing, BURTC stands still for a
   full window (`ul0=0`) while the crystal keeps counting (`tl` is in the record); the source comes back and it counts
   again (`ur=988`). One direction alone would also be satisfied by a counter that stopped for ever.
5. **The window has a name.** `32768` SYSRTC ticks took `hx` crystal counts, which is 32 959.6 Hz -- the rate `SYSRTC`'s
   own selector is running at. It agrees with what BURTC measured on the same branch to 0.001 per mille, which is what
   two counters counting the same 32 768 ticks of the same branch *should* do: the value of the check is that a
   disagreement would mean one of the two selectors (`SYSRTC0CLK` and `EM4GRPACLK`) was pointing somewhere else. It is
   **not** independent evidence about which branch that is, and it is not quoted as any.
6. **The record fits.** `len=178` of 200 bytes, and the example fails outright if `Record::truncated()` -- a cut number
   reads as a different number, which is the failure mode the mailbox's torn-record protocol exists for.

## What would make these numbers lie

- **The crystal.** The datasheet's HFXO row is 38.0 / 39.0 / 40.0 MHz, not +-10 ppm: the module on this board is not
  characterised here, and every number above is linear in it. At the two ends of that range:

  | crystal | `HFRCOEM23` | `FSRCO` | `ULFRCO` |
  |---|---|---|---|
  | 38.0 MHz | 18.685 MHz | 19.700 MHz | 968 Hz |
  | 39.0 MHz | 19.177 MHz | 20.218 MHz | 994 Hz |
  | 40.0 MHz | 19.669 MHz | 20.737 MHz | 1019 Hz |

  All three verdicts survive it: band 8 (16.4..24.7 MHz), `FSRCO` (17.2..21.2 MHz) and `ULFRCO` (944..1095 Hz) are
  wide enough that the crystal's own tolerance cannot move any of them out. The *absolute* frequencies carry that
  uncertainty and the record does not pretend otherwise.
- **Temperature.** One bench, one afternoon, one temperature. `HFRCOEM23` is not temperature-compensated (the tick
  timer notes already say so); `ULFRCO`'s datasheet limits are specified over the operating range, which was not swept.
- **What the trim was produced for.** `hc=0xd868a141` is the word the part arrived with; nothing here writes `CAL` from
  `DEVINFO`, so the frequency is that of an unknown production-calibration target inside band 8.

## What this changes in the HAL

- `HFRCOEM23` at 19.177 MHz is 0.93 % from the 19 MHz the pacing code and the tick-timer example assume, and those two
  places state their assumption and their bands. Nothing needs re-calibrating; what changes is that the 19 MHz is now a
  measured number with a date instead of a nominal.
- `pace_microphone` and `adc` force `HFRCOEM23` on and point a selector at it. Both keep working, and `timer0_free_running`
  is the door for an instrument that must *not* have its group clock changed underneath it.
- `clock::sources` is the only place the datasheet's numbers for these parts live; the module says which table each one
  came from, so a regeneration of this file cannot silently invent them.

## Not measured, and not claimed

- `HFRCODPLL` (the core's own oscillator) -- it is only reachable as `SYSCLK`, and `clock` treats it as the reset
  default, not a frequency anyone has measured.
- `LFXO` -- this SVD has no `CMU_OSCENCMD` and no `LFXOEN` bit, so the crystal cannot be started, let alone timed
  (`clock::Clocks::new` refuses it with that reason).
- `HFRCOEM23` in EM2/EM3 (`CTRL.EM23ONDEMAND`) and `ULFRCO` across an EM4 hibernation: the counters were measured in
  EM0. `BURTC-2026-11-27.md` measures the counter's behaviour across a *core* reset, which is a different claim.
- Any oscillator's behaviour across the operating-temperature and supply range.