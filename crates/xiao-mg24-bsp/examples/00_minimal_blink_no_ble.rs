//! Minimal Blink (No BLE Dependencies)
//!
//! Ultra-minimal blink example that avoids bluetooth-sys dependency.
//! Uses only HAL and BSP. Blinks the onboard LED on PA7 (active low).
//!
//! # Hardware
//!
//! - XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//! - LED on PA7 (Arduino pin 20, `LED_BUILTIN`, active low)
//!
//! # Flashing
//!
//! ```bash
//! cd xiao-mg24-bsp
//! cargo build --release --example 00_minimal_blink_no_ble --features rt
//! probe-rs run --chip EFR32MG24B220F1536IM48 \
//!   target/thumbv8m.main-none-eabihf/release/examples/00_minimal_blink_no_ble
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::delay::Delay;
use efr32mg24_hal::gpio::GpioExt;
use efr32mg24_hal::pac;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Take PAC peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks: HFXO @ 39 MHz, LFXO @ 32.768 kHz
    let (clocks, cmu) = efr32mg24_hal::clock::Clocks::new(
        dp.cmu_s,
        efr32mg24_hal::clock::ClockConfig {
            hfxo: Some(efr32mg24_hal::clock::HfxoConfig::new(39_000_000)),
            lfxo: Some(efr32mg24_hal::clock::LfxoConfig::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Initialize delay
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // Initialize GPIO
    let gpio = dp.gpio_s.split(&frozen_clocks);

    // LED on PA7 (built-in user LED, active low, NOT on XIAO connector)
    let mut led = gpio.porta.pa7.into_push_pull_output();

    // Main blink loop
    loop {
        led.set_low().unwrap(); // Active low: low = ON
        delay.delay_ms(500);
        led.set_high().unwrap(); // High = OFF
        delay.delay_ms(500);
    }
}
