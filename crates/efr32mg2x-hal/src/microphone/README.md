# microphone

Microphone: the analog MEMS mic on the XIAO MG24 **Sense**, sampled by IADC0.

## What it is

Microphone: the analog MEMS mic on the XIAO MG24 **Sense**, sampled by IADC0.

## Hardware facts, from Seeed's own MG24 backend

`Seeed_Arduino_Mic/src/hardware/mg24_adc.{h,cpp}` is unambiguous: the mic is
**not** a PDM device on this module. It is an analog microphone on **PC9**,
powered through **PC8**, sampled by the **IADC** with `IADCCLK` from an
internal oscillator, 1.2 V reference and 12-bit right-aligned results.

## What the IADC demands (measured, each one cost a flash)

1. `CTRL.TIMEBASE` must be non-zero — `ceil(fsrc / 1 MHz) - 1`. With 0 the
converter produces nothing at all and every status bit stays clear.
2. Configuration writes are dropped while `STATUS.SYNCBUSY` or
`EN.DISABLING` is set. `CTRL` can land while `SINGLE` silently does not.
3. `SINGLE` has its own write handshake: wait for `STATUS.SINGLEWRITEPENDING`.
4. A converter left with `EN.DISABLING` set **survives a core reset** and
faults on the next `SINGLE` write (`BFAR = 0x49004098`). Only a power
cycle clears it, so every entry point here refuses to write in that state
and reports it instead of faulting.

## Why single conversions

The vendor driver scans into a FIFO and lets the LDMA drain it. A scan needs
a trigger source configured; with the scan path and no trigger the converter
sat with `CONVERTING=1` and `SCANQUEUEPENDING=1` forever. One channel polled
with `SINGLESTART` needs no trigger, which is the right starting point for a
level meter. The DMA path belongs later, next to the streaming work.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Converter` | struct | — |
| `Level` | struct | — |
| `MAX_WINDOW` | const | Largest window `level()` will measure, and the buffer it needs: 128 samples is 256 bytes of stack, and it is two windows of the dosimeter's 63-sample second. |
| `MicError` | enum | — |
| `Microphone` | struct | A configured single-channel microphone capture. |
| `arm` | fn | Arm the single queue once, so a trigger has a queue to start. |
| `capture_window` | fn | Fill a whole window with paced samples, returning how many actually arrived. |
| `isqrt` | fn | Integer square root (no libm on a bare-metal target). |
| `level` | fn | RMS and peak amplitude over `window` samples, about the window's **own mean**. |
| `microphone` | fn | Start a microphone on this converter. |
| `overflows` | fn | How many FIFO overflows this converter has reported since boot. |
| `pace_from_timer` | fn | Configure the single queue to trigger from the IADC's local timer, and load the period. |
| `read_paced` | fn | Wait for the conversion the timer started, and read it — without issuing a start of its own. |
| `retune_period` | fn | Set the timer period again, without touching the trigger configuration. |
| `sample` | fn | One conversion, or `None` if the converter did not deliver. |
| `start_timer` | fn | Start the local timer. |
| `timer_active` | fn | — |

## Registers it touches

`Iadc0S`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

- ⏳ Continuous conversion and multi-channel scan (deferred; the microphone path uses LDMA instead). (PLAN.md (`../../../../docs/03-development/PLAN`))

## Using it

No example imports this module yet, so its API has never been exercised from `examples/`.
