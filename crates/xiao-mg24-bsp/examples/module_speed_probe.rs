//! Spin with the cycle counter running, so the core's speed can be measured from outside.
//!
//! Every on-chip reference we have is in doubt: the HAL says 19 MHz, the delay defect implies ~44 MHz, and a
//! probe that used the SYSRTC as its reference produced an impossible 1.75 MHz because that counter is not on
//! the 32768 Hz source it assumed. So this firmware carries no reference at all.
//!
//! It starts `DWT->CYCCNT` and spins. The measurement is done from the host:
//!
//! ```text
//! a=$(probe-rs read b32 0xE0001004 1)      # DWT->CYCCNT
//! sleep 2
//! b=$(probe-rs read b32 0xE0001004 1)
//! core_hz = (b - a) / 2
//! ```
//!
//! The reference is the host's own clock, and the only assumption is that two seconds on the host are two
//! seconds. Nothing on the part has to be trusted, and the same trick works for any board with a debug probe.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;
use xiao_mg24_bsp::Module;

#[path = "common/mailbox.rs"]
mod mailbox;

#[entry]
fn main() -> ! {
    // SAFETY: the mailbox slot is this program's own record, in RAM, written by nothing else.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    // Keep the board's own peripherals taken so nothing else reconfigures the clock under us.
    let _module = match Module::take() {
        Some(m) => m,
        None => loop {
            core::hint::spin_loop();
        },
    };

    // SAFETY: nothing else in this program touches DWT; the counter is started once, here.
    let start = unsafe { dwt::start() }
        .expect("dwt: the cycle counter does not run without a debug session");
    mailbox::info(b"speed: DWT running, spinning for the host to time");

    // Deliberately a spin, not a WFI: a sleeping core may stop counting, and this is about the clock.
    loop {
        if dwt::since(start) % 1_000_000 == 0 {
            cortex_m::asm::nop();
        }
    }
}
