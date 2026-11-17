# IADC timer trigger — the documentation, quoted, for the next analysis

Collected 2026-11-17. Every quote below is verbatim from a file in this repository or from a vendor page, with
the source beside it. Nothing here is inferred; where a source could not be obtained it says so instead of
being paraphrased.

## 1. What `SINGLETRIGSEL = Timer` means

The field description as shipped in the device SVD and rendered by the PAC:

```
crates/efr32mg2x-pac/src/iadc0_s/trigger.rs
  #[doc = "Single Trigger Select\n\nValue on reset: 0"]
  pub enum Singletrigsel {
      #[doc = "0: Immediate triggering. The single queue will be disabled once the conversion is
               complete, unless TRIGGERACTION is set to continuous."]
      Immediate = 0,
      #[doc = "1: Triggers when the local timer count reaches zero."]
      Timer = 1,
      #[doc = "2: Triggers on PRS0 from a timer module that is using the same clock group as the ADC
               and has been programmed to use the same clock source as the ADC. The prescale may be
               different between the timer module."]
      Prsclkgrp = 2,
      #[doc = "3: Triggers on asynchronous PRS0 positive edge. ..."]
      Prspos = 3,
```

The vendor's own enumeration in the emlib header, which is the same values with the SDK's names:

```
crates/efr32mg24-emlib-sys/vendor/emlib/inc/em_iadc.h:715-727
  /** IADC trigger action */
  typedef enum {
    /** Start single/scan queue immediately */
    iadcTriggerSelImmediate  = _IADC_TRIGGER_SCANTRIGSEL_IMMEDIATE,
    /** Timer starts single/scan queue */
    iadcTriggerSelTimer  = _IADC_TRIGGER_SCANTRIGSEL_TIMER,
    /** PRS0 from timer in same clock group starts single/scan queue  */
    iadcTriggerSelPrs0SameClk  = _IADC_TRIGGER_SCANTRIGSEL_PRSCLKGRP,
    /** PRS0 positive edge starts single/scan queue  */
    iadcTriggerSelPrs0PosEdge  = _IADC_TRIGGER_SCANTRIGSEL_PRSPOS,
```

Reading them together: the trigger is described as **starting the queue**, and the timer variant keys on the
**local** timer count reaching zero. That is the IADC's own counter, not TIMER0 — and the queue has to exist
(be armed) before a trigger can start it. Both facts came out of the silicon the hard way (see
`MICROPHONE-PACING-OPEN.md`), and both are written here so the next attempt does not rediscover them.

## 2. The local timer's register

```
crates/efr32mg2x-pac/src/iadc0_s/timer.rs
  #[doc = "Register `TIMER` reader"] / #[doc = "Register `TIMER` writer"]
  #[doc = "Bits 0:15 - Timer Period"]
```

Sixteen bits, described only as "Timer Period". The unit is not stated in the SVD. The working assumption to
test is ADC clock cycles — which is why the ADC clock has to be established first (see §4).

## 3. Arming and enabling the timer

```
crates/efr32mg24-emlib-sys/vendor/device/efr32mg24_iadc.h:311-316
  #define IADC_CMD_TIMEREN     (0x1UL << 16)   /**< Timer Enable  */
  #define IADC_CMD_TIMEREN_SHIFT  16
  #define IADC_CMD_TIMEREN_MASK   0x10000UL
  #define IADC_CMD_TIMERDIS    (0x1UL << 17)   /**< Timer Disable */
```

`CMD.TIMEREN` enables the timer; the register block also carries `TIMER`, `TIMER_SET`, `TIMER_CLR` and
`TIMER_TGL` aliases (header lines 63, 95, 127, 159), so the period can be written without a read-modify-write.

## 4. The clock the ADC actually runs at

```
AN0004.2 (EFR32 Series 2 CMU), §2.3.3, Rev 1.2, p.17
  "The HFRCO defaults to operation in the 19 MHz band but can be switched to operate in one of the other
   preset frequency bands by changing the FREQRANGE bit field in the HFRCO_CAL."

  HFRCODPLL bands: 1, 2, 4, 5, 7, 10, 13, 16, 19, 20, 26, 32, 38, 48, 56, 64, 80 MHz
```

Source: <https://www.silabs.com/documents/public/application-notes/an0004.2-efr32-series2-cmu.pdf>

What this does **not** say is the IADC's own divide: `CTRL.PRESC` and the high-speed clock rate select a
divider between HFRCOEM23 and `ADC_CLK`. The local timer counts in that divided clock, so the period for
16 kHz is `ADC_CLK / 16 000`, not `19 000 000 / 16 000`, until `CTRL` is read back and the divisor confirmed.

## 5. The part's maximum clock, for the record

```
Seeed wiki, https://wiki.seeedstudio.com/xiao_mg24_getting_started/
  features:  "ARM Cortex-M33 core, with a maximum clock speed of 78MHz"
  spec table: "Processor | Silicon Labs EFR32MG24 ARM Cortex-M33 @ 78MHz"
```

78 MHz is the ceiling; 19 MHz is what the part runs at after reset (AN0004.2 §2.3.3). Reaching 78 requires
programming the DPLL, which this repository does not do yet — `efr32mg2x_hal::clock` has HFXO and no DPLL.

## 6. What could not be obtained, and where to get it

```
EFR32xG24 Reference Manual (PDF)   https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf
  attempt 2026-11-17: HTTP 403 "Access Denied" from this host (curl and the browser backend both blocked;
  no display for the browser). The IADC chapter's prose about the local timer is the missing piece —
  specifically whether TIMER counts ADC_CLK and whether the queue must be armed before TIMEREN.
  Fetch it from a browser session, or from the DFP pack's HTML docs if the pack is unpacked locally.
```

Until that chapter is read, the pacing work stays where `MICROPHONE-PACING-OPEN.md` leaves it: the timer
runs, the converter converts, and the two are not yet connected by evidence.
