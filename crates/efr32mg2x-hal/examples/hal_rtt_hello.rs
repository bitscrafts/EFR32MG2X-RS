//! Deferred-formatting logging over RTT, and a panic that says what happened.
//!
//! Why this rather than a hand-rolled ring buffer: `defmt` keeps the format strings on the host and sends only an index
//! plus the arguments, so the flash cost is a handful of bytes per message instead of the text, and `panic-probe`
//! prints the panic message and a backtrace through the same channel. The probe reads all of it over the SWD cable that
//! flashed the board, while the core keeps running.
//!
//! ```bash
//! probe-rs run --chip EFR32MG24B220F1536IM48 \
//!   target/thumbv8m.main-none-eabihf/release/examples/hal_rtt_hello
//! ```
//!
//! The last line deliberately divides by a value the compiler cannot know, to show what a panic looks like over RTT: a
//! program that faults is the case the mailbox instrument cannot report, because the mailbox is only read after the
//! program has written it.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, warn};
use defmt_rtt as _;
// Linking the HAL is what brings the PAC's interrupt vectors; without it cortex-m-rt refuses to link.
use efr32mg2x_hal as _;
use panic_probe as _;

// Timestamp for every record: defmt requires one when a panic handler prints through it, and the DWT cycle counter is
// the cheapest real clock on this core. Enabled in `main` before the first record; reading it has no side effects.
defmt::timestamp!("{=u64}", unsafe {
    (*cortex_m::peripheral::DWT::PTR).cyccnt.read() as u64
});

#[entry]
fn main() -> ! {
    let mut core = cortex_m::Peripherals::take().unwrap();
    core.DCB.enable_trace();
    core.DWT.enable_cycle_counter();

    // A heartbeat rather than one burst at boot: defmt-rtt runs the up channel non-blocking, so a record written before
    // the probe has attached is dropped -- there is nobody to hand it to. A program that logs once and then sleeps
    // looks silent no matter how correct it is.
    let mut beat = 0u32;
    loop {
        info!("rtt: heartbeat {} -- {} MHz", beat, 78u32);
        beat = beat.wrapping_add(1);
        for _ in 0..2_000_000 {
            core::hint::spin_loop();
        }
    }
}
