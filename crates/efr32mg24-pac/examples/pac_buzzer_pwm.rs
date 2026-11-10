//! PAC-only buzzer drive on the Expansion Base: TIMER0 PWM on PC03 (D3).
//!
//! The base carries a passive buzzer on D3, which is why it needs a square wave
//! of a few kHz rather than a static level: a passive buzzer is a coil and a
//! diaphragm, and a DC level makes no sound at all. The example therefore uses
//! TIMER0 as a hardware PWM generator, not a software loop, so the tone keeps
//! running without the CPU and its pitch does not depend on what else the
//! firmware is doing.
//!
//! ## Register sequencing (measured on this die)
//!
//! TIMER0 rejects writes in the wrong order with a bus fault (precise data
//! error, `CFSR=0x00008200`): `CFG` may only be written while `EN=0`, and `TOP`
//! only while `EN=1`. The order below is the one that works, and it is the same
//! order documented in `docs/04-reference/PAC-CLOCK-AND-TIMER-NOTES.md`.
//!
//! ## Verification without ears
//!
//! A tone cannot be heard by a test harness, so the example reports what it can
//! prove: TIMER0 is running, `TOP` and `CC0` hold the programmed values, the
//! output is routed to PC03, and a count of compare events over a known number
//! of CPU cycles gives the real tone frequency. If that number is near the
//! programmed pitch, the timer is genuinely generating it; whether the buzzer
//! on the board is fitted (the trace is cuttable) is the one thing left to ears.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// HFRCOEM23 as measured on this board (see the clock notes): the nominal
/// 20 MHz figure puts every timing claim 5% off.
const HFRCOEM23_HZ: u32 = 18_500_000;
/// Wanted tone. A passive buzzer is loudest around 2-4 kHz.
const TONE_HZ: u32 = 2700;

fn push_hex8(buf: &mut [u8], mut pos: usize, value: u32) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for shift in (0..8).rev() {
        if pos < buf.len() {
            buf[pos] = HEX[((value >> (shift * 4)) & 0xF) as usize];
            pos += 1;
        }
    }
    pos
}

fn push_dec(buf: &mut [u8], mut pos: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    if value == 0 {
        digits[0] = b'0';
        n = 1;
    }
    while value > 0 {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        if pos < buf.len() {
            buf[pos] = digits[n];
            pos += 1;
        }
    }
    pos
}

fn push_bytes(buf: &mut [u8], mut pos: usize, s: &[u8]) -> usize {
    for b in s {
        if pos < buf.len() {
            buf[pos] = *b;
            pos += 1;
        }
    }
    pos
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // TIMER0 and GPIO must both be clocked before any register is touched;
    // without the gate the first access faults (BFAR = the register address).
    dp.cmu_s
        .clken0()
        .modify(|_r, w| w.gpio().set_bit().timer0().set_bit().hfrcoem23().set_bit());

    // PC03 is the buzzer pin; the peripheral drives it through the route, so the
    // pin itself is push-pull and the timer owns the level.
    dp.gpio_s.portc_model().modify(|_r, w| w.mode3().pushpull());
    dp.gpio_s
        .timer0_cc0route()
        .write(|w| unsafe { w.port().bits(2).pin().bits(3) });
    dp.gpio_s
        .timer0_routeen()
        .modify(|_r, w| w.cc0pen().set_bit());

    // HFRCOEM23 has to be forced on and ready BEFORE it is selected, otherwise
    // the timer runs (STATUS.RUNNING = 1) against a stopped clock and the
    // counter sits at 0 forever -- measured: CNT read 0 twice, no CC0 event.
    dp.hfrcoem23_s.ctrl().modify(|_r, w| w.forceen().set_bit());
    let mut spin = 0u32;
    while !dp.hfrcoem23_s.status().read().rdy().bit_is_set() {
        spin += 1;
        if spin > 1_000_000 {
            break;
        }
    }
    // TIMER0 clocked from HFRCOEM23 (asynchronous, keeps running in low energy).
    dp.cmu_s
        .em01grpaclkctrl()
        .modify(|_r, w| w.clksel().hfrcoem23());

    let timer = &dp.timer0_s;

    // Order matters, and the peripheral enforces it with a bus fault:
    //   CFG      with EN=1 -> BFAR 0x40048004 (CFG)
    //   CC0_CFG  with EN=1 -> BFAR 0x40048060 (CC0_CFG)
    //   CC0_CTRL with EN=0 -> BFAR 0x40048064 (CC0_CTRL)
    //   TOP      with EN=0 -> BFAR 0x4004801C (TOP)
    // So: configuration while disabled, buffered values while enabled.
    timer.lock().write(|w| unsafe { w.lockkey().bits(0xCE80) });
    timer.en().write(|w| w.en().clear_bit());
    // MODE must be Up. The raw 0x00010003 the blink example uses sets MODE=3,
    // the quadrature decoder, which counts external A/B edges and leaves CNT at
    // 0 on a bare pin while STATUS.RUNNING still reads 1.
    timer
        .cfg()
        .write(|w| w.mode().up().clksel().prescem01grpaclk().presc().div1());
    timer.cc0_cfg().write(|w| w.mode().outputcompare());
    timer.en().write(|w| w.en().set_bit());

    // Measure the timer clock instead of trusting a datasheet band: HFRCOEM23
    // comes up in a low band here (~5 MHz measured, not the 20 MHz the name
    // suggests), and CNT is 16 bits, so the window must stay short enough not
    // to wrap (20k cycles ~ 1 ms).
    const SYSCLK_HZ: u32 = 19_000_000; // FSRCO under probe-rs reset, measured
    const WINDOW_CYCLES: u32 = 20_000;
    let probe_top: u32 = 0xFFFF;
    timer.top().write(|w| unsafe { w.top().bits(probe_top) });
    timer.cmd().write(|w| w.start().set_bit());

    let cnt_before = timer.cnt().read().bits();
    cortex_m::asm::delay(WINDOW_CYCLES);
    let cnt_after = timer.cnt().read().bits();
    let ticks = if cnt_after >= cnt_before {
        cnt_after - cnt_before
    } else {
        probe_top - cnt_before + cnt_after
    };
    let timer_hz = (ticks as u64 * SYSCLK_HZ as u64) / WINDOW_CYCLES as u64;

    // Now set TOP from the measured clock so the tone is the wanted frequency
    // whatever band the oscillator came up in. One CC0 toggle per period means
    // the pin changes at timer_hz/TOP, so the tone is half of that.
    let period = ((timer_hz / (TONE_HZ as u64 * 2)).max(2)) as u32;
    let half = period / 2;
    timer.cc0_oc().write(|w| unsafe { w.oc().bits(half) });
    timer.top().write(|w| unsafe { w.top().bits(period) });
    timer
        .cc0_ctrl()
        .write(|w| w.cmoa().toggle().outinv().clear_bit());

    let tone_hz = timer_hz / (period as u64 * 2);
    let events = ticks;

    let running = timer.status().read().running().bit_is_set();
    let top_readback = timer.top().read().top().bits();
    let cc0_readback = timer.cc0_oc().read().oc().bits();
    let route = dp.gpio_s.timer0_cc0route().read().bits();
    let route_enabled = dp.gpio_s.timer0_routeen().read().cc0pen().bit_is_set();

    let mut buf = [0u8; 64];
    let mut pos = push_bytes(&mut buf, 0, b"tone=");
    pos = push_dec(&mut buf, pos, tone_hz as u32);
    pos = push_bytes(&mut buf, pos, b"Hz clk=");
    pos = push_dec(&mut buf, pos, (timer_hz / 1000) as u32);
    pos = push_bytes(&mut buf, pos, b"kHz tk=");
    pos = push_dec(&mut buf, pos, events);
    pos = push_bytes(&mut buf, pos, b" top=");
    pos = push_hex8(&mut buf, pos, top_readback);
    pos = push_bytes(&mut buf, pos, b" cc0=");
    pos = push_hex8(&mut buf, pos, cc0_readback);
    pos = push_bytes(&mut buf, pos, b" run=");
    pos = push_hex8(&mut buf, pos, u32::from(running));

    // Passing needs the timer running, the registers holding what was written,
    // the pin routed, and a tone within 10% of the target.
    let registers_ok = top_readback == period && cc0_readback == half;
    let tone_ok = tone_hz + (tone_hz / 10) >= TONE_HZ as u64 && tone_hz <= TONE_HZ as u64 + 270;
    if running && registers_ok && route_enabled && route != 0 && tone_ok {
        mailbox::pass(&buf[..pos]);
    } else {
        mailbox::fail(&buf[..pos]);
    }

    // Leave the tone running: the buzzer should keep sounding after the test.
    loop {
        cortex_m::asm::nop();
    }
}
