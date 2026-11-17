# Pacing the microphone — closed 2026-11-17 (the attempt trail kept as the record)

The dosimeter needs windows at a chosen rate (16 kHz). Today the driver reads at whatever rate the asking
produces (303 148 Sa/s measured). This document records the attempts, the faults they raised, and the exact
point where the work stands — so the next attempt starts from evidence rather than from scratch.

## Proven

```
TIMER0 runs at the asked rate     `Board::pace_microphone(16_000)` -> period 1187, counter moves, 16 006 Hz
TIMER0 is write-protected         LOCKKEY 0xCE80, EN=0 before CFG, TOP written with EN=1
                                  (writing TOP with EN=0 raised a precise bus error at 0x4004_801C)
IADC TRIGGER is write-protected   writing it while the converter runs raised a precise bus error at
                                  0x4900_402C (CFSR=0x00008200 PRECISERR|BFARVALID, HFSR=0x40000000)
SINGLETRIGSEL=Timer is NOT TIMER0 the register description says "Triggers when the local timer count reaches
                                  zero": the IADC has its own down-counter in its own register block
sample() defeats pacing           it issues CMD.SINGLESTART on every call, so a paced window still filled at
                                  465 kHz (256 samples in 10 456 cycles)
the queue needs arming            without one CMD.SINGLESTART the paced read never delivered a single
                                  conversion (n=0, 14,3 M cycles spent waiting)
```

## RESOLVED — the four rules that were missing, all measured

```
rate achieved        1187 CLK_SRC_ADC cycles per sample -> 256 samples in 303 895 core cycles
                     = 16 005 Hz, status=PASS, distinct=64 values, STATUS.TIMERACTIVE=1

rule 1  the local timer counts CLK_SRC_ADC, not ADC_CLK
        RM p782: IADC_TIMER.TIMER = "Number of CLK_SRC_ADC cycles per timer event". The emlib header
        (em_iadc.h:817) says ADC_CLK and is wrong against the reference manual. Proven on the board:
        a period of 593 produced exactly 593 cycles per sample, i.e. 32 kHz = 19 MHz / 593, not 16 kHz
        = 9,5 MHz / 593.

rule 2  CONFIG registers may only be written with EN=0, COMMAND registers only with EN=1
        TRIGGER written with EN=1   -> precise bus error, BFAR = 0x4900_402C
        CMD written with EN=0       -> precise bus error, BFAR = 0x4900_400C
        There is no instant when both are allowed, so the order is: EN=0 -> TRIGGER + TIMER -> EN=1
        -> arm -> TIMERDIS + TIMEREN (RM p48, p748 for config, p778 for CMD being synchronised).

rule 3  the queue must be armed once (CMD.SINGLESTART) before the timer is enabled
        without it a paced read delivered nothing at all: n=0 after 14,3 M cycles of waiting.

rule 4  a paced read must pop the FIFO
        SINGLEDATA leaves the valid flag set, so the loop returns instantly with the same value:
        measured 256 samples in 8 741 cycles under a 16 kHz timer. SINGLEFIFODATA pops and the loop
        then waits for the timer, which is the whole point.
```

TIMEBASE was already correct — `microphone.rs` writes 19, and the RM (§p779) says it only times the warmup,
so it was never the blocker even though it looked like the missing variable.

## Continuity across windows — 2026-11-17

A single window proves the pace; eight prove the architecture, because features are computed between windows
and nothing buffers audio in hardware in the meantime.

```
stream: windows=8 short=0 min=303899 max=304034 sum=2432011 total=2432177 timer=1   status=PASS

8 consecutive windows, none short; each 303 899..304 034 cycles (16 ms of audio at 19 MHz)
sum of the eight = 2 432 011 against a measured total of 2 432 177 -> 166 cycles of slack, 0,007 %
```

Two things this settled:

- `Microphone::capture_window` returns **how many samples arrived**, not a Result: a streaming loop needs to
  know whether it kept up, and "short by 7" is a different fact from "the converter stopped".
- `STATUS.TIMERACTIVE` must be read **after** the work. `CMD` is synchronised with the ADC clock, so a read
  immediately after `start_timer` returned 0 while eight windows were plainly being paced — the flag is true,
  it is just late.

## The open question, as it stood before the resolution

```
with the queue armed and the local timer configured, 256 samples still arrive in 8 741 cycles (556 kHz),
not in 4 864 000 (16 kHz): the conversions are free-running, so the local timer is not the thing that
gates them
```

Hypotheses, in the order they are worth testing:

1. **`ADC_CLK` is not 19 MHz.** The local timer counts ADC clock cycles after `CTRL.PRESC` and the high-speed
   rate divider. If the converter runs at 19/4 MHz, a period of 1 187 asks for 4 kHz, not 16 kHz — and a
   mismatch in that direction would still *produce* conversions, so this does not explain free-running alone.
   Read `CTRL` back and compute the true ADC clock before setting the period again.
2. **The local timer needs `CMD.TIMEREN` after arming, not before.** Order: arm the queue, then enable the
   timer.
3. **`SINGLETRIGSEL=PRSCLKGRP` with PRS0 routed from a timer in the ADC's clock group** is the documented
   alternative and the one the silicon's own examples use for continuous sampling; it costs a PRS channel and
   is more moving parts, but it is explicit rather than implied.

## What the reference manual adds (2026-11-17, §23.3.3.4 and the CMD register text)

The manual was obtained and read; three statements change the picture, all quoted:

```
SINGLETRIGSEL  "TIMER - Use the IADC local timer to trigger conversions. This is useful for triggering
                conversions at precise intervals."                                       (§23.3.3.4)

TRIGACTION     "Setting SINGLETRIGACTION to ONCE will make one conversion request each time the selected
                single trigger occurs... Setting SINGLETRIGACTION to CONTINUOUS allows the single trigger to
                begin the first conversion, and when a conversion completes a new one will be requested
                immediately without requiring a new trigger."                            (§23.3.3.4)

CMD            "TIMERDIS  Disable the local timer and reset the counter to timer reload value."
               "TIMEREN   Enable the local timer."
STATUS         "TIMERACTIVE  The local timer is running."
```

The CONTINUOUS sentence explains the free-running window exactly: with CONTINUOUS the first trigger is enough and
every later conversion is requested without one — measured 256 samples in 8 741 cycles, 556 kHz, which is the
converter's own speed. So CONTINUOUS is not a pacing setting at all.

Three orders were then tried on the board with TRIGACTION = ONCE, and **`STATUS.TIMERACTIVE` read 0 in all
three**, with the queue armed:

```
1. TIMER (reload) -> EN=1 -> TIMEREN              timer=0, window still free-running
2. TIMER (reload) -> EN=1 -> TIMERDIS -> TIMEREN   timer=0, window still free-running
3. TIMERDIS -> TIMER (reload) -> EN=1 -> TIMEREN   the run hung; no record written
```

The two methods written for this (`pace_from_timer`, `read_paced`) were then removed from the driver rather than
left as a plausible-looking API that does not pace anything. What remains is `timer_active()`, a read-only
window on `STATUS.TIMERACTIVE`, so the next attempt can tell "the timer did not run" from "the converter did not
deliver" without a new method.

The next thing to read is the IADC chapter's register map around `IADC_CMD` and `IADC_TIMER` (offets 0x00C and
0x10C in the manual) to see whether `TIMEREN` is conditioned on something this driver has not set — the prose
says only "enable", and the prose is evidently not the whole story.

## The cost of not having it

At 16 kHz the dosimeter's audio costs 18,3 % of the core (measured: 37,8 cycles per sample acquired plus
46 044 cycles for eight bands per 256-sample window). At the current free-running 303 kHz the same code costs
62 % just to acquire. So this is not a nicety: it is the difference between a core that has room for the model
and the BLE stack and one that does not.

## How to re-run

`crates/xiao-mg24-bsp/examples/module_mic_paced.rs` is the instrument. It reports `hz_asked`, the period it
programmed, the sample count, the cycles spent and how many distinct values came back, and it **fails** while
the achieved rate is not within 20 % of the asked one. Today it fails with `n=256 cycles=8741` — that failure
is the open question, recorded rather than hidden.

## What not to do

Do not lower the pass threshold to make the example green. A window that took 8 741 cycles is not a 16 kHz
window, and the number is the only thing that says so.
