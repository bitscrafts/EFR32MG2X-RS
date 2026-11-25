# The DWT cycle counter only runs while a debug session is attached

**Measured 2026-11-24, on the XIAO MG24, and it invalidates any timing taken from a mailbox record that was read after
the probe detached.** Six examples in this repository and the dosimeter's cycle budget use `dwt`. They are not wrong,
they are *conditionally* right: `probe-rs run` watching RTT is a measurement, a record read later over SWD is not.

## The two probes, both anchored outside the loops

Nothing inside the firmware can settle "is my clock slow or stopped" -- that is the mistake that cost this session two
attempts. The host's clock is the only reference outside both.

**Detached** (flashed, then read over SWD twice, 10.8 s apart on the host clock):

```
probe: t=1865376137 c=29605      SYSRTC and DWT at the first read
probe: t=1865722228 c=29605      same DWT value 10.8 s later
Δ SYSRTC = 346 091 ticks / 10.795 s = 32 060 Hz  (correct, nominal 32.768 kHz)
Δ DWT    = 0                                    (frozen)
```

**Attached** (`probe-rs run` reading RTT, same firmware family):

```
DWT/SYSRTC = 578.4   over any interval
19 MHz / 32.768 kHz = 579.8
```

Both counters agree to 0.2 %. So the DWT is correct *while attached* and frozen while detached: it lives in the debug
power domain, and the probe's detach takes its clock with it. `DCB::enable_trace()` and `DWT->CTRL.CYCCNTENA` are both
set and readable (`DEMCR=0x01100000`, `CTRL=0x40000001`) while the counter stands still -- the writes landed and are
not the issue.

## What this cost, and the rule that follows

`hal_timer_ticks` failed twice for this reason. First it waited on the cycle counter itself (a window defined by the
thing under test); then it took the window from the SYSRTC and read `core=283kHz`, because the DWT had stopped 1.5 ms
into the boot and only a fraction of the window was counted. The same 29 000-ish final value appeared in both.

**Rule: for a measurement that has to work from a standalone run, the fast-clock reference is a peripheral counter**
-- SysTick (CPU-clocked) or, when the board already owns SysTick for its delays, a TIMER counting at its own clock; neither
cares about the debugger. `hal_timer_ticks` now derives the fast clock from TIMER0 counts and SYSRTC ticks over the same window, and reads
`cyc=1896280 fast=18961kHz over ticks=3277` (0.2 % from the 19 MHz nominal of HFRCOEM23).

**Rule: if a number came from `dwt`, say whether the probe was attached.** For the examples here: `hal_mvp_bench`,
`hal_sysrtc_wide`, `hal_timer_ticks` (before this change), `module_clock_probe`, `module_mic_rate` and
`module_ble_dosimeter` all use `dwt::start`/`cycles`, so a date next to them is only valid if it was taken under
`probe-rs run`. A record read from the mailbox afterwards is evidence about something else.

## The pattern behind both failures

Both attempts measured the effect and then invented the cause -- "a timing race", "the bus is held". The instrument
that answered each time was the one anchored outside the disputed component: the host clock for the DWT, and the
SYSRTC (already measured against the host) for the tick timer. **When two clocks disagree, get a third that neither of
them feeds.**