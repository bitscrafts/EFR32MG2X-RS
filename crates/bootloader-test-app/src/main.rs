//! Simple blink application to test custom bootloader
//!
//! This application will be loaded at 0x08004000 by the custom bootloader.
//! It should blink the LED at 1 Hz (500ms ON, 500ms OFF).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use efr32mg24_pac as pac;

/// Simple delay function (busy-wait)
/// Configured for 19 MHz HFRCO (default after reset)
/// ~19 MHz / 4 cycles per iteration = ~4750 iterations per ms
#[inline(never)]
fn delay_ms(ms: u32) {
    let cycles = ms * 4750;
    for _ in 0..cycles {
        unsafe {
            core::arch::asm!("nop", options(nomem, nostack));
        }
    }
}

#[entry]
fn main() -> ! {
    // CRITICAL: Disable watchdog FIRST!
    let peripherals = pac::Peripherals::take().unwrap();

    // Disable WDOG0 (the Gemini document confirms this is THE fix!)
    peripherals.wdog0_ns.en().write(|w| w.en().clear_bit());

    unsafe {
        // Unlock CMU
        const CMU_LOCK: *mut u32 = 0x40008010 as *mut u32;
        core::ptr::write_volatile(CMU_LOCK, 0x93F7);

        // Enable GPIO clock (bit 26)
        const CMU_CLKEN0: *mut u32 = 0x40008064 as *mut u32;
        core::ptr::write_volatile(CMU_CLKEN0, 1 << 26);

        // Configure PA7 as push-pull output (MODE = 0x4)
        const GPIO_PORTA_MODEL: *mut u32 = 0x5003C034 as *mut u32;
        let model = core::ptr::read_volatile(GPIO_PORTA_MODEL);
        core::ptr::write_volatile(GPIO_PORTA_MODEL, (model & !0xF0000000) | 0x40000000);

        const GPIO_PORTA_DOUT: *mut u32 = 0x5003C03C as *mut u32;

        // Blink LED at 1 Hz (active-low: 0=ON, 1=OFF)
        loop {
            core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);  // LED ON (bit 7 = 0)
            delay_ms(500);

            core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x80);  // LED OFF (bit 7 = 1)
            delay_ms(500);
        }
    }
}
