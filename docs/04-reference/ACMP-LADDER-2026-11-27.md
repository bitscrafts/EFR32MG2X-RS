# ACMP0: the internal ladder, swept against an internal source, with three controls

Date: 2026-11-27. Item 7 of `_tmp/night/STATE-NIGHT2.md` (queue: "ACMP0 / comparador"). Board: XIAO MG24 Sense
(EFR32MG24B220F1536IM48-B) on the bench at `lnx`, CMSIS-DAP probe. Example:
`crates/efr32mg2x-hal/examples/hal_acmp_ladder.rs`; driver: `crates/efr32mg2x-hal/src/acmp/mod.rs`; arithmetic:
`src/cores/acmp.rs` (+ `libraries/acmp-core`, six host tests and a printed table); hand-out: `Board::acmp`.

The comparison is entirely on-chip: no pin, no wire, no external component. The positive input is the internal sense node
the register list calls `VSENSE0/4`, the negative input is the reference ladder, and the ladder is walked from tap 1 to
tap 63 -- 63 comparisons per sweep, four sweeps, 252 comparisons a run.

## The record

Three flashes, three boots with the formatted tree, `status=PASS` in all three:

```
acmp: ok=1 n=252 c1=43 c2=22 c3=43 e=111 f=0 u=0 lo=833 hi=853 sw=01 off=01 st=0x00000000 ic=0x003f2012
acmp: ok=1 n=252 c1=44 c2=22 c3=44 e=111 f=0 u=0 lo=853 hi=873 sw=01 off=01 st=0x00000000 ic=0x003f2012
acmp: ok=1 n=252 c1=43 c2=22 c3=43 e=111 f=0 u=0 lo=833 hi=853 sw=01 off=01 st=0x00000000 ic=0x003f2012
```

| field | meaning |
| --- | --- |
| `n` | comparisons taken in the run: 4 sweeps x 63 taps |
| `c1` | first `VREFDIV` tap that read low, sweeping against the **1.25 V** reference |
| `c2` | the same against the **2.5 V** reference |
| `c3` | `c1`'s sweep repeated |
| `e` | edges per sweep, in sweep order 1-2-3: one each |
| `f` | edges in the flat control -- 63 comparisons at the pass-through tap, so 0 |
| `u` | samples where the eight repeated reads disagreed -- 0 |
| `lo`/`hi` | the intersection of that run's three windows, in millivolts |
| `sw` | the two swap-control reads at `VREFDIV = 63`: `pos=VSENSE0` then `pos=1V25` |
| `off` | `STATUS.ACMPOUT` with `EN = 0` and `CTRL.NOTRDYVAL` 0 then 1 |
| `st` | `STATUS` at the end (block disabled: ready bit clear) |
| `ic` | `INPUTCTRL` read back: `possel=0x12 negsel=0x20 vrefdiv=0x3f` |

## The number, and why it moves by one tap between boots

The arithmetic is in `cores/acmp.rs` and printed on the host by
`cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-acmp-core --example ladder-table`:

- 1.25 V reference, crossing at tap 43: the node is between `1250*42/63 = 833` and `1250*43/63 = 853` mV.
- 1.25 V reference, crossing at tap 44: between 853 and 873 mV.
- 2.5 V reference, crossing at tap 22: between `2500*21/63 = 833` and `2500*22/63 = 873` mV.

The two references place the node on the same boundary: 853 mV is exactly tap 43 of the 1.25 V ladder, and the node sits
within a millivolt or two of it. That is why the 1.25 V sweep crosses at 43 in one boot and 44 in another -- one tap is
19.8 mV, so a signal on the boundary lands on either side -- and why each run's own three-sweep intersection is one tap
wide, `833..853` or `853..873`. Across the three boots the node is in **833..873 mV, at about 853 mV**; nothing narrower
is claimed, because the instrument is the ladder and its resolution is one tap.

The example's repeatability rule is one tap for the same reason: `c1` and `c3` are the same sweep twice, and a
repetition that moved two taps would be a moving signal rather than a quantised one. The 2.5 V crossing (`c2 = 22`)
halves correctly in every boot, which is the check that survives the quantisation: 43 against 22 and 44 against 22 are
both "half, to one tap".

## What each run rules out

1. **A ladder that is written and not applied.** `e=111` with `c1` at 43 or 44: the output is high for the taps below the
   crossing and low above it, one contiguous run each way. A `VREFDIV` that the mux latches but never applies gives zero
   edges (all high or all low); a stuck bit in the ladder gives more than one.
2. **A crossing that belongs to the comparator's offset rather than to the ladder.** The 2.5 V reference is twice the
   1.25 V one, so the crossing must land at half the tap: `c2=22` against `c1=43..44`. An offset would not halve, and
   would show as the same crossing against both references.
3. **The flat control that makes point 1 mean something.** The same 63 comparisons at one tap (`f=0`): if that run had
   turned as well, the edge counter would be measuring noise and `e=1` in the sweeps would prove nothing.
4. **A crossing that is a single sample of noise.** `u=0`: every sample is eight identical reads, and the repeat sweep
   lands within one tap. A flickering output shows up as an unstable sample instead of as a crossing.
5. **Control: the inputs swapped at the pass-through tap.** `sw=01`: `VSENSE0/4` against 1.25 V reads low (853 mV <
   1250 mV) and the same two sources the other way round read high. Without this, "high at low taps" would also be
   satisfied by an output stuck at 1 while enabled.
6. **Control: the block disabled.** `off=01`: with `EN = 0` and the inputs left as they were, `STATUS.ACMPOUT` follows
   `CTRL.NOTRDYVAL` -- 0 when it is 0, 1 when it is 1. A disabled comparator is not "a comparator that says low", which
   is why the driver exposes `is_ready` and the example checks it before believing a sweep.

## Two facts paid for with a fault

Both were found by the first two images of this item, and both are encoded in the driver.

1. **The bus gate is `CMU_CLKEN1.ACMP0`, not `CLKEN0`.** Every other analogue block this crate drives (`IADC0`) is in
   `CLKEN0`; `ACMP0` and `ACMP1` are in `CLKEN1` (`efr32mg24_cmu.h:674`, bit 18). With the gate clear the block does not
   answer at all -- the first register read is a precise bus fault at `ACMP0_S + 0x04`:

   ```
   acmpprobe: FAULT stage=2 pc=0x323d6567 cfsr=0x00008200 bfar=0x49008004
   ```

   The same image then read the non-secure alias at `0x5900_8000`, which faulted identically (`bfar=0x59008000`), so it
   is the gate and not the alias. With the gate set, both aliases answer and `IPVERSION` reads `2`. A control in the same
   image: reading `IADC0_S` (`0x4900_4000`) with *its* gate clear faults the same way (`bfar=0x49004000`), which is what
   says the fault is the gate rather than the block.

2. **`INPUTCTRL` is not writable with `EN = 0`.** The next image configured the block disabled and faulted at
   `ACMP0_S + 0x14` (`bfar=0x49008014`). `CFG` and `CTRL` *are* writable disabled -- the enable sequence writes both --
   but the input mux wants the analogue domain awake. `Acmp0::new` therefore enables first, and `compare` is only
   reachable after it.

A third, smaller measurement from the same night: `SYNCBUSY.INPUTCTRL` clears when the mux has *accepted* the write, and
the analogue path settles after that. The first run of the example read the output immediately after the swap control and
got the previous comparison (`sw=00`); with 20 000 cycles of settling it reads `sw=01` on every boot since. Every sample
in the example settles before it reads.

## What this does not prove

- **Not a calibrated voltage.** The ladder and the reference it divides come from the same bandgap, so "about 853 mV"
  means "this is where this part's ladder crosses this part's sense node". The interval is computed from the reference's
  name (1.25 V / 2.5 V) and inherits whatever trim error it has. Nothing here is a voltmeter.
- **Nothing is claimed about `VSENSE0`.** The register list names the node and divides it by four; what it physically is
  (temperature sensor, supply sense, something else) was not measured.
- **Not the pin path.** No `POSSEL`/`NEGSEL` entry that names a package pin was driven. An earlier throw-away probe did
  walk the AVDD tap and the 2.5 V tap, but that image is in `_trash/` and no number from it appears here.
- **Not the interrupt path.** `IF`/`IEN`, rise/fall, `ACMP1`, hibernate mode, the capacitive-sense resistor bank and the
  GPIO output route are not exercised; the driver does not expose them.
- **Not `VREFDIV = 0`.** The sweeps run 1..=63. Tap 0 is 0 V by the same arithmetic and was not compared.
- **Not temperature.** Three boots inside two minutes on a bench; the node moved less than one tap.