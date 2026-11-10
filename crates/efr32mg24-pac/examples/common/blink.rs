//! Shared PAC-only blink patterns used by the rate/pattern examples.
//!
//! Deliberately identical in behaviour to `blink_xiao` (PA7, read-modify-write
//! toggle, NOP-loop delay) so any difference in what the LED does can be
//! attributed to the pattern parameters alone. The addition is the mailbox
//! record, which carries a toggle counter: read `seq` twice over SWD with a
//! known wall-clock gap and you get the toggle rate without a logic analyser.
//!
//! A "group" is `pulses` short ON/OFF pairs followed by one `gap` phase, i.e.
//! the shape `blink blink ______`:
//!
//! ```text
//! ON pulse OFF pulse | ON pulse OFF pulse | gap (LED off) | repeat
//! ```
//!
//! `mailbox` lives next to this file in `examples/common/`.

#![allow(dead_code)]

use efr32mg24_pac as pac;

#[path = "mailbox.rs"]
mod mailbox;

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

#[inline(never)]
fn delay(iterations: u32) {
    for _ in 0..iterations {
        cortex_m::asm::nop();
    }
}

struct Led<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
    seq: u32,
    on: bool,
    pulse_iters: u32,
    eui_hi: u32,
    eui_lo: u32,
}

impl Led<'_> {
    fn toggle(&mut self) {
        self.gpio
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
        self.on = !self.on;
        self.seq = self.seq.wrapping_add(1);

        let mut buf = [0u8; 72];
        let mut pos = push_bytes(&mut buf, 0, b"blink seq=");
        pos = push_hex8(&mut buf, pos, self.seq);
        pos = push_bytes(&mut buf, pos, b" p=");
        pos = push_hex8(&mut buf, pos, self.pulse_iters);
        pos = push_bytes(
            &mut buf,
            pos,
            if self.on {
                b" led=1 eui="
            } else {
                b" led=0 eui="
            },
        );
        pos = push_hex8(&mut buf, pos, self.eui_hi);
        pos = push_hex8(&mut buf, pos, self.eui_lo);
        mailbox::pass(&buf[..pos]);
    }
}

/// Run the pattern forever: `pulses` ON/OFF pairs, then one `gap` phase.
pub fn run_pattern(pulses: u32, pulse_iters: u32, gap_iters: u32) -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let eui_hi: u32 = dp.devinfo.eui64h().read().uniqueh().bits().into();
    let eui_lo = dp.devinfo.eui64l().read().uniquel().bits();

    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    mailbox::info(b"boot: blink pattern example");

    let mut led = Led {
        gpio: &dp.gpio_s,
        seq: 0,
        on: false,
        pulse_iters,
        eui_hi,
        eui_lo,
    };

    loop {
        for _ in 0..pulses {
            led.toggle(); // ON
            delay(pulse_iters);
            led.toggle(); // OFF
            delay(pulse_iters);
        }
        delay(gap_iters);
    }
}
