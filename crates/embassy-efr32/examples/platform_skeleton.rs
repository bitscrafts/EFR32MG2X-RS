//! The smallest thing a BSP has to write: a `Platform`, and the driver installed on it.
//!
//! This example exists because the gate refuses a crate whose API nothing builds. It is a skeleton
//! of the wiring, not a driver: the two platform functions are stubs, and nothing here touches the
//! radio or the clock. What it proves is that the trait, the driver and the `time_driver!` macro fit
//! together and compile for the chip -- which is the part that cannot be discovered later without
//! rewriting the crate.
//!
//! On a real board the two stubs become two calls into `efr32mg2x_hal::sysrtc`:
//!
//! ```ignore
//! fn ticks_64() -> u64 { /* the HAL's 64-bit tick, fed by the ISR */ }
//! fn arm(value: u32, rearm: bool) { /* the HAL's comparator, and one note for the ISR */ }
//! ```
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_efr32::{time_driver, Platform};
use panic_halt as _;

/// What the BSP supplies. Stubs here, on purpose.
struct SysrtcPlatform;

impl Platform for SysrtcPlatform {
    fn ticks_64() -> u64 {
        0
    }

    fn arm(_value: u32, _rearm: bool) {}
}

// Installs the driver so `embassy-time` can find it. In a real BSP this appears once.
time_driver!(SysrtcPlatform);

#[entry]
fn main() -> ! {
    // Nothing to run: the example's job is to make the API build. A real BSP would start the clock,
    // enable the comparator interrupt, and let `embassy-executor` take over.
    loop {
        cortex_m::asm::wfi();
    }
}
