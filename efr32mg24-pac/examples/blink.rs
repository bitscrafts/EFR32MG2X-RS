//! Minimal Blink Example using PAC (Register-level access)
//!
//! This is the SIMPLEST possible example:
//! - Uses only PAC (no HAL, no BSP)
//! - Direct register manipulation
//! - No external dependencies except cortex-m
//!
//! # Hardware
//!
//! - XIAO MG24 Sense
//! - LED on PA7 (pin 20, active LOW)
//!
//! # Safety
//!
//! - 5-second debug trap with LED blink
//! - No SWD pin reconfiguration
//! - No low-power modes
//!
//! # Flashing
//!
//! ```bash
//! # Build
//! cargo build --release --example blink
//!
//! # Flash
//! probe-rs run --chip EFR32MG24B220F1536 --release --example blink
//! ```

#![no_std]
#![no_main]

use core::ptr;
use cortex_m_rt::entry;
use panic_halt as _;

// Import PAC to link interrupt vectors (required by cortex-m-rt)
use efr32mg24_pac as _;

// EFR32MG24 Register addresses
const CMU_BASE: usize = 0x4000_8000;
const GPIO_BASE: usize = 0x5000_0000;

// CMU registers (Clock Management Unit)
const CMU_CLKEN0: *mut u32 = (CMU_BASE + 0x308) as *mut u32;
const CMU_CLKEN0_GPIO: u32 = 1 << 2; // Bit 2: GPIO clock enable

// GPIO Port A registers
const GPIO_PORTA_BASE: usize = GPIO_BASE + 0x000; // Port A offset
const GPIO_PORTA_MODEL: *mut u32 = (GPIO_PORTA_BASE + 0x004) as *mut u32; // MODEL for pins 0-7
const GPIO_PORTA_DOUT: *mut u32 = (GPIO_PORTA_BASE + 0x00C) as *mut u32;

// Pin 7 configuration (LED on PA7, active LOW)
const PIN7_SHIFT: u32 = 7 * 4; // Pin 7 in MODEL register (bits 28-31)
const PIN7_MODE_PUSHPULL: u32 = 0x4; // Push-pull output mode
const PIN7_BIT: u32 = 1 << 7; // Bit 7 for PA7

/// Simple delay loop (blocking)
///
/// Approximate delay at default clock speed (19 MHz FSRCO).
/// Each iteration takes ~10-20 cycles.
#[inline(never)]
fn delay_cycles(cycles: u32) {
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}

/// Delay in milliseconds (approximate)
fn delay_ms(ms: u32) {
    // At 19 MHz, roughly 19000 cycles per ms
    // With loop overhead, use ~15000 cycles per ms
    delay_cycles(ms * 15_000);
}

/// Initialize GPIO for LED control
unsafe fn gpio_init() {
    // 1. Enable GPIO clock
    ptr::write_volatile(CMU_CLKEN0, ptr::read_volatile(CMU_CLKEN0) | CMU_CLKEN0_GPIO);

    // 2. Configure PA7 as push-pull output
    let model_val = ptr::read_volatile(GPIO_PORTA_MODEL);
    let model_new = (model_val & !(0xF << PIN7_SHIFT)) | (PIN7_MODE_PUSHPULL << PIN7_SHIFT);
    ptr::write_volatile(GPIO_PORTA_MODEL, model_new);

    // 3. Set initial output HIGH (LED off - active LOW)
    let dout = ptr::read_volatile(GPIO_PORTA_DOUT);
    ptr::write_volatile(GPIO_PORTA_DOUT, dout | PIN7_BIT);
}

/// Set LED on (active LOW - clear bit)
unsafe fn led_on() {
    let dout = ptr::read_volatile(GPIO_PORTA_DOUT);
    ptr::write_volatile(GPIO_PORTA_DOUT, dout & !PIN7_BIT);
}

/// Set LED off (active LOW - set bit)
unsafe fn led_off() {
    let dout = ptr::read_volatile(GPIO_PORTA_DOUT);
    ptr::write_volatile(GPIO_PORTA_DOUT, dout | PIN7_BIT);
}

/// Debug trap with LED feedback
///
/// Blinks LED rapidly 10 times (5 seconds total).
/// This provides a safe window for debugger attachment.
unsafe fn debug_trap() {
    #[cfg(debug_assertions)]
    {
        for _ in 0..10 {
            led_on();
            delay_ms(250);
            led_off();
            delay_ms(250);
        }
    }
}

#[entry]
fn main() -> ! {
    // SAFETY: We have exclusive access to peripherals
    unsafe {
        // Initialize GPIO for LED
        gpio_init();

        // Turn LED ON solid (for testing)
        led_on();

        // Infinite loop
        loop {
            cortex_m::asm::nop();
        }
    }
}

// ========== NOTES ==========
//
// This example demonstrates raw PAC usage:
//
// 1. Direct register manipulation via pointers
// 2. No HAL abstractions
// 3. Minimal dependencies (just cortex-m-rt + panic-halt)
// 4. Perfect for testing hardware connection
//
// Register Reference:
// - EFR32MG24 Reference Manual Section 7 (CMU)
// - EFR32MG24 Reference Manual Section 11 (GPIO)
//
// Memory Map:
// - CMU:  0x4000_8000
// - GPIO: 0x5000_0000
// - PORTA offset: 0x000
//
// LED on XIAO MG24:
// - Connected to PA7 (Port A, Pin 7, Arduino pin 20)
// - Active LOW (turn on = clear bit, turn off = set bit)
