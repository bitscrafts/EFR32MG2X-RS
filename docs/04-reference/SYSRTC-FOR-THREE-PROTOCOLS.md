# SYSRTC, for the three protocols that will share it

Written 2026-11-13 because the user asked for these results to be kept: *"salve os resultados, esses poderão
nos ajudar no zigbee e no thread!"*. This is the durable part — the offsets, the measured state machine, the
instruments, and the decision that is still open. The blow-by-blow is in `SYSRTC-DEFECT.md`; the independent
review is in `SYSRTC-OPUS-REVIEW.md`.

## Why this is not a Bluetooth problem

All three protocol stacks need a timer, and on this part there is exactly one right answer for all of them:
Silicon Labs' `sl_sleeptimer` ships **two** backends, `sl_sleeptimer_hal_sysrtc.c` and
`sl_sleeptimer_hal_timer.c` (a 32-bit TIMER). Zigbee and Thread use the same sleeptimer component, and RAIL
uses SYSRTC over PRS to sync its own timer across EM2. So whatever is learned here is shared property:

- if the timer is written as a Bluetooth-specific shim, Zigbee would need a second driver for the same
  peripheral — the reason it lives in `efr32mg24-hal::sysrtc` instead;
- if SYSRTC stays broken, the fallback (TIMER0) has to be honest about what it costs, because the cost is
  paid by all three: no EM2/EM3, so no deep sleep.

## Offsets, and who says what (the table that cost the most)

The single most expensive fact of this exercise: **four sources disagree, and only the silicon is
authoritative.**

| Register | PAC (`efr32mg24-pac`) | Vendor header (`offsetof`) | RAIL blob (disassembly) | Measured on the board |
|---|---|---|---|---|
| `CMU.SYSRTC0CLKCTRL` | 0x1DC | **0x240** | – | **0x240 works**: CLKSEL reads back 1 with LFRCO, 3 with ULFRCO |
| `SYSRTC.EN_SET` | – | **0x1004** | – | **0x1004 works**: `EN` reads 1; 0x1000 is `IPVERSION_SET` (read-only) and leaves `EN=0` |
| `SYSRTC.CNT` | 0x18 | 0x18 | **0x18** (`RTCCSYNC_GetRtccCount` reads `[r3,#24]` plainly) | read stalls after START |
| `SYSRTC.GRP0_IF` | 0x40 | 0x40 | **0x60** (`SYSRTC_SEQ_IRQHandler` reads `[r3,#96]`) | writing 0x40..0x48 vs 0x60..0x68 changes behaviour |
| `SYSRTC.GRP0_IEN` | 0x44 | 0x44 | **0x64** (`[r3,#100]`) | – |
| `SYSRTC.SYNCBUSY` | 0x1C | 0x1C | – | read stalls after START |
| `LFRCO.CTRL` / `STATUS` | – | 0x04 / 0x08 at 0x5002_4000 | – | `STATUS = 0x10001` → **RDY=1, ENS=1** |
| Alias pages | – | +0x1000 SET, +0x2000 CLR, +0x3000 TGL | `0x500a9000`, `0x500aa000` appear in RAIL's literal pools | consistent |

How each was obtained, because the method matters more than the numbers:

- PAC: its own accessors, read from the generated source.
- Vendor header: a host C program printing `offsetof(CMU_TypeDef, …)` and `offsetof(SYSRTC_TypeDef, …)`,
  compiled against `efr32mg24_cmu.h` and `efr32mg24_sysrtc.h`.
- RAIL blob: `arm-none-eabi-objdump -d` on the archive, then reading the functions whose literal pools hold
  `0x500a8000`. `SYSRTC_SEQ_IRQHandler`, `RTCCSYNC_GetRtccCount`, `RTCCSYNC_UsecToRtccTicks` are the
  interesting ones.
- Board: the probes below.

**Rule this produces:** when a register access misbehaves, read the vendor *blob's* own offsets before
believing any header. The blob is the only code that has run against this silicon.

## What the hardware does now (measured)

```text
CLKEN0        = 0x40400000   SYSRTC0 gate (bit 30) and LFRCO gate (bit 22) both set
CLKCTRL       = 0x1 or 0x3   LFRCO selected, or ULFRCO -- both read back
LFRCO.STATUS  = 0x10001      ENS=1 RDY=1: the low-frequency oscillator IS running
SYSRTC.EN     = 0x1          enabled (after writing EN_SET at 0x1004)
CMD = START                    accepted; no fault, on both the S and NS instance
CNT read                      never returns  -> the open defect
SYNCBUSY read                 never returns  -> same event
```

Both aliases behave identically (S `0x400A8000`, NS `0x500A8000`), and the S/NS CMU views are true aliases
(writing gates through both and reading both back gives the same value).

## What is ruled out (each by a test, not an argument)

- PAC register-block layout (raw-pointer replay gives the same results).
- Secure vs non-secure instance and CMU view.
- LFRCO not running — **it is running** (RDY=1), which kills the simplest hypothesis.
- LFRCO clock select wrong — it reads back as selected.
- Init order differing from the vendor's.
- `EN` not set, or `EN_SET` written to the wrong offset (that one was real and is fixed).
- NVIC unmask with a pending compare (removed from init; no change).
- Unmapped-address behaviour (a deliberate bad address reproduces the PRECISERR + BFAR signature exactly).

## The hypotheses still standing (ranked, from `SYSRTC-OPUS-REVIEW.md`)

1. **The LF clock never reaches SYSRTC** even though the oscillator runs and CLKSEL says LFRCO — the first
   synchronised access waits in the handshake forever. Decisive test: `CLKSEL = ULFRCO` (always on).
   **Measured 2026-11-13: ULFRCO also stalls**, so if the hypothesis survives, the fault is not the source
   selection but the routing.
2. **The `CMD = START` store itself hangs**, and the marker published after it is a buffered store that had
   not reached the peripheral. Test: `dsb` immediately after the `CMD` store, then an unrelated read
   (`CMU.IPVERSION`) before any SYSRTC access.
3. Leftover state from a warm reset: disable must use `EN_CLR` (0x2004), not writing 0 to `EN_SET`.
4. Debug freeze: `CFG.DEBUGRUN = 0` means the counter stops while the core is halted — set it to 1 once and
   re-measure.
5. Missing `DSB` after the bus-clock enable (the vendor has one in `sl_sleeptimer_hal_sysrtc.c`).

Also worth knowing: the stack imports **ten** sleeptimer functions, not one — `init`, `restart_timer`,
`get_tick_count`, `get_tick_count64`, `ms_to_tick`, `ms32_to_tick`, `tick64_to_ms`, `get_timer_frequency`,
`get_clock_accuracy` (plus `start_timer`/`stop_timer` in RAIL). Implementing the HAL timer satisfies all of
them through the adapters, but they must all be consistent: `get_timer_frequency` must report the real rate
because the stack converts through `ms_to_tick`/`tick64_to_ms`.

## The decision, stated so Zigbee and Thread inherit it

- **Destination:** SYSRTC, because it is the only path to EM2/EM3 and it is what RAIL itself uses for its
  EM2 timer sync. Written once in the HAL.
- **Stopgap if SYSRTC does not yield:** TIMER0, which is what the vendor's own `sl_sleeptimer_hal_timer.c`
  backend does (32-bit TIMER, prescale 1024, ~38 kHz from 39 MHz). Cost, stated plainly: **no EM2/EM3**
  (the backend registers an EM1 requirement), a rate that is not 32768 Hz (so `get_timer_frequency` and
  `get_clock_accuracy` must tell the truth), and the callback runs in the timer ISR exactly as with SYSRTC.

Either way the API stays `efr32mg24_hal::sysrtc::Sysrtc`, so the protocol modules do not learn which
peripheral is behind it.

## The instruments (reusable, and this is the part that saved the most time)

- `crates/efr32mg24-pac/examples/pac_sysrtc_gates.rs` — cumulative log (each fact appended, whole text
  republished) so a later fault or hang cannot erase the earlier markers; a `HardFault` handler that writes
  the stacked PC, `CFSR` and `BFAR` into a **second** mailbox slot.
- `crates/efr32mg24-pac/examples/pac_sysrtc_raw.rs` — raw-pointer replay, to separate PAC bugs from
  hardware behaviour.
- `scripts/read-mailbox.sh`, plus the openocd one-liners in this file's history: `mdw 0x2003F000 4` for the
  header, `mdb 0x2003F010 180` for the text.
- The two lessons worth carrying to Zigbee: **a fault that reports is a diagnosis** (install the handler
  first, not last), and **a marker that can be overwritten is not a marker** (hence the second slot).

## If a future protocol hits this

1. Read `BFAR` — it names the register, and it is the difference between a day and an hour.
2. Compile `offsetof` against the vendor headers, and cross-check against the blob's own disassembly.
3. Re-run the cumulative probe before touching the HAL: it says which step stalls in one flash.
4. Do not trust `PAC` for a register that misbehaves; do not trust a header alone for the same reason.

## Update, same day: the counter runs

After the blob's group offsets, a `dsb` after `CMD`, and disabling through `EN_CLR`, the counter counts. Measured
on the board through the HAL:

```
rtc: ticks=287906  rate=2879kHz  en=1  run=1  deadline=289507  reached=324185  late=34678  fired=0
```

What that establishes, and what it does not:

- **It counts.** `EN=1`, `RUNNING=1`, and the counter passed a programmed deadline (`reached > deadline`), so
  the "read never returns" defect is gone. The three changes that did it were: the group block at the blob's
  offsets (0x60/0x64/0x68, not the headers' 0x40/0x44/0x48), `dsb`/`isb` after the `CMD = START` store (a
  buffered store was the reason the *next* access appeared to hang), and disabling through `EN_CLR` (0x2004).
- **The rate is not 32.768 kHz.** Measured against the DWT cycle counter it is ~2.88 MHz, about 88x the
  nominal LFRCO rate. Either the clock select is not taking effect (0x240 may not be `SYSRTC0CLKCTRL` after
  all) or the DWT/core clock assumption behind the comparison is wrong. This project already has a known
  clock discrepancy (the `delay_ms(10)` that takes 4.35 ms, `DELAY-CLOCK-DEFECT.md`), so the core-clock
  assumption is the first thing to re-measure, not the last.
- **The compare event does not fire.** `fired=0` while the counter passed the deadline, so `GRP0_CTRL` is
  either not at 0x68 or its enable bit is not bit 0. The blob only proves `IF` (0x60) and `IEN` (0x64).

Both remaining questions are cheap to answer and neither is a blocker for a *timer that polls* — only for one
that interrupts. That distinction matters for the stack: `sl_sleeptimer` needs the interrupt.

Nothing here is guessed: the three offsets and the `dsb` came from measurement, and the two open points are
stated with the observation that produced them.

## Correction, same evening: the counter runs at 32 968 Hz

The "~2.88 MHz" above was **my instrument, not the silicon**. Measured in absolute terms — the host reads
`SYSRTC0->CNT` over SWD twice, with the host's own clock between the reads, so no chip clock enters the
arithmetic:

```
CNT 12957624 -> 13309201   delta=351577 ticks in 10.66 s   rate = 32,968 Hz   (1.01x of 32,768)
```

32.968 kHz against a nominal 32.768 kHz: 0.6 %, which is LFRCO being a trimmed RC oscillator. The earlier
ratio of ~88x came from dividing by an *assumed* 19 MHz core clock through the DWT; the loop that "waited
1.9 M cycles" actually waited 8.79 s. The lesson is the book's own: when the test disagrees with the hardware,
suspect the test — and this project has a second instance of the same family in `DELAY-CLOCK-DEFECT.md`.

`scripts/read-rate.sh` is the instrument that settled it, and it stays: any future timing claim about this
board should be made in wall-clock seconds, not in cycles of a clock nobody has verified.

## Still open: the compare event does not fire

Measured with the corrected rate, a deadline 100 ms ahead, and a wraparound-safe wait:

```
cmp: start=0 deadline=3200 reached=4201 late=1001 fired=0 en=1 run=1
```

The counter passed the deadline by 1001 ticks and `GRP0_IF` bit 0 never set. So `GRP0_CTRL` at 0x68 either is
not the control register or its enable is not bit 0 — the blob only proves where `IF` (0x60) and `IEN` (0x64)
live. **This does not block the stack from booting**: `sl_sleeptimer` is currently implemented as a polled
scheduler (`crates/efr32mg24-bluetooth-sys/src/sleeptimer.rs`) with the ten functions the stack imports, driven
by `poll()` from the application loop. It blocks only interrupt-driven timers and low-power sleep.
