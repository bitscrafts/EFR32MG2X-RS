//! Blink example using PAC peripherals (v2 - using PAC API)
//!
//! Uses PAC's peripheral access for correct register addresses

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

// Provide critical section implementation for single-core Cortex-M
use cortex_m as _;

#[entry]
fn main() -> ! {
    // Get peripheral instances
    let dp = pac::Peripherals::take().unwrap();

    // Enable GPIO clock via CMU
    dp.cmu_s.clken0().modify(|_, w| w.gpio().set_bit());

    // Configure PA7 as push-pull output
    // PA7 is in MODEL register (pins 0-7)
    dp.gpio_s.porta_model().modify(|_, w| unsafe {
        w.mode7().bits(0x4)  // Mode 4 = push-pull
    });

    // Blink loop
    let mut state = false;
    loop {
        // Toggle PA7 using DOUT register
        if state {
            dp.gpio_s.porta_dout().modify(|r, w| unsafe {
                w.bits(r.bits() | (1 << 7))  // Set bit 7
            });
        } else {
            dp.gpio_s.porta_dout().modify(|r, w| unsafe {
                w.bits(r.bits() & !(1 << 7))  // Clear bit 7
            });
        }
        state = !state;

        // Delay loop
        cortex_m::asm::delay(10_000_000);  // Longer delay
    }
}
