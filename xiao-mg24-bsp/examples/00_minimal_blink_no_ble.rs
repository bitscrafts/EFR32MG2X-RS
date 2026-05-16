//! Minimal Blink (No BLE Dependencies)
//!
//! Ultra-minimal blink example that avoids bluetooth-sys dependency.
//! Uses only HAL and PAC.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::gpio::{GpioExt, Output, PushPull};
use efr32mg24_hal::delay::DelayExt;
use efr32mg24_pac as pac;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Take PAC peripherals
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Initialize clock
    let clocks = efr32mg24_hal::clock::Clocks::new(dp.CMU_S);

    // Initialize delay
    let mut delay = cp.SYST.delay(&clocks);

    // Initialize GPIO
    let gpio = dp.GPIO_S.split();

    // LED on PD02 (D8 on XIAO)
    let mut led = gpio.pd2.into_push_pull_output();

    // Debug trap with LED feedback
    #[cfg(debug_assertions)]
    {
        for _ in 0..10 {
            let _ = led.set_high();
            delay.delay_ms(250);
            let _ = led.set_low();
            delay.delay_ms(250);
        }
    }

    // Main blink loop
    loop {
        let _ = led.set_high();
        delay.delay_ms(500);
        let _ = led.set_low();
        delay.delay_ms(500);
    }
}
