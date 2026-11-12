//! PAC-only blink on a hardware timer: TIMER0 clocked from HFRCOEM23.
//!
//! Why a timer at all: the NOP-loop blinks measured 2026-11-10 ran at ~3 Hz
//! when the debugger reset the board (FSRCO ~19 MHz) and would run 2-4x faster
//! after a power cycle, because the bootloader configures the clock tree. A
//! blink whose period depends on how the board was started is not a blink.
//!
//! Clock path:
//!
//! ```text
//! HFRCOEM23 (internal RC, independent of SYSCLK) -> EM01GRPACLK
//!   -> TIMER0 up-count, PRESC = 1024, TOP = 9765 ticks (~500 ms)
//! ```
//!
//! Measured tick rate on this die: CNT advanced 5620 counts in 300 ms, i.e.
//! ~18.7 kHz, and TOP was read back as 0x2625 (9765). One wrap is therefore
//! ~0.52 s. `measure-blink-rate.sh` is the running check on that number.
//!
//! ## Three hardware behaviours this example had to discover, all measured
//!
//! 1. **`CMU_CLKEN0.SYSRTC0` / `.TIMER0` / `.HFRCOEM23` gate the bus clock.**
//!    Touching a register of an ungated peripheral raises a precise data bus
//!    error (`CFSR=0x00008200`) and the image sits in `HardFault_`.
//! 2. **Register writes are refused in the wrong enable state.**
//!    `CFG` written with `EN=1` faults with `BFAR=0x40048004` (CFG);
//!    `TOP` written with `EN=0` faults with `BFAR=0x4004801C` (TOP).
//!    Correct order: configure CFG while disabled, enable, then write TOP.
//! 3. **`IF.OF` cannot be cleared on this part.** It reads 1 and stays 1 after
//!    writing 1 to the bit, and after writing `0xFFFFFFFF` to the register.
//!    Polling it therefore spins at CPU speed: the first version of this
//!    example toggled 21 352 times per second instead of 2. The periodic event
//!    is taken from the counter WRAP instead (`cur < prev`).
//!
//! Verified over SWD, not by eye:
//!   scripts/read-led-lnx.sh        mode7=4, DOUT toggles, DIN follows
//!   scripts/measure-blink-rate.sh  toggle counter from the mailbox

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Timer prescaler: div 1024.
const PRESC_DIV: u32 = 1024;
/// Half-period in prescaled ticks, CALIBRATED BY MEASUREMENT.
///
/// Datasheet math (HFRCOEM23 = 20 MHz / 1024 / 2) gives 9765 ticks, which
/// measured 1.92 and 1.86 toggle/s on this die -- 0.96 and 0.93 Hz, ~5% slow,
/// because the actual HFRCOEM23 is nearer 18.5 MHz than 20 MHz. Scaling by the
/// measured ratio (1.89/2.0) gives 9229. Re-measure after any change to the
/// oscillator band; `scripts/measure-blink-rate.sh` prints the number.
const TOP_TICKS: u32 = 9_229;

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

    // Bus clocks: GPIO, TIMER0, and the HFRCOEM23 oscillator.
    dp.cmu_s
        .clken0()
        .modify(|_r, w| w.gpio().set_bit().timer0().set_bit().hfrcoem23().set_bit());

    // HFRCOEM23 forced on: a consumer would otherwise keep it on demand only.
    dp.hfrcoem23_s.ctrl().modify(|_r, w| w.forceen().set_bit());
    let mut guard = 0u32;
    while !dp.hfrcoem23_s.status().read().rdy().bit_is_set() {
        guard = guard.wrapping_add(1);
        if guard > 2_000_000 {
            break;
        }
    }

    // TIMER0 draws from EM01GRPACLK; select HFRCOEM23 (not SYSCLK).
    dp.cmu_s
        .em01grpaclkctrl()
        .modify(|_r, w| w.clksel().hfrcoem23());

    // PA7 push-pull output.
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    // Order: CFG while disabled, enable, then TOP, then start.
    dp.timer0_s
        .cfg()
        .write(|w| w.mode().up().clksel().prescem01grpaclk().presc().div1024());

    dp.timer0_s.en().write(|w| w.en().set_bit());

    dp.timer0_s
        .top()
        .write(|w| unsafe { w.top().bits(TOP_TICKS) });

    dp.timer0_s.cmd().write(|w| w.start().set_bit());

    let mut msg = [0u8; 56];
    let mut pos = push_bytes(&mut msg, 0, b"boot: timer0 blink top=");
    pos = push_hex8(&mut msg, pos, TOP_TICKS);
    pos = push_bytes(&mut msg, pos, b" presc=");
    pos = push_hex8(&mut msg, pos, PRESC_DIV);
    mailbox::info(&msg[..pos]);

    let mut seq: u32 = 0;
    let mut led_on = false;
    let mut prev_cnt = dp.timer0_s.cnt().read().cnt().bits();

    loop {
        // One full period = the counter wrapped (see note 3 above).
        let cur_cnt = dp.timer0_s.cnt().read().cnt().bits();
        if cur_cnt < prev_cnt {
            dp.gpio_s
                .porta_dout()
                .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
            led_on = !led_on;
            seq = seq.wrapping_add(1);

            let mut buf = [0u8; 64];
            let mut pos = push_bytes(&mut buf, 0, b"timer0 seq=");
            pos = push_hex8(&mut buf, pos, seq);
            pos = push_bytes(
                &mut buf,
                pos,
                if led_on {
                    b" led=1 top="
                } else {
                    b" led=0 top="
                },
            );
            pos = push_hex8(&mut buf, pos, TOP_TICKS);
            mailbox::pass(&buf[..pos]);
        }
        prev_cnt = cur_cnt;
    }
}
