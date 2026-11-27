# rtc

Real-Time Clock (SYSRTC0) Module

## What it is

Real-Time Clock (SYSRTC0) Module

This module provides a Hardware Abstraction Layer (HAL) for the EFR32MG24's
System Real-Time Clock (SYSRTC0), a 32-bit counter for timekeeping and alarms.

# Features

- **32-bit Counter**: Up to 136 years at 32.768 kHz
- **Low Power**: Runs in EM0-EM3 energy modes
- **Compare Match (Alarm)**: Single alarm with interrupt capability
- **LFXO/LFRCO Clock**: High-accuracy crystal or RC oscillator
- **Simple API**: Counter-based timekeeping (not full calendar)

# Current Implementation (C4-RTC Stage 1)

- ✅ Counter read/write
- ✅ Single compare match (alarm)
- ✅ Interrupt flag checking
- ✅ LFXO clock source
- ✅ Helper functions (counter ↔ seconds)

# Not Implemented (Future)

- ⏳ Full calendar mode (date/time structures)
- ⏳ Multiple alarms (CMP1)
- ⏳ Capture mode
- ⏳ Output actions

# Usage

```rust,no_run
use efr32mg2x_hal::{rtc::Rtc, pac};

let dp = pac::Peripherals::take().unwrap();

// Initialize RTC with LFXO (requires LFXO to be enabled in CMU)
let mut rtc = Rtc::new(dp.sysrtc0_ns);

// Set current time to 0 (reset counter)
rtc.set_counter(0);

// Set alarm for 10 seconds from now
let current = rtc.counter();
let alarm_time = current + rtc.seconds_to_ticks(10);
rtc.set_alarm(alarm_time).expect("Failed to set alarm");

// Wait for alarm
while !rtc.alarm_flag() {
// Could enter sleep mode here
}

rtc.clear_alarm_flag();
```

# Clock Source

The RTC requires a low-frequency clock (LFCLK) to be configured in the CMU.
Typically this is LFXO (32.768 kHz crystal) for accuracy.

```rust,no_run
// Configure LFXO before initializing RTC
let (clocks, cmu) = Clocks::new(
dp.cmu_s,
ClockConfig {
hfxo: None,
lfxo: Some(LfxoConfig::default()),  // Enable LFXO for RTC
},
).expect("Clock config failed");
```

## API at a glance

| item | kind | what it does |
|---|---|---|
| `ADDR_DEFAULT` | const | Default 7-bit address of the PCF8563. |
| `ClockSource` | enum | — |
| `DateTime` | struct | — |
| `Error` | enum | — |
| `Pcf8563` | struct | A PCF8563 on some I2C bus. |
| `Rtc` | struct | RTC (System Real-Time Clock) Owns the SYSRTC0 peripheral and provides timekeeping functionality. |
| `RtcError` | enum | — |
| `Seconds` | type | RTC Time in seconds Represents time as a simple seconds counter. |
| `alarm_flag` | fn | Checks if the alarm flag is set Returns true if the counter has reached the compare match 0 value. |
| `clear_alarm_flag` | fn | Clears the alarm flag Clears the compare match 0 interrupt flag. |
| `clock_valid` | fn | True when the chip says its contents survived a power loss. |
| `counter` | fn | Reads the current counter value Returns the 32-bit counter value in ticks (32.768 kHz with LFXO). |
| `disable_alarm` | fn | Disables the alarm Disables compare match 0, preventing alarm triggers. |
| `free` | fn | Give the bus back. |
| `millis_to_ticks` | fn | Converts milliseconds to ticks Converts milliseconds to RTC counter ticks. |
| `new` | fn | Creates a new RTC instance Takes ownership of the SYSRTC0 peripheral and initializes it with LFXO clock. |
| `read` | fn | Read the whole calendar in one transaction. |
| `seconds_to_ticks` | fn | Converts seconds to ticks Converts seconds to RTC counter ticks, assuming 32.768 kHz LFXO. |
| `set` | fn | Set the calendar. |
| `set_alarm` | fn | Sets the alarm (compare match 0) value Configures the compare match 0 register to trigger an interrupt flag when the counter reaches the specified value. |
| `set_counter` | fn | Sets the counter value Writes a new value to the 32-bit counter. |
| `ticks_to_millis` | fn | Converts ticks to milliseconds Converts RTC counter ticks to milliseconds. |
| `ticks_to_seconds` | fn | Converts ticks to seconds Converts RTC counter ticks to seconds, assuming 32.768 kHz LFXO. |
| `with_address` | fn | Bind the driver to a chip strapped at another address. |

## Registers it touches

`CmuS`, `Peripherals`, `Sysrtc0Ns`

## What was measured

```
    burtc: ok=1 run=1 pre=0 d0=32901 s0=32901 r0=1000 pd=3 d1=4113 s1=32901 r1=125 ctl=0 cs=32902 bk=16448 bs=16451 rb=999 if=0x00000000 st=0x00000001 sb=0x00000000 ck=0x00000101
    burtc: ok=1 run=1 pre=0 d0=32900 s0=32900 r0=1000 pd=3 d1=4112 s1=32899 r1=124 ctl=0 cs=32899 bk=16446 bs=16450 rb=999 if=0x00000000 st=0x00000001 sb=0x00000000 ck=0x00000101
    probe: t=1865376137 c=29605      SYSRTC and DWT at the first read
    cmp: start=…57 deadline=…77 reached=…78 late=1 fired=1 en=1 run=1      PASS   SYSRTC compare fires once, late=1
```

Controls and the full story: [BURTC-2026-11-27.md](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/BURTC-2026-11-27.md), [DWT-NEEDS-A-DEBUG-SESSION.md](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/DWT-NEEDS-A-DEBUG-SESSION.md), [HAL-REVIEW-BRIEF-2026-11-25.md](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/HAL-REVIEW-BRIEF-2026-11-25.md), [LETIMER-TOP-2026-11-26.md](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/LETIMER-TOP-2026-11-26.md)

## What is not proven

- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))
- ~~H28~~ | EM2 entry and exit sequence | **CLOSED 2026-11-21** (`B3`): ten ~1 s naps in EM2 woken by the SYSRTC comparator, intervals 0.92-1.03 s, mean 0.99, total 9.92 s, eleven interrupts. (HAL-GAP-ANALYSIS-2026-11-18.md (`../../../../docs/04-reference/HAL-GAP-ANALYSIS-2026-11-18`))

## Using it

No example imports this module yet, so its API has never been exercised from `examples/`.
