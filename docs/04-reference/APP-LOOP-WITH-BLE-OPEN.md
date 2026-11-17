# The loop with the Bluetooth stack running: 206 % of real time, unexplained as of 2026-11-17

## What works

`crates/xiao-mg24-bsp/examples/module_ble_dosimeter.rs` publishes the dosimeter's service and runs the capture:

```
ble: seconds=5 windows=315 published=5 notifies=0 timer=1 rms=68 b0=89423434   status=PASS

315 windows = 63 a second, exactly five seconds of audio
published = 5 one update per second written into the characteristics
notifies = 0 no phone was connected — a fact, not a failure
rms and the band powers are real values from the microphone
```

The service is `e1d0e000-…` with `rms` (u16), `bands` (u32×8) and `status` (u8 flags + u16 window count). No dB and no
dose percent, because there is no acoustic calibration on this board.

## What does not

**The same 5 seconds of audio take 195,8 M core cycles, which is 10,3 s at 19 MHz.** Without the stack the
interleaved loop takes 99 % of real time (`docs/04-reference/APP-LOOP-BUDGET.md`), so the stack costs a factor of
about two, and it is not the notifications: there are three per second.

Three things were tried, and none changed the number:

```
move the pacing after Bluetooth::new          195 870 529 cycles   (was 195 852 591)
move the microphone AFTER the stack           195 868 053 cycles
shorten read_paced's miss budget 200k -> 2,5k 195 873 824 cycles
self-calibrating period (one window measured) 195 892 558 cycles
```

So the cost is not in the pacing setup, not in the miss budget, and not a simple clock mismatch that a period
correction fixes.

## The two candidates left, and how to tell them apart

```
1. the stack keeps ~50 % of the CPU busy (advertising, event loop, link-layer timing) and the paced reads
   simply take longer in wall time
   test: run the loop with the stack started but advertising OFF, and with advertising ON. If the number
   halves when advertising is off, it is load.

2. the stack changes the IADC's clock so a window is 32 ms of audio instead of 16, and 315 windows cover twice
   the time
   test: read CMU_IADCCLKCTRL and SCHED0.PRESCALE after the stack starts, and compute CLK_SRC_ADC. If it is
   9,5 MHz rather than 19, the period must be 594, not 1187. The self-calibration attempted above should have
   covered this and did not, which makes candidate 1 more likely — unless the first window used for calibration
   was itself correct and the slowdown came afterwards.
```

## What is not in doubt

The data path works end to end: microphone paced, bands computed inside the wait, values written into the GATT
characteristics every second, for five seconds, with the radio stack running. What remains is a budget question,
not a correctness one — and it is the one place where the DPLL (38 / 56 / 80 MHz, all measured) is genuinely
needed rather than a convenience.
