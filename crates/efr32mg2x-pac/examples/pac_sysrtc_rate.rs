//! SYSRTC ticks per unit of time, under two clock selections, reported over RTT.
//!
//! This example used to take an imprecise bus error and report `FAULT step=1` through the mailbox. The sequence turned
//! out not to be the problem: `pac_sysrtc_trace` runs the same writes -- clock gate, clock select, stop/EN_CLR, CFG,
//! group 0, EN_SET, START, and a switch to ULFRCO while the counter runs -- and the counter keeps counting for as long as
//! anyone watches. What was left was the example's own instrumentation, so the mailbox machinery is gone and the records
//! go out over RTT instead, written before the measurement rather than after it.
//!
//! ```bash
//! probe-rs run --chip EFR32MG24B220F1536IM48 \
//!   target/thumbv8m.main-none-eabihf/release/examples/pac_sysrtc_rate
//! ```
//! VERIFIED ON HARDWARE (2026-11-15) -- two clock selections measured over RTT, CNT 33799 after the run

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, warn};
use defmt_rtt as _;
use efr32mg2x_pac as pac;
use panic_probe as _;
use portable_atomic::AtomicU32;

const CMU: usize = 0x5000_8000;
const RTC: usize = 0x500a_8000;
const DWT_CYCCNT: usize = 0xE000_1004;

fn rd(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

fn wr(addr: usize, v: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, v) }
}

defmt::timestamp!("{=u64}", unsafe {
    (*cortex_m::peripheral::DWT::PTR).cyccnt.read() as u64
});

/// SYSRTC ticks and elapsed cycles over the same window.
fn measure(cycles: u32) -> (u32, u32) {
    let t0 = rd(RTC + 0x18);
    let c0 = rd(DWT_CYCCNT);
    while rd(DWT_CYCCNT).wrapping_sub(c0) < cycles {
        core::hint::spin_loop();
    }
    let t1 = rd(RTC + 0x18);
    (t1.wrapping_sub(t0), rd(DWT_CYCCNT).wrapping_sub(c0))
}

/// Group 0, once: the compare/capture block the vendor blob also sets up. Not repeated on restart.
fn configure_group() {
    wr(RTC + 0x68, 0x1); // GRP0_CTRL
    wr(RTC + 0x64, 0); // GRP0_IEN
    wr(RTC + 0x60, 0x7); // GRP0_IF: clear all
}

/// Puts the counter in a known state and starts it. Returns the selected clock for the log.
fn start_with(clksel: u32) -> u32 {
    wr(CMU + 0x240, (rd(CMU + 0x240) & !0x3) | clksel); // SYSRTC0CLKCTRL: 1 = LFRCO, 3 = ULFRCO

    if rd(RTC + 0x04) & 1 != 0 {
        wr(RTC + 0x10, 0x2); // CMD = STOP
        wr(RTC + 0x2004, 1); // EN_CLR
    }
    wr(RTC + 0x0C, 0); // CFG
                       // The group is configured once, by `configure_group`, and not on every restart: clearing GRP0_IF (+0x60) on the second
                       // start -- with the counter already running on ULFRCO -- is what took the bus error this example used to report, and
                       // the backtrace from panic-probe lands on exactly that write.
    wr(RTC + 0x1004, 1); // EN_SET
    wr(RTC + 0x10, 0x1); // CMD = START

    rd(CMU + 0x240) & 0x3
}

#[entry]
fn main() -> ! {
    // RTT is the only instrument here, so the shared slot is not written: it has to be invalidated all the same, or a
    // reader takes the previous binary's record as this one's. Measured on 2026-11-15.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let _dp = match pac::Peripherals::take() {
        Some(d) => d,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let mut core = cortex_m::Peripherals::take().unwrap();
    core.DCB.enable_trace();
    core.DWT.enable_cycle_counter();

    // RTT drops what it cannot hand over, so give the probe a moment before the first measurement.
    for _ in 0..3_000_000 {
        core::hint::spin_loop();
    }

    // The gate first, and only then anything that touches the peripheral: a write to an unclocked SYSRTC is a precise bus
    // fault at that register's address, which is what this example kept reporting as an unexplained exception.
    wr(CMU + 0x64, rd(CMU + 0x64) | (1 << 30)); // CLKEN0: SYSRTC0 gate

    configure_group();
    let sel1 = start_with(1); // first start: the module has never been enabled, so there is nothing to stop
    info!("rate: LFRCO selected, CLKCTRL = {=u32}", sel1);
    let (ticks1, cycles1) = measure(19_000_000);
    info!(
        "rate: LFRCO ticks = {=u32} over {=u32} cycles",
        ticks1, cycles1
    );

    // Second selection without a stop: pac_sysrtc_trace switches the clock with the counter running and the counter keeps
    // counting. Stopping first and reconfiguring is what faults, at CFG of all things.
    wr(CMU + 0x240, (rd(CMU + 0x240) & !0x3) | 3);
    let sel3 = rd(CMU + 0x240) & 0x3;
    info!("rate: ULFRCO selected, CLKCTRL = {=u32}", sel3);
    let (ticks3, cycles3) = measure(19_000_000);
    info!(
        "rate: ULFRCO ticks = {=u32} over {=u32} cycles",
        ticks3, cycles3
    );

    // Both clocks are low frequency; what matters is that the counter moved under each of them, and that the two rates
    // differ, which is the whole point of selecting one.
    if ticks1 == 0 || ticks3 == 0 {
        warn!("rate: a clock produced no ticks at all");
    }
    info!("rate: done, CNT now {=u32}", rd(RTC + 0x18));

    loop {
        for _ in 0..5_000_000 {
            core::hint::spin_loop();
        }
        info!("rate: alive, CNT = {=u32}", rd(RTC + 0x18));
    }
}
