# Fall detection — calibration against real drops

Three rounds of drops performed by hand, recorded by the board, and used to set every
constant in `crates/motion-detection`. Nothing here is a datasheet number.

## The rounds

| round | sample rate | drops performed | drops detectable in the recording | detected |
|---|---|---|---|---|
| 1 | 200 Hz | 2 | 2 | **2** |
| 2 | 200 Hz | 5 | 3 (two lost their free fall between samples) | **3** |
| 3 | 400 Hz | 4 | 4 | **4** |
| control | 200 Hz | 0 (handling: shake, set down) | — | **0 false alarms** |

## What the data said, and how it changed the design

### 1. The impact cannot separate a fall from a knock

Real landings, nine of them: **1148, 1329, 1493, 1759, 1931, 2268, 3885, 10084,
12823 mg**. The handling recording peaked at **1801 mg**.

The ranges overlap: a board laid down flat lands more gently than a board shaken hard.
Any single impact threshold either misses the soft landings or alarms on handling. The
discriminator is the **free-fall phase** — and the impact threshold only has to exclude
"returned to one g because it was caught":

```
FREEFALL_MG     400    drops reach 23-50 mg; handling never goes below 630 mg
FREEFALL_MIN_MS  40    shortest real free fall 42 ms; handling dips 10-25 ms
LANDING_MG      1100   softest real landing 1148; a caught board returns to ~990
IMPACT_MG       2000   a bang with no free fall (handling peaks at 1801)
QUIET_MG        1500   "it has stopped moving"
SETTLE_MS        600   see below
```

### 2. The impact does not always follow the free fall immediately

In the first round the board read about one g for 120 ms between falling and the
impact, and in the second for 400 ms. A state machine that required the impact *at the
end of* the free fall detected **zero of two** drops. The corrected model opens a
600 ms window and accepts the landing anywhere inside it.

### 3. A band for "still" was wrong; "quiet" is right

Requiring the magnitude to stay within 700..1300 mg for 800 ms lost two of five drops
in round 2: the board bounces with the USB cable attached, and each bounce reset the
timer. The rule is now "below 1500 mg for 600 ms" — a landed board is quiet, not
necessarily still in a band.

With 800 ms, round 3 scored 3/4. At 600 ms it scores **4/4**, and the handling
recording is still silent.

### 4. 200 Hz was not enough to see the free fall

Round 2 lost two drops entirely: their free fall never appeared in the samples and the
event read as a bare impact (correctly reported as `Impact`, not `Fall`). At 400 Hz the
shortest free fall in round 3 measured **42 ms — about 17 samples**. The project now
samples at 400 Hz, just under the sensor's 416 Hz output rate, and the detector's timers
are in microseconds so 2.5 ms per sample is exact.

## Where it stands

```
handling  (real, 320 samples)   -> no fall, no false alarm
round 1   (real, 2 drops)       -> 2/2
round 2   (real, 3 detectable)  -> 3/3
round 3   (real, 4 drops)       -> 4/4
synthetic (constructed)         -> detected
```

On the device, at boot, without a host:

```
fall selftest real=clean synth=ok 200Hz=2/2 400Hz=4/4
```

## What is still not proven

- **Nine drops is a small sample**, and all nine were the same board on the same desk,
  dropped the same way by the same person. A claim about fall detection in general
  needs falls from different heights, orientations and surfaces.
- **A real fall is not a dropped board.** A person falling takes their device down with
  them: the free fall is shorter and the landing is cushioned by a body. The classifier
  is tuned to a device falling onto a desk, which is the harsher case for the impact
  threshold and the easier one for the free-fall phase.
- The **gyroscope is not used**. Rotation would separate "dropped while flat" from
  "tipped over", and the sensor is already there and already read.

## The time base was wrong, and the log found it

Everything above was analysed on a time axis that did not exist. The sampler was written
to run at 400 Hz and handed the detector a fixed `dt = 2500 µs`, but the loop was actually
running at **158 Hz**: the display flush is 1090 bytes on a bit-banged bus and blocks for
tens of milliseconds, so the real interval was ~6.3 ms while every timer believed 2.5 ms.
Every duration in the first analysis was therefore **2.5 times too short**.

The event log is what exposed it. Four drops onto a bed reported free falls of 85, 52, 120
and 40 ms, and the gaps between the airborne event and the fall were all suspiciously
close to 0.62 s — the settle timer, unmoved by any property of the world.

Two fixes, both measured rather than assumed:

1. **The interval is measured, not declared.** The loop takes the difference of the DWT
   cycle counter between samples and hands *that* to the detector and the recorder.
2. **Every recorded sample carries its own interval**, one byte in 250 µs units
   (`recording::DTS`). A dataset whose timing is unknown can only be analysed dishonestly.

The display was also moved out of the critical path: refreshed every 800 samples instead
of 80, and never while the device is airborne or within 1.2 s of an event. That took the
loop from **158 Hz to ~257 Hz** without touching the sensor.

### What the corrected round measured

Four drops onto the same bed, with real timing:

| free fall | landing | settle |
|---|---|---|
| 176 ms | 11699 mg | 1.04 s |
| 223 ms | 13840 mg | 1.05 s |
| 248 ms | 13149 mg | 1.14 s |
| 252 ms | 13742 mg | 1.17 s |

**4 of 4 detected.** The free falls are 176–252 ms, not the 40–160 ms the compressed axis
suggested, and the bed is not soft: landings measured 11.7–13.8 g. The prediction that a
mattress would land below `LANDING_MG = 1100` was wrong.

### What this leaves open

The **false-positive control was measured on the compressed axis**, so "handling produces
no fall" is not yet proven under real timing. It needs one handling round recorded by the
current firmware before the detector can be called validated rather than promising.
