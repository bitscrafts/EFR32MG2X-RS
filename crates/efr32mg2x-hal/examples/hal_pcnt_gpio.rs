//! # Counting the pulses the converter only sampled
//!
//! The PRS producer measured on 2026-11-26 carries a pad's level, and the IADC converts once per rising edge of it.
//! A converter is a poor counter — every event costs a FIFO slot, a pop and a microsecond of settling — so the same
//! edge is pointed at `PCNT0` here, whose S0IN is a PRS consumer (`CONSUMER_PCNT0_S0IN`). Two questions: does it
//! count at all, and how fast does it follow?
//!
//! ## The pad, moved two ways
//!
//! - **Pull.** A pin in pull mode uses the output latch to select the resistor, so writing the latch moves the pad.
//!   Eight writes are four rising edges; the software counts the `DIN` transitions it actually caused, so every count
//!   is compared to the pad's edges rather than to a wish.
//! - **PWM.** TIMER0's channel 0 drives the same pad at a frequency the CPU chooses — the only way to produce edges
//!   far faster than the core can toggle, which is what a rate ladder needs.
//!
//! ```text
//!   A  channel 3, 8 pull toggles, rising, read running then stopped   -> 4, and the same value twice
//!   B  channel 3, 5 toggles, rising                                   -> 2
//!   C  channel 3, 5 toggles, falling                                  -> 3
//!   D  channel Off, 8 toggles                                         -> 0
//!   E  channel 3, counter stopped, 8 toggles                          -> 0
//!   F  channel 3, pad left alone for the same span                    -> 0
//!   G  producer moved to channel 0, consumer still on 3, PA0          -> 0
//!   H  consumer moved to channel 0, PA0 toggles                       -> 4
//!   I  pad driven as an output, 200 pulses at a ladder of spacings    -> the highest rate fully counted
//! ```
//!
//! A, D, E and F are the same experiment with one thing removed each (the source, the start, the movement). G and H
//! are the consumer word under test: moving the producer alone is not enough, the consumer has to name the channel.
//! B and C differ only in `CTRL.EDGE`, and five toggles are used on purpose — with an even number both polarities
//! would count the same and the phases could not tell each other apart.
//!
//! ## The read
//!
//! The plan for this item said `CNT` is read "through `CMD.CAPTURE`". `CMD` has no capture bit on this part and `CNT`
//! has no writer, so A reports the value read while running and the value read after `CMD.STOPCNT`; a difference
//! between them would be the finding that a running read is not live.
//!
//! ## What the hardware turned out to do
//!
//! ```text
//! pcnt: ch=3 cfg=0x00000100 ctrl=0x00000010 st=0x00000000 a=4 lv=4 b=2 c=3 off=0 stop=0 q=0 mm=0 p0=4 ok=25000 bad=50000 g=77 e=200 of=0 ar=4 br=2 cf=2 cm=4 hr=4 em=8 if=0x00000000
//! ```
//!
//! Two runs, identical. The counter follows the producer exactly where the producer is slow — 8 pull moves gave 4,
//! the same 4 the software counted at the pad — and the controls are all zero. The ladder's wall is between 25 kHz
//! (200 of 200 pulses counted) and 50 kHz (77 of 200): `CFG.MODE = OVSSINGLE` samples S0IN with the `EM23GRPACLK`
//! branch, which this board runs at 32.768 kHz, and `delay_us`'s own overhead means the real spacing is longer than
//! nominal, so 25 kHz is the conservative end of the bracket.
//!
//! One field does not match its software twin: `c=3` and `cf=2`, one falling edge more than the pad made after the
//! counter started. Phases that begin with the pad at the polarity's resting level count exactly (`a`, `p0`), so the
//! extra count belongs to the boundary; the example bounds it instead of pretending it is absent, and explaining it is
//! open (`docs/04-reference/PCNT-GPIO-2026-11-26.md`).
//!
//! ## The faults on the way here
//!
//! `CFG` is refused while `EN` is set (`BFAR = PCNT0 + 0x0C`), `CMD` is synchronised (`SYNCBUSY.CMD`, `BFAR =
//! PCNT0 + 0x14`), and clearing `EN` is not instantaneous — the first read of `EN.DISABLING` happens before the flag is
//! raised. Each of those halted the core once; the driver's `wait_disabled`, `sync` and `write_cmd` are the answer to
//! them. `CFG.MODE = EXTCLKSINGLE` with `CLKSEL = 1` counts the branch instead of S0IN, and `CLKSEL = 2` on a routed
//! GPIO leaves the block unclocked, so the driver offers the sampled path only. The driver's own documentation carries
//! the full list.
//!
//! Record: `docs/04-reference/PCNT-GPIO-2026-11-26.md`.
//!
//! VERIFIED ON HARDWARE (2026-11-26).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pcnt::{Edge, Pcnt};
use efr32mg2x_hal::pin::{EdgeWatch, Input, Port};
use efr32mg2x_hal::prs::Source;
use efr32mg2x_hal::record::Record;
use embedded_hal::delay::DelayNs;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Edges per treated window. Eight is small enough that a lost edge is visible and large enough that one stray edge
/// does not look like a working counter.
const EDGES: u32 = 8;

/// Time between two pull moves. The pad slews in microseconds; this is a hundred times that.
const SPACING_US: u32 = 500;

/// The pin under test, and therefore the external-interrupt channel and the PRS signal number: PC3.
const TEST_PIN: u8 = 3;

/// The second pin: PA0, channel 0, the encoding's least interesting value.
const OTHER_PIN: u8 = 0;

/// The spacings, in microseconds, between the two halves of each pulse. The ladder goes from a rate the sampled path
/// must follow to one it cannot: 100 µs is 5 kHz, 20 µs is 25 kHz, and the branch behind the sampling runs at
/// 32.768 kHz.
const SPACINGS: [u32; 8] = [200, 100, 50, 20, 10, 5, 2, 1];

/// Pulses per ladder rung: every pulse is one rising edge, whatever the spacing.
const PULSES: u32 = 200;

/// What one window measured.
struct Window {
    /// Count after `CMD.STOPCNT`.
    stopped: u16,
    /// Count read with the counter still running.
    live: u16,
    /// `DIN` changes the software saw.
    moved: u32,
    /// Of those, low-to-high.
    rising: u32,
    /// Of those, high-to-low.
    falling: u32,
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            core::hint::spin_loop();
        },
    };
    let mut prs = match board.prs() {
        Ok(prs) => prs,
        Err(_) => fail(b"pcnt: the PRS is not available"),
    };
    let mut pcnt = match board.pcnt() {
        Ok(pcnt) => pcnt,
        Err(_) => fail(b"pcnt: the counter is not available"),
    };

    pcnt.init();
    // TOP resets to 0x00FF on this part, not 0xFFFF: left alone, a window of 200 events would wrap and read back as
    // the remainder. Set it before any count is believed.
    pcnt.set_top(0xffff);

    let tested = board.input_on(Port::C, TEST_PIN);
    let tested_watch = board.watch_edges(Port::C, TEST_PIN, true, true);
    let other = board.input_on(Port::A, OTHER_PIN);
    let other_watch = board.watch_edges(Port::A, OTHER_PIN, true, true);

    // A: the channel bound to the pad's channel.
    if prs.connect(TEST_PIN, Source::GpioPin(TEST_PIN)).is_err() {
        fail(b"pcnt: the PRS refused the channel");
    }
    if pcnt.use_prs_input(TEST_PIN).is_err() {
        fail(b"pcnt: the counter refused the channel");
    }
    let a = phase(
        &board,
        &mut pcnt,
        &tested,
        &tested_watch,
        EDGES,
        Edge::Rising,
    );

    // B and C: the same five toggles, one polarity each.
    let b = phase(
        &board,
        &mut pcnt,
        &tested,
        &tested_watch,
        EDGES / 2 + 1,
        Edge::Rising,
    );
    let c = phase(
        &board,
        &mut pcnt,
        &tested,
        &tested_watch,
        EDGES / 2 + 1,
        Edge::Falling,
    );

    // D: nothing drives the channel while the pad moves.
    if prs.connect(TEST_PIN, Source::Off).is_err() {
        fail(b"pcnt: the PRS refused the channel");
    }
    let d = phase(
        &board,
        &mut pcnt,
        &tested,
        &tested_watch,
        EDGES,
        Edge::Rising,
    );

    // E: the producer back, the counter stopped.
    if prs.connect(TEST_PIN, Source::GpioPin(TEST_PIN)).is_err() {
        fail(b"pcnt: the PRS refused the channel");
    }
    pcnt.reset();
    pcnt.stop();
    let stopped_moved = toggle_all(&board, &tested, &tested_watch, EDGES);
    let e = pcnt.count();

    // F: armed and started, pad left alone for the same span.
    pcnt.count_edges(Edge::Rising);
    let mut wait = board.delay();
    wait.delay_us(EDGES * SPACING_US);
    pcnt.stop();
    let f = pcnt.count();

    // G: the producer moved to channel 0 while the consumer still names channel 3.
    if prs.connect(OTHER_PIN, Source::GpioPin(OTHER_PIN)).is_err() {
        fail(b"pcnt: the PRS refused the channel");
    }
    let g = phase(&board, &mut pcnt, &other, &other_watch, EDGES, Edge::Rising);

    // H: the consumer follows the producer.
    if pcnt.use_prs_input(OTHER_PIN).is_err() {
        fail(b"pcnt: the counter refused the channel");
    }
    let h = phase(&board, &mut pcnt, &other, &other_watch, EDGES, Edge::Rising);

    let (cfg, ctrl, status) = pcnt.words();

    // I: the pad driven from the CPU as a push-pull output, at a ladder of spacings. The number of rising edges is known
    // exactly (one per pulse, whatever the spacing), so the only thing that changes between rungs is the rate — no time
    // base and no second producer are needed to read the ladder. The channel has to be pointed at the pad again:
    // phase D left channel 3 at `Off` and phase H moved the producer to channel 0.
    if prs.connect(TEST_PIN, Source::GpioPin(TEST_PIN)).is_err() {
        fail(b"pcnt: the PRS refused the channel");
    }
    if pcnt.use_prs_input(TEST_PIN).is_err() {
        fail(b"pcnt: the counter refused the channel");
    }
    let output = board.output_on(Port::C, TEST_PIN, false);
    let mut kept_hz = 0u32;
    let mut lost_hz = 0u32;
    let mut lost_count = 0u16;
    let mut lost_expect = 0u32;

    // The presence of the pad's edges is not assumed: the ladder itself is the evidence, because no count can reach 200
    // unless 200 rising edges arrived at the PRS channel from that pad.
    for &spacing_us in SPACINGS.iter() {
        pcnt.count_edges(Edge::Rising);
        for _ in 0..PULSES {
            output.on();
            let mut h = board.delay();
            h.delay_us(spacing_us);
            output.off();
            let mut l = board.delay();
            l.delay_us(spacing_us);
        }
        pcnt.stop();
        let seen = pcnt.count();
        // The rate a caller asks for: one rising edge per `on`, and two spacings per pulse. The delay has its own
        // overhead, so the real rate is slower than this and a rung that keeps up is a conservative claim.
        let nominal_hz = if spacing_us == 0 {
            1_000_000
        } else {
            500_000 / spacing_us
        };
        if u32::from(seen) + 10 >= PULSES {
            kept_hz = nominal_hz;
        } else {
            lost_hz = nominal_hz;
            lost_count = seen;
            lost_expect = PULSES;
            break;
        }
    }

    // The control for the ladder: the highest rate it followed, with the channel pointed at nothing.
    let mut off_counts = 0u16;
    if kept_hz > 0 {
        if prs.connect(TEST_PIN, Source::Off).is_ok() {
            pcnt.count_edges(Edge::Rising);
            for _ in 0..PULSES {
                output.on();
                output.off();
            }
            let mut wait = board.delay();
            wait.delay_us(1_000);
            pcnt.stop();
            off_counts = pcnt.count();
        }
    }

    let mut t = Record::<200>::new();
    t.s(b"pcnt: ch=");
    t.u(TEST_PIN);
    t.s(b" cfg=0x");
    t.h32(cfg);
    t.s(b" ctrl=0x");
    t.h32(ctrl);
    t.s(b" st=0x");
    t.h32(status);
    t.s(b" a=");
    t.u(a.stopped);
    t.s(b" lv=");
    t.u(a.live);
    t.s(b" b=");
    t.u(b.stopped);
    t.s(b" c=");
    t.u(c.stopped);
    t.s(b" off=");
    t.u(d.stopped);
    t.s(b" stop=");
    t.u(e);
    t.s(b" q=");
    t.u(f);
    t.s(b" mm=");
    t.u(g.stopped);
    t.s(b" p0=");
    t.u(h.stopped);
    t.s(b" ok=");
    t.u(kept_hz);
    t.s(b" bad=");
    t.u(lost_hz);
    t.s(b" g=");
    t.u(lost_count);
    t.s(b" e=");
    t.u(lost_expect);
    t.s(b" of=");
    t.u(off_counts);
    t.s(b" ar=");
    t.u(a.rising);
    t.s(b" br=");
    t.u(b.rising);
    t.s(b" cf=");
    t.u(c.falling);
    t.s(b" cm=");
    t.u(c.moved);
    t.s(b" hr=");
    t.u(h.rising);
    t.s(b" em=");
    t.u(stopped_moved.moved);
    t.s(b" if=0x");
    t.h32(pcnt.flags());

    // Every treated count is compared to the edges the pad actually made, not to the number of toggles asked for: a
    // pad that never moved and a counter that never counted agree at zero, and that agreement proves nothing. The
    // ladder needs both a rate it followed and a rate it did not, so a counter stuck at zero cannot pass either.
    let pass = a.stopped == a.rising as u16
        && a.live == a.stopped
        && a.rising == EDGES / 2
        && b.stopped >= 1
        && b.stopped <= b.rising as u16 + 1
        && c.stopped >= 1
        && c.stopped <= c.falling as u16 + 1
        // The two phases used the same five toggles, so a counter that ignored `CTRL.EDGE` would return the same
        // number for both. Different numbers are what proves the field is doing something.
        && b.stopped != c.stopped
        && d.stopped == 0
        && stopped_moved.moved >= EDGES
        && e == 0
        && f == 0
        && g.stopped == 0
        && h.stopped == h.rising as u16
        && h.rising == EDGES / 2
        && kept_hz >= 4_000
        && lost_hz > kept_hz
        && u32::from(lost_count) * 2 <= lost_expect
        && off_counts == 0;
    if pass {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}

/// One window: reset and start, move the pad `toggles` times, read while running, then stop and read again.
fn phase(
    board: &Board,
    pcnt: &mut Pcnt<'_>,
    input: &Input<'_>,
    watch: &EdgeWatch<'_>,
    toggles: u32,
    edge: Edge,
) -> Window {
    pcnt.count_edges(edge);
    let mut window = toggle_all(board, input, watch, toggles);
    window.live = pcnt.count();
    pcnt.stop();
    window.stopped = pcnt.count();
    window
}

/// Move the pad `toggles` times and report what `DIN` did, independently of the counter.
fn toggle_all(board: &Board, input: &Input<'_>, watch: &EdgeWatch<'_>, toggles: u32) -> Window {
    watch.clear();
    let mut moved = 0u32;
    let mut rising = 0u32;
    let mut falling = 0u32;
    let mut level = input.is_high();
    for i in 0..toggles {
        let up = i % 2 == 1;
        input.set_pull_up(up);
        // Poll until the pad has actually reached the level the latch asked for. A single sample 500 µs later was
        // measured to miss the last edge of a phase (the software counted 4 changes where the counter saw 5), and the
        // fix is to follow the pad instead of assuming it has finished slewing.
        let mut settle = 0u32;
        while input.is_high() != up && settle < 200 {
            let mut gap = board.delay();
            gap.delay_us(50);
            settle += 1;
        }
        // Then leave the pad alone for the same span the earlier phases used: the settle loop alone put consecutive
        // edges tens of microseconds apart, which is inside the counter's sampling window and lost them (measured:
        // the pad made four rising edges and the counter read zero).
        let mut gap = board.delay();
        gap.delay_us(SPACING_US);
        let now = input.is_high();
        if now != level {
            moved += 1;
            if now {
                rising += 1;
            } else {
                falling += 1;
            }
        }
        level = now;
    }
    if toggles == 0 {
        // A window with nothing in it must last as long as one that has something: a phase cannot pass by ending
        // before the producer had a chance.
        let mut wait = board.delay();
        wait.delay_us(EDGES * SPACING_US);
    }
    Window {
        stopped: 0,
        live: 0,
        moved,
        rising,
        falling,
    }
}

fn fail(message: &[u8]) -> ! {
    mailbox::fail(message);
    loop {
        cortex_m::asm::wfi();
    }
}
