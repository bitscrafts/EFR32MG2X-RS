# Microphone (and IADC) notes — measured 2026-11-11

The XIAO MG24 **Sense** microphone is an **analog** MEMS part read by the IADC.
This contradicts the obvious assumption (a Sense board with a digital mic would
use PDM) and it comes straight from Seeed's own MG24 backend in
`Seeed_Arduino_Mic/src/hardware/mg24_adc.{h,cpp}`:

| fact | value |
|---|---|
| power/enable | **PC8** (`MIC_ENPIN`), driven high |
| signal | **PC9** (`MIC_INPUTPIN`), `iadcPosInputPortCPin9` |
| converter | **IADC0**, single/scan channel, not PDM |
| clock | `IADCCLK`, vendor selects FSRCO, source 20 MHz / ADC 10 MHz |
| reference | internal 1.2 V (`iadcCfgReferenceInt1V2`) |
| oversampling | 2x high speed (`iadcCfgOsrHighSpeed2x`), gain 1x |
| data | 12-bit right aligned, FIFO valid level 2, DMA wakeup |
| analog bus | `GPIO->CDBUSALLOC |= GPIO_CDBUSALLOC_CDODD0_ADC0` |

## What the IADC demands, and what each mistake looked like

1. **`CTRL.TIMEBASE` must not be zero.** With 0 the converter produces nothing:
   `STATUS` reads 0, `SINGLEFIFODV` never sets, and a polling loop spins forever.
   Correct value is `ceil(fsrc / 1 MHz) - 1` → 19 for a 20 MHz source.
2. **Configuration writes are dropped while the synchroniser is busy.** `CTRL`
   and `CFG0` landed while `SINGLE` and `SINGLEFIFOCFG` read back as 0 (the
   reset values). The vendor's `IADC_disable()` waits for `STATUS.SYNCBUSY` and
   `EN.DISABLING` — that wait is mandatory, not defensive.
3. **`SINGLE` has its own write handshake**: `STATUS.SINGLEWRITEPENDING` must
   clear, and `MASKREQWRITEPENDING` for `MASKREQ`.
4. **The scan path needs a trigger.** With the scan configured and no trigger
   source the converter sat at `CONVERTING=1` and `SCANQUEUEPENDING=1` forever
   and never filled the FIFO. Single conversions polled with `SINGLESTART` need
   no trigger, which is what the examples use.
5. **A converter stuck in `EN.DISABLING` survives a core reset.** `EN` then reads
   `0x3` (enabled *and* disabling), and the next write to `SINGLE` is a precise
   bus fault: `CFSR=0x8200`, `BFAR=0x49004098`. Only a **power cycle** clears it.
   Both examples therefore check `EN.DISABLING` before touching any register and
   report it instead of faulting.

## The mistake that cost the most, and the fix

**`SINGLE` can only be written while the converter is ENABLED.** The vendor says
it in a comment — *"IADCn->SINGLE has WSYNC type and can only be written while
enabled"* — and `IADC_updateSingleInput()` enables the block, writes `SINGLE`,
then restores the previous state.

Writing it with `EN=0` is a precise bus fault (`CFSR=0x8200`,
`BFAR=0x49004098`) that leaves the converter in `EN.DISABLING`, and that state
**survives a core reset**. The order that works:

```
EN=1 -> stop conversions -> MASKREQ=0, SINGLE=0 (enabled) -> drain FIFO
EN=0 -> CTRL, CFG0, SCHED0, SINGLEFIFOCFG
EN=1 -> SINGLE = channel config -> SINGLEWRITEPENDING clear -> start
```

## Verified

```
pac_mic_vu  mic rst=ok rms=562 pk=598 n=128
hal_mic_vu  hal mic rms=521 pk=544 n=128
```

128 samples per window from a real microphone, same noise floor through both
layers (the ~7% difference is run-to-run room noise, not configuration).

**Settling matters for the first reading**: reporting the very first window gave
RMS 98; the same chain measured ~550 a few hundred milliseconds later. Both
examples now wait before the first measurement.

## Why HFRCOEM23 and not FSRCO

The vendor selects FSRCO for `IADCCLK`. With this board already in the wedged
state it was impossible to tell whether FSRCO was the cause, so the HAL points
`IADCCLK` at **HFRCOEM23** — the oscillator already proven to clock TIMER0 here —
forcing it on and waiting for `RDY` before selecting it. If the converter comes up
with that clock, the difference is irrelevant to the application; if it does not,
the FSRCO path is the next thing to test.
