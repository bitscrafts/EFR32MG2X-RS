# The application loop does not fit at 19 MHz — measured, 2026-11-17

## What was measured

The loop the product needs: paced 16 kHz capture, eight band energies per window, and the normative dose. The
dose half is not wired in yet (it lives in its own project and the dependency runs the other way), so this is
the loop without it — which makes the number below a **floor**, not a worst case.

```
app: windows=63 short=0 cycles=20735103 timer=1 bands=377810863 pct_of_realtime=108

63 windows of 256 samples at 16 kHz = 1,008 s of audio = 19 152 000 cycles at 19 MHz
the loop took 20 735 103 -> 108 % of real time
329 000 cycles per window against 304 000 available
```

## Why `short=0` was lying

`capture_window` returns how many samples arrived, and it returned 256 every time. That reads like a healthy
capture and is not one: when the CPU falls behind, the converter does not stop — it overruns, the FIFO drops
what it cannot hold, and the window still comes back full, with gaps. **Only the clock against real time says
whether the loop kept up**, which is why the criterion is now `pct_of_realtime <= 102` and not "no short
windows". This is the same class of error as the earlier false greens: a check that cannot fail while the thing
it checks is broken.

## The arithmetic of the budget

```
capture          304 000 cycles per window — the CPU is asleep on the timer, but the time is spent
8 Goertzel bands  46 044 cycles measured over 256 samples (22,5 cycles per step)
needed           ~350 000 cycles for a window that must fit in 304 000
```

So the loop is over by about 15 % before the model, the dose and the radio are added. Every one of those makes
it worse.

## The three ways out, with what each costs

```
1. features on every second window   half the feature rate: 31 a second instead of 62. Fits, and the audio
                                     windows are still 62. Loses temporal resolution in the model only.
2. DPLL to 78 MHz                    the same 350 000 cycles complete 4,1x faster. This is the first measured
                                     reason to run the part at its rated speed: not "more headroom", but the
                                     only option that keeps 62 feature windows a second.
3. the MVP for the bands             the driver exists and measured 3,18x on a 64-element fp16 add. The bands
                                     are MACs over 256 samples, which is the shape the MVP is for — but its
                                     MAC encodings are not in this repository yet (only the vector add is).
```

Option 2 also rescales nothing else: the measured numbers in this project are cycles, so at 78 MHz the same
work takes 4,1x less wall time and every existing measurement stays true.

## What this changes in the plan

The DPLL moves from "last, a nicety" to "next, because the loop does not fit without it" — and it arrives with
a number it has to beat: **the same loop at the same rate, `pct_of_realtime` below 102**. If the DPLL is
configured and that number does not fall to roughly a quarter, the configuration is wrong, whatever the
registers say.

## RESOLVED — the loop fits at 19 MHz, because the waiting was the resource (2026-11-17)

The loop was never short of arithmetic. A paced read spends 1187 cycles per sample waiting for the timer, of
which about ten are work: **1177 idle cycles per sample, 301 312 per window, against 46 044 needed for the eight
bands.** The first harness captured a whole window and only then computed, so those idle cycles were thrown away
and the loop came out 108 % of real time.

Advancing the eight Goertzel states one step per sample — inside the wait — changes the answer without touching
the clock:

```
interleaved: windows=63 missing=0 cycles=19151407 pct=99 timer=1 bands=464915712   status=PASS

63 windows of 16 kHz audio in 19 151 407 cycles, against 19 152 000 cycles of real time: 99 %
```

So the three options listed above were not the whole list: the fourth, and the right one, is to use the wait.
The DPLL work was not wasted — `Board::boost_to_38/56/80_mhz` is measured and documented — but it is **margin
for the model and the radio**, not the thing that makes the loop fit.
