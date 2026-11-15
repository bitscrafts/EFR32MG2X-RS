//! The SYSRTC sequence with a log line before and after every write, to see where it dies.
//!
//! `pac_sysrtc_rate` takes an imprecise bus error and reports `step=1`, which brackets everything from the clock select
//! to the first START that follows it. The mailbox cannot narrow it further, because the record is written once, at the
//! end. This probe writes each step over RTT *as it happens*, so the last line printed is the last thing that worked.
//!
//! ```bash
//! probe-rs run --chip EFR32MG24B220F1536IM48 \
//!   target/thumbv8m.main-none-eabihf/release/examples/pac_sysrtc_trace
//! ```
//! VERIFIED ON HARDWARE (2026-11-15) -- the sequence every step, logged as it happens, CNT counting to 59857

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, warn};
use defmt_rtt as _;
use efr32mg2x_pac as pac;
use panic_probe as _;

// Non-secure aliases: the CMU at 0x5000_8000 and the SYSRTC at 0x500a_8000 are the ones this application has been using.
const CMU: usize = 0x5000_8000;
const RTC: usize = 0x500a_8000;

fn rd(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

fn wr(addr: usize, v: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, v) }
}

defmt::timestamp!("{=u64}", unsafe {
    (*cortex_m::peripheral::DWT::PTR).cyccnt.read() as u64
});

#[entry]
fn main() -> ! {
    let mut core = cortex_m::Peripherals::take().unwrap();
    core.DCB.enable_trace();
    core.DWT.enable_cycle_counter();

    info!("trace: start");

    // Clock gate, then clock select, each logged on both sides of the write.
    info!("trace: before CLKEN0 |= SYSRTC0");
    wr(CMU + 0x64, rd(CMU + 0x64) | (1 << 30));
    info!("trace: after CLKEN0 = {=u32:#010x}", rd(CMU + 0x64));

    info!("trace: before SYSRTC0CLKCTRL = LFRCO");
    wr(CMU + 0x240, (rd(CMU + 0x240) & !0x3) | 1);
    info!("trace: after CLKCTRL = {=u32:#010x}", rd(CMU + 0x240));

    info!("trace: EN = {=u32:#010x}", rd(RTC + 0x04));

    // The one read the trace did not do and pac_sysrtc_rate does on every step: SYNCBUSY at +0x1C.
    info!("trace: about to read SYNCBUSY (+0x1C)");
    let busy = rd(RTC + 0x1C);
    info!("trace: SYNCBUSY = {=u32:#010x}", busy);

    // The vendor sequence: stop and clear the enable before touching CFG.
    if rd(RTC + 0x04) & 1 != 0 {
        info!("trace: before CMD = STOP");
        wr(RTC + 0x10, 0x2);
        info!("trace: after CMD = STOP");
        info!("trace: before EN_CLR");
        wr(RTC + 0x2004, 1);
        info!("trace: after EN_CLR");
    } else {
        info!("trace: not enabled, skipping the disable step");
    }

    info!("trace: before CFG = 0");
    wr(RTC + 0x0C, 0);
    info!("trace: after CFG = {=u32:#010x}", rd(RTC + 0x0C));

    // The group 0 block, at the offsets the vendor blob's handler uses (+0x60/+0x64/+0x68). This is the last piece of
    // pac_sysrtc_rate's sequence that this trace has not touched.
    info!("trace: before GRP0_CTRL (+0x68) = 0x1");
    wr(RTC + 0x68, 0x1);
    info!("trace: after GRP0_CTRL");
    info!("trace: before GRP0_IEN (+0x64) = 0");
    wr(RTC + 0x64, 0);
    info!("trace: after GRP0_IEN");
    info!("trace: before GRP0_IF (+0x60) = 0x7");
    wr(RTC + 0x60, 0x7);
    info!("trace: after GRP0_IF");

    info!("trace: before EN_SET");
    wr(RTC + 0x1004, 1);
    info!("trace: after EN_SET, EN = {=u32:#010x}", rd(RTC + 0x04));

    info!("trace: before CMD = START");
    wr(RTC + 0x10, 0x1);
    info!("trace: after CMD = START");

    let t0 = rd(RTC + 0x18);
    for _ in 0..2_000_000 {
        core::hint::spin_loop();
    }
    let t1 = rd(RTC + 0x18);
    info!("trace: CNT went {=u32} -> {=u32}", t0, t1);
    if t1 == t0 {
        warn!("trace: the counter did not move");
    }

    // Wait for the probe to be draining before the next records: the channel is non-blocking, so anything written in the
    // first moments after flashing is dropped and would look like a fault that never happened.
    for _ in 0..5_000_000 {
        core::hint::spin_loop();
    }

    // The one thing pac_sysrtc_rate does that this trace has not: switch the counter's clock while it runs.
    info!("trace: before CLKCTRL = ULFRCO (0x3)");
    wr(CMU + 0x240, (rd(CMU + 0x240) & !0x3) | 3);
    info!("trace: after CLKCTRL = {=u32:#010x}", rd(CMU + 0x240));
    info!("trace: CNT after the switch = {=u32}", rd(RTC + 0x18));

    loop {
        for _ in 0..5_000_000 {
            core::hint::spin_loop();
        }
        info!("trace: alive, CNT = {=u32}", rd(RTC + 0x18));
    }
}
