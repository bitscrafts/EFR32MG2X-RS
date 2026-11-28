//! SYSRTC's 64-bit tick, the comparator waking the core, and an independent clock confirming it.
//!
//! ## The two phases, and why there are two
//!
//! **Phase A -- sleep.** Ten 100 ms deadlines armed on the comparator, with the core in `wfi` between them. It proves
//! the comparator fires (the interrupt count) and that the 64-bit tick measures the total.
//!
//! **Phase B -- spin.** The same ten intervals, but waiting for the wide tick **without** `wfi`. Here the DWT's cycle
//! counter is worth something, because the core is awake.
//!
//! The second phase exists because of a measurement, not for symmetry. The first version timed phase A with the DWT
//! and read `wall_ms=1` for a second of sleep: **`CYCCNT` counts core cycles, and with the core in `wfi` the core's
//! clock is stopped**. The instrument could not answer the question, and a criterion that used it was measuring the
//! tool instead of the driver. In phase B the DWT measures an interval that contains the same SYSRTC counter, and the
//! two numbers can now disagree -- which is what makes them a control.
//!
//! ## The criterion
//!
//! Ten comparator firings; the wide tick says 1000 ms in both phases; the DWT, in the phase where it is worth
//! anything, agrees with the wide tick to within 5%; no wrap lost. The tolerance covers the 32 kHz crystal and the
//! cost of waking.
//!
//! Read it with `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
// `panic-probe` and not `panic-halt`: the `defmt` the HAL pulls in needs a transport, and a panic over the same SWD
// cable is what turns a silent panic into a readable line.
use defmt_rtt as _;
use panic_probe as _;

/// How many 100 ms waits each phase makes.
const WAITS: u32 = 10;

/// A 100 ms wait in the SYSRTC counter: 3276.8 ticks at 32768 Hz, rounded down.
const TICKS_100MS: u32 = 3_276;

/// SYSRTC ticks in a second, for the report.
const TICKS_HZ: u64 = 32_768;

/// How many times the comparator woke the core.
///
/// The handler writes here because `main` is inside the `wfi`: there is no way to lend it the `Sysrtc`, and without a
/// shared place to count in, "it woke ten times" and "it woke once and got stuck" would produce the same record.
static WAKES: AtomicU32 = AtomicU32::new(0);

/// The strong handler that takes the vector's `DefaultHandler`'s place.
///
/// The PAC declares `PROVIDE(SYSRTC_APP_IRQHandler = DefaultHandler)` in `device.x`, so a definition with this exact
/// name takes the place of the handler that does not return. `cortex-m-rt`'s `#[interrupt]` attribute needs the
/// `device` feature, which the HAL does not enable; the vector's name is what the linker looks for.
#[no_mangle]
pub extern "C" fn SYSRTC_APP_IRQHandler() {
    efr32mg2x_hal::sysrtc::Sysrtc::acknowledge_compare_interrupt();
    WAKES.fetch_add(1, Ordering::Relaxed);
}

/// One report, with the numbers that decide.
fn report(
    ok: bool,
    sleep_ms: u32,
    wakes: u32,
    spin_ms: u32,
    wall_b_ms: u32,
    cyc0: u32,
    cyc1: u32,
    wraps: u32,
) {
    let mut m = [0u8; 160];
    let mut n = 0usize;

    fn text(m: &mut [u8], n: &mut usize, s: &[u8]) {
        for b in s {
            if *n < m.len() {
                m[*n] = *b;
                *n += 1;
            }
        }
    }
    fn number(m: &mut [u8], n: &mut usize, v: u32) {
        let mut d = [0u8; 10];
        let mut k = 0usize;
        let mut v = v;
        loop {
            d[k] = b'0' + (v % 10) as u8;
            v /= 10;
            k += 1;
            if v == 0 || k == d.len() {
                break;
            }
        }
        while k > 0 {
            k -= 1;
            if *n < m.len() {
                m[*n] = d[k];
                *n += 1;
            }
        }
    }

    text(&mut m, &mut n, b"wide: sleep_ms=");
    number(&mut m, &mut n, sleep_ms);
    text(&mut m, &mut n, b" wakes=");
    number(&mut m, &mut n, wakes);
    text(&mut m, &mut n, b" spin_ms=");
    number(&mut m, &mut n, spin_ms);
    text(&mut m, &mut n, b" spin_wall=");
    number(&mut m, &mut n, wall_b_ms);
    text(&mut m, &mut n, b" c0=");
    number(&mut m, &mut n, cyc0);
    text(&mut m, &mut n, b" c1=");
    number(&mut m, &mut n, cyc1);
    text(&mut m, &mut n, b" wraps=");
    number(&mut m, &mut n, wraps);

    if ok {
        mailbox::write(mailbox::Status::Pass, &m[..n]);
    } else {
        mailbox::write(mailbox::Status::Fail, &m[..n]);
    }
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = Board::take().unwrap();
    let led = board.output_on(Port::A, 7, false);
    let rtc = board.sysrtc().expect("SYSRTC is handed out once");

    // The comparator's interrupt has to be enabled in the peripheral before the first `wfi`.
    rtc.unmask_interrupt();

    // `Hertz` is a type with the unit inside: converting to an integer here is explicit, not a disguised `as`.
    let hw_hz: u32 = board.clock_hz().into();
    let _ = unsafe { efr32mg2x_hal::dwt::start() };

    // ## Phase A: sleep on each deadline
    let ticks_a0 = rtc.now_wide();
    for _ in 0..WAITS {
        // The deadline comes from the WIDE tick and only the truncation to 32 bits is done at the end: that is the
        // arithmetic that survives one counter wrap, and it is what `schedule_at` receives.
        let target = rtc.now_wide() + TICKS_100MS as u64;
        let before = WAKES.load(Ordering::Relaxed);
        rtc.schedule_at(target as u32);

        // Wait for the comparator to fire. The iteration bound exists so that a mute comparator gives a red instead
        // of a hung example -- a `wfi` returns on any interrupt, not only ours.
        let mut spins = 0u32;
        while WAKES.load(Ordering::Relaxed) == before {
            cortex_m::asm::wfi();
            spins += 1;
            if spins > 4_000_000 {
                break;
            }
        }
    }
    let ticks_a = (rtc.now_wide() - ticks_a0) as u32;
    let wakes = WAKES.load(Ordering::Relaxed);

    // ## Phase B: the same intervals, with the core awake
    //
    // Without `wfi`, and waiting for the SAME wide tick. This is the phase where the DWT measures wall time; in phase
    // A the core's counter stops when the core sleeps.
    // **The DWT does not survive sleep.** Measured twice, in two ways: with the counter timing phase A it gave 1 ms
    // for a second (the core sleeps and the core's clock stops), and with the raw values in the record one saw
    // `c0=27297 c1=27297` -- frozen at the first `wfi`, not resuming after waking. So it is restarted HERE, after the
    // sleep, and measures only the phase in which the core does not sleep. An instrument that cannot answer the
    // question does not enter the criterion.
    let _ = unsafe { efr32mg2x_hal::dwt::start() };
    let ticks_b0 = rtc.now_wide();
    let cycles_b0 = efr32mg2x_hal::dwt::cycles();
    for _ in 0..WAITS {
        let target = rtc.now_wide() + TICKS_100MS as u64;
        // `is_future` is the 32-bit comparison that survives the counter wrap; the decision lives in the tested core,
        // not here.
        while efr32mg2x_hal::sysrtc::is_future(rtc.now(), target as u32) {
            cortex_m::asm::nop();
        }
    }
    let ticks_b = (rtc.now_wide() - ticks_b0) as u32;
    let cycles_b1 = efr32mg2x_hal::dwt::cycles();
    let cycles = cycles_b1.wrapping_sub(cycles_b0);

    let sleep_ms = (ticks_a as u64 * 1000 / TICKS_HZ) as u32;
    let spin_ms = (ticks_b as u64 * 1000 / TICKS_HZ) as u32;
    let wall_b_ms = if hw_hz == 0 {
        0
    } else {
        (cycles as u64 * 1000 / u64::from(hw_hz)) as u32
    };

    let expected = WAITS * 100;
    let margin = expected / 20;
    let near = |v: u32| v >= expected - margin && v <= expected + margin;
    // The DWT and the SYSRTC measure the same interval in phase B: a factor of two, a lost wrap or a swapped clock
    // all show up here as a difference the margin does not cover.
    let agree = wall_b_ms.abs_diff(spin_ms) <= margin;
    let ok = wakes == WAITS && near(sleep_ms) && near(spin_ms) && agree;

    report(
        ok,
        sleep_ms,
        wakes,
        spin_ms,
        wall_b_ms,
        cycles_b0,
        cycles_b1,
        rtc.wraps(),
    );
    led.on();

    loop {
        cortex_m::asm::wfi();
    }
}
