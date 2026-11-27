//! EUSART0 in asynchronous mode: eight distinct bytes out and back through `CFG0.LOOPBK`, and a control that reads
//! none.
//!
//! ## Why the EUSART needed its own instrument
//!
//! The crate carried "USART0 only" as a limitation, and the two EUSART blocks were reachable only as `spi::Spi1` and
//! `spi::Spi2` in synchronous mode. The registers are not the same block under another name: the frame lives in
//! `FRAMECFG` (where eight data bits is the value `2`, not the USART's `5`), the asynchronous divider is `CLKDIV.DIV`
//! while the synchronous one is `CFG2.SDIV`, and the baud clock comes from `CMU_EUSART0CLKCTRL` rather than from the
//! bus clock alone. A driver copied from `usart` would compile and produce no frames at all.
//!
//! ## The three runs, and what each one rules out
//!
//! 1. **Control, loopback off.** The same eight bytes are written, the pads are routed, the receiver's pad is pulled
//!    up, and nothing is looped. The record must show **zero** bytes received. Without this run, "eight bytes came
//!    back" would not distinguish the internal loop from a receiver reading its own noise -- and this project has
//!    already measured a floating USART line producing 271 bytes of noise (`hal_usart_link`).
//! 2. **Loopback on.** Eight distinct bytes, written one at a time and drained one at a time, because the receiver's
//!    FIFO is shallow and the transmitter would otherwise outrun it. Every byte has to come back identical, and the
//!    `IF` error flags have to be clear.
//! 3. **Timing.** The same eight bytes written back to back with the loopback still on and nobody draining: the RX
//!    FIFO overflows (which is itself the evidence that the receiver saw the frames), and the interval is measured on
//!    SYSRTC. Eight 8N1 frames are 80 bit times, so the rate the divider actually produced is `80 * 32768 / ticks`.
//!    That number is what catches a divider computed against the wrong clock -- the defect that once put `52` in a
//!    field that wanted `2382` and produced a console nobody could read.
//!
//! ## What the first three attempts measured, before this one passed
//!
//! The instrument earned its keep three times, and each failure is now a rule in the driver:
//!
//! 1. **`CFG0` is not writable with `EN` set.** The first version wrote `CFG0.LOOPBK` on a live block and died with a
//!    precise bus fault at `EUSART0_S + 0x08` (`CFSR=0x8200`, `BFAR=0x4b010008`). The USART's `CTRL.LOOPBK` tolerates
//!    the write; this register does not.
//! 2. **Dropping `EN` is not instantaneous.** Clearing the enable and writing `CFG0` on the next bus access faulted at
//!    the same address. `EN.DISABLING` has to go low first, and after the cycle `CMD.TXEN`/`RXEN` have to be asserted
//!    again -- without them the transmitter never drains and `write_byte` waits for ever (measured: the run stopped after
//!    the third byte with no record and no fault).
//! 3. **The EUSART does not share the USART's divider.** With the USART arithmetic (`1 + DIV/256`) a request for 115 200
//!    produced 15 650 baud -- 167 ticks for 80 bit times, with the same SYSRTC calibrated at 3290 ticks per 100 ms. The
//!    EUSART's field carries five fraction bits (`1 + DIV/32`); with `DIV = 297` the same request produced 23 ticks and
//!    113 975 baud on the wire.
//!
//! ## What would make this instrument lie
//!
//! A pattern of zeros would pass on a line that drives nothing, so the eight bytes are distinct and include `0x00` and
//! `0xFF`. Polling `RXDATAV` alone would accept noise, so each byte is also gated on the block's error flags. The time
//! base is calibrated in the same run (`cal`, a 100 ms board delay against SYSRTC) before anything is timed with it, and
//! the band around 23 ticks is wide enough for the oscillators but far too narrow for a divider out by 2x.
//!
//! VERIFIED ON HARDWARE (2026-11-27), four boots (the fourth from the formatted tree). Three were byte-identical;
//! the fourth differs in one field, `cal`, by one tick:
//! `eusart: ok=1 n=8 am=8 wr=8 err=0 ctl=0 div=297 cal=3290 ct=23 tt=23 bd=113975 if=0x000c2007 ft=0x000c2007
//! fc=0x000c2007 cf=0x00000002`. The `IF` value is `TXC|TXFL|RXFL|TXIDLE|STARTF|SIGF`: no `PERR`, no `FERR`, no `RXOF`.
//! `RXFL` set after the timing run is the receiver still holding the eight frames nobody drained, and `RXOF` clear says
//! the FIFO took all eight. Record: `docs/04-reference/EUSART-ASYNC-2026-11-27.md`.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::entry;
use efr32mg2x_hal::eusart::{Eusart0, IF_FERR, IF_PERR, IF_RXOF};
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::record::Record;
use efr32mg2x_hal::usart::Config;
use embedded_hal::delay::DelayNs;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// What goes out, and therefore what must come back. Distinct values, including both rails: a line that drives
/// nothing reads `0x00`, and so does a working line sending zeros.
const PATTERN: [u8; 8] = [0xA5, 0x5A, 0x00, 0xFF, 0x0F, 0xF0, 0x12, 0x34];

/// The rate asked for. Chosen because the divider it needs (2382 at 19 MHz, 16x) is far from both ends of the field.
const BAUD: u32 = 115_200;

/// Bit times in one 8N1 frame: start, eight data, stop.
const BITS_PER_FRAME: u64 = 10;

/// SYSRTC ticks per second. The counter is the part's 32.768 kHz branch, measured against a host stopwatch elsewhere in
/// this crate; here it is only used as a ratio against itself.
const TICK_HZ: u64 = 32_768;

/// Ticks eight 8N1 frames should take at `BAUD`: 80 bits is 694 us, and 694 us is 22.7 ticks. The band below accepts
/// 18..=30 -- a divider out by 2x lands at 11 or 46 and fails.
const TICKS_MIN: u32 = 18;
const TICKS_MAX: u32 = 30;

/// How long a single expected byte is waited for, in polls of `RXDATAV`. At 19 MHz a poll is a few cycles and a frame
/// is about 1650 of them, so this is orders of magnitude more than one frame and still bounded: a wait that never ends
/// is indistinguishable from a hang.
const RX_POLLS: u32 = 200_000;

/// How long the control drains for, in polls. The control expects nothing; this is the budget in which something would
/// have to show up if the loopback were on.
const DRAIN_POLLS: u32 = 400_000;

/// Progress, written at every step. "No record" then says something died and not where.
///
///   0 inside run 2: 1 + bytes written | 1 took the peripherals | 2 the port came up | 3 control done
///   4 loopback run done | 5 timing done
#[used]
#[no_mangle]
static ETAPAS: [core::sync::atomic::AtomicU32; 6] =
    [const { core::sync::atomic::AtomicU32::new(0) }; 6];

fn etapa(i: usize, v: u32) {
    ETAPAS[i].store(v, core::sync::atomic::Ordering::Relaxed);
}

/// Read everything the receiver holds, up to `polls` polls of the status register. Counts what it accepted.
fn drain(uart: &mut Eusart0, polls: u32) -> u32 {
    let mut seen = 0u32;
    let mut quiet = 0u32;
    for _ in 0..polls {
        match uart.read_byte() {
            Some(Ok(_)) => {
                seen += 1;
                quiet = 0;
            }
            Some(Err(_)) => {
                seen += 1;
                quiet = 0;
            }
            None => {
                quiet += 1;
                // Two thousand consecutive empty polls after a `flush` is far longer than a frame; stop rather than
                // spend the whole budget pretending to look.
                if quiet > 2_000 {
                    break;
                }
            }
        }
    }
    seen
}

/// Wait for one byte, bounded. `None` means nothing arrived in the budget.
fn receive_one(uart: &mut Eusart0) -> Option<Result<u8, efr32mg2x_hal::usart::RxProblem>> {
    for _ in 0..RX_POLLS {
        if let Some(byte) = uart.read_byte() {
            return Some(byte);
        }
    }
    None
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => {
            etapa(1, 1);
            board
        }
        None => {
            etapa(1, 9);
            mailbox::fail(b"eusart: peripherals taken twice");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // TX on PA0, RX on PC3: two module pads nothing else in this example touches, and no jumper anywhere near them.
    // The loop is internal, so the pads only have to hold a defined level -- the RX one is pulled up by the routing.
    if board
        .route_eusart0_async((Port::A, 0), (Port::C, 3))
        .is_err()
    {
        etapa(2, 3);
        mailbox::fail(b"eusart: the board refused the pad pair");
        loop {
            cortex_m::asm::wfi();
        }
    }
    let mut uart = match board.eusart0_async(Config::new(BAUD)) {
        Ok(uart) => {
            etapa(2, 1);
            uart
        }
        Err(_) => {
            etapa(2, 2);
            mailbox::fail(b"eusart: the board refused the port or the rate");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    let divider = uart.divider();

    // ---- run 1: control, loopback off ---------------------------------------------------------------------------
    // The time base is taken before anything is timed: `now()` on SYSRTC is only useful if the counter really runs at
    // 32768 Hz, so the first thing measured is a delay the board itself generates. `cal` should be about 3277 ticks
    // for 100 ms; a different number says the instrument is wrong and every `tt` below with it.
    let rtc = match board.sysrtc() {
        Ok(rtc) => rtc,
        Err(_) => {
            etapa(2, 4);
            mailbox::fail(b"eusart: SYSRTC already handed out, so nothing can be timed");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };
    let c0 = rtc.now();
    board.delay().delay_ms(100);
    let c1 = rtc.now();
    let cal_ticks = c1.wrapping_sub(c0);

    let (flags_before_control, _) = uart.clear_error_flags();
    let c0 = rtc.now();
    uart.write(&PATTERN);
    uart.flush();
    let c1 = rtc.now();
    let control_ticks = c1.wrapping_sub(c0);
    let ctl = drain(&mut uart, DRAIN_POLLS);
    etapa(3, 1 + ctl);

    // ---- run 2: loopback on, one byte at a time -----------------------------------------------------------------
    uart.set_internal_loopback(true);
    let mut matched = 0u32;
    let mut written = 0u32;
    for &byte in PATTERN.iter() {
        uart.write_byte(byte);
        written += 1;
        etapa(0, 1 + written);
        match receive_one(&mut uart) {
            Some(Ok(got)) if got == byte => matched += 1,
            Some(_) => {}
            None => {}
        }
    }
    uart.flush();
    let refused = uart.rx_errors();
    let loop_flags = uart.rx_flags();
    etapa(4, 1 + matched + 10 * refused);

    // ---- run 3: timing, loopback still on, nobody draining ------------------------------------------------------
    let t0 = rtc.now();
    uart.write(&PATTERN);
    uart.flush();
    let t1 = rtc.now();
    let ticks = t1.wrapping_sub(t0);
    etapa(5, 1);

    // The timing run wrote eight frames into a receiver nobody drained. What the block latched here is a fact about what
    // the receiver saw; the reading after `clear_error_flags` says whether this block's `IF` clears the way the reference
    // manual says (the USART0 measurement found that its flags did not).
    let (flags_after_timing, flags_after_clear) = uart.clear_error_flags();

    let measured_baud = if ticks > 0 {
        (BITS_PER_FRAME * PATTERN.len() as u64 * TICK_HZ / ticks as u64) as u32
    } else {
        0
    };

    let error_mask = IF_RXOF | IF_PERR | IF_FERR;
    let ok = ctl == 0
        && matched == PATTERN.len() as u32
        && written == PATTERN.len() as u32
        && refused == 0
        && loop_flags & error_mask == 0
        && ticks >= TICKS_MIN
        && ticks <= TICKS_MAX
        && control_ticks >= TICKS_MIN
        && control_ticks <= TICKS_MAX;

    let mut record: Record<200> = Record::new();
    record
        .s(b"eusart: ok=")
        .u(ok as u32)
        .s(b" n=")
        .u(PATTERN.len() as u32)
        .s(b" am=")
        .u(matched)
        .s(b" wr=")
        .u(written)
        .s(b" err=")
        .u(refused)
        .s(b" ctl=")
        .u(ctl)
        .s(b" div=")
        .u(divider)
        .s(b" cal=")
        .u(cal_ticks)
        .s(b" ct=")
        .u(control_ticks)
        .s(b" tt=")
        .u(ticks)
        .s(b" bd=")
        .u(measured_baud)
        .s(b" if=0x")
        .h32(loop_flags)
        .s(b" ft=0x")
        .h32(flags_after_timing)
        .s(b" fc=0x")
        .h32(flags_after_clear)
        .s(b" cf=0x")
        .h32(flags_before_control);

    if ok {
        mailbox::pass(record.bytes());
    } else {
        mailbox::fail(record.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
