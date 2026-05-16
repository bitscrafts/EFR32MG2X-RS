//! Simple blink application to test custom bootloader
//!
//! This application will be loaded at 0x08004000 by the custom bootloader
//! It should blink the LED at 1 Hz
//!
//! Build with:
//! ```bash
//! cargo build --example test-blink-app --release
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

/// Simple delay function (busy-wait)
fn delay_ms(ms: u32) {
    // Rough estimate: ~39 MHz / 4 cycles per loop = ~10 NOPs per microsecond
    let cycles = ms * 10000;
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    unsafe {
        // Initialize peripherals
        // Note: CMU might already be unlocked by bootloader, but we'll do it anyway

        // Unlock CMU
        const CMU_LOCK: *mut u32 = 0x40008010 as *mut u32;
        core::ptr::write_volatile(CMU_LOCK, 0x93F7);

        // Enable GPIO clock (bit 26)
        const CMU_CLKEN0: *mut u32 = 0x40008064 as *mut u32;
        core::ptr::write_volatile(CMU_CLKEN0, 1 << 26);

        // Configure PA7 as push-pull output
        const GPIO_PORTA_MODEL: *mut u32 = 0x5003C034 as *mut u32;
        let model = core::ptr::read_volatile(GPIO_PORTA_MODEL);
        core::ptr::write_volatile(GPIO_PORTA_MODEL, (model & !0xF0000000) | 0x40000000);

        // Blink LED at 1 Hz
        const GPIO_PORTA_DOUT: *mut u32 = 0x5003C03C as *mut u32;

        loop {
            // LED ON (active-low: write 0)
            core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);
            delay_ms(500);

            // LED OFF (active-low: write 1 to bit 7)
            core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x80);
            delay_ms(500);
        }
    }
}
