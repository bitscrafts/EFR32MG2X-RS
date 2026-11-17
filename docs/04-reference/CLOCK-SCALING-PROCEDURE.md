# Running the core faster — the vendor's own path, quoted, and the procedure before any flash

Derived 2026-11-17 from sources that are in this repository. Nothing here is inferred: each step has the file
and line, or the manual page, beside it.

## STATUS 2026-11-17 — implemented and measured; no longer a requirement

```
19 MHz  baseline        304 186 cycles per 16 ms window
38 MHz  ratio 1,99      cal 0xc8cc9d43
56 MHz  ratio 2,93      cal 0xdceea141    ("times three" lands here; 57 is not a band)
80 MHz  ratio 4,19      cal 0xdcf09d41    (the ceiling under 100 MHz)
```

Implemented as `Board::boost_to_38_mhz`, `boost_to_56_mhz`, `boost_to_80_mhz`, all through one shared
`apply_hfrco_cal`. And the application loop now fits at 19 MHz by interleaving the features with the capture
(99 % of real time, `docs/04-reference/APP-LOOP-BUDGET.md`), so the clock is **margin**, not a prerequisite.

## Why this was the next task at the time

`docs/04-reference/APP-LOOP-BUDGET.md` measured the application loop at **108 % of real time** at 19 MHz: 63
windows of 16 ms of audio took 20,7 M cycles instead of 19,15 M. The loop is 15 % over before the model, the
dose and the radio are added. Faster clock is the only option that keeps 62 feature windows a second.

## What the part actually offers

```
em_cmu.h, CMU_HFRCODPLLFreq_TypeDef:
  1, 2, 4, 7, 13, 16, 19, 26, 32, 38, 48, 56, 64, 80, 100 MHz

There is NO 78 MHz band. The 78 MHz in Seeed's table is the Cortex-M33's rated maximum for this part; the
supported frequencies are the bands above, and the closest at or under it is 80 MHz. Anything else would be a
number nobody calibrated.
```

## The four pieces, and where the vendor keeps them

```
1. core voltage      EMU VSCALE. Higher frequency needs the higher performance scale.
                     em_cmu.c:104-108  VSCALE_DEFAULT = VSCALE_EM01_HIGH_PERFORMANCE
                     this HAL already has it: efr32mg2x_hal::emu::Emu::set_em01_voltage_scale

2. flash wait states MSC_READCTRL.MODE (Ws0..Ws3).
                     RM p57 §6.3.9: "MODE should be changed after the clock is changed when reducing clock
                     speed and BEFORE the clock is changed when increasing clock speed."
                     em_cmu.c: CMU_UpdateWaitStates(freq, VSCALE) — the wait states depend on frequency AND
                     on the voltage scale, which is why step 1 comes first.

3. the frequency     HFRCO0 gets the factory calibration for the chosen band:
                     DEVINFO->HFRCODPLLCAL[n].HFRCODPLLCAL, with the band -> index mapping in
                     em_cmu.c:4991-5035:
                        1/2/4 MHz -> [0]     7 -> [3]     13 -> [6]    16 -> [7]    19 -> [8]
                        26 -> [10]           32 -> [11]   38 -> [12]   48 -> [13]   56 -> [14]
                        64 -> [15]           80 -> [16]   100 -> [17]

4. no SYSCLK switch  CMU_SYSCLKCTRL.CLKSEL accepts only Fsrco(1), Hfrcodpll(2), Hfxo(3) — there is no "Dpll"
                     option, and the core is ALREADY on Hfrcodpll at its 19 MHz band. Raising the band raises
                     the core clock in place. This is why the bricking incident's HFXO sequence is not needed
                     here at all: nothing gets switched.
```

## The procedure, in the order that fails safe

```
0. record the intent in the mailbox BEFORE touching anything, so a hang leaves a trace with a timestamp
1. VSCALE to high performance, wait for the EMU to report it
2. MSC_READCTRL.MODE to the setting for the TARGET frequency (over-setting only slows the flash; under-setting
   faults, so when in doubt go slower)
3. DMEM wait states if the target is above the documented threshold (RM p124: "Must be set to 1 if the clock
   frequency is above N MHz"). The datasheet's table did not extract to searchable text, so the field is set
   from the RM's own wording and the value is read back and reported rather than assumed.
4. write the DEVINFO calibration for the chosen band into HFRCO0.CAL (TUNING + FINETUNING) and set FREQRANGE
5. wait for HFRCO0.STATUS.RDY
6. read back what the CMU says the core is running at, and write that to the mailbox
7. measure: the same application loop, `pct_of_realtime` must fall to roughly 108 % x (19 / f_new)
```

## The numbers to beat, per band

```
19 MHz   pct_of_realtime = 108      (measured, today)
38 MHz   expect ~54                 the safe doubling: one band, one VSCALE step
80 MHz   expect ~26                 four bands up; the one that gives room for model + BLE + stack
```

## Safety, because the board has been bricked once already

- Recovery path exists: SWD does not depend on SYSCLK, so a wrong clock can be undone by flashing again with
  `scripts/flash.sh --confirm` (which reads back what it wrote).
- Never flash a clock change without the mailbox record from step 0: a silent hang is the one outcome this
  project has paid for repeatedly.
- Do the 38 MHz step first and measure. 80 MHz is another doubling of the same procedure, and if 38 does not
  produce the expected ratio, the mistake is in the procedure and not in the band.
