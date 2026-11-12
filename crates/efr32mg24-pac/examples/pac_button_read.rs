//! PAC-only read of the Expansion Base user button on PC01 (XIAO pin D1).
//!
//! The base wires its user button to D1 with the other side to ground, so the
//! pin is configured as an input with an internal pull-up: pressed reads 0,
//! released reads 1. Every transition is written to the mailbox with its state
//! and a counter, so a press can be confirmed from the host over SWD without
//! anyone watching the board -- and without a timer or an interrupt.
//!
//! Note: the button lives on the Expansion Base. On a bare XIAO the pin floats
//! to the pull-up and the image will simply report `idle` forever, which is
//! itself the diagnostic (it does not hang).
//!
//! Verified over SWD: `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
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

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // PC01: input with pull-up. MODE field 2 = "Input enabled, DOUT determines
    // pull direction", so DOUT bit 1 = 1 selects the pull-up.
    dp.gpio_s
        .portc_model()
        .modify(|_r, w| w.mode1().inputpull());
    dp.gpio_s
        .portc_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 1)) });

    // PA7 stays an output so the LED can echo the button.
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    let mut pressed_seen = 0u32;
    let mut reported_pressed = false;
    let mut settled = 0u32;

    mailbox::info(b"boot: button on PC01 (pull-up)");

    loop {
        let raw = dp.gpio_s.portc_din().read().bits();
        let pressed = (raw & (1 << 1)) == 0;

        // Debounce: require the same reading four passes in a row before acting.
        if pressed == reported_pressed {
            settled = 0;
        } else {
            settled += 1;
            if settled >= 4 {
                reported_pressed = pressed;
                settled = 0;

                if pressed {
                    pressed_seen = pressed_seen.wrapping_add(1);
                    // LED on while pressed (active low).
                    dp.gpio_s
                        .porta_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 7)) });
                } else {
                    dp.gpio_s
                        .porta_dout()
                        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 7)) });
                }

                let mut buf = [0u8; 48];
                let mut pos = push_bytes(&mut buf, 0, b"button ");
                pos = push_bytes(
                    &mut buf,
                    pos,
                    if pressed {
                        b"pressed n="
                    } else {
                        b"released n="
                    },
                );
                pos = push_hex8(&mut buf, pos, pressed_seen);
                pos = push_bytes(&mut buf, pos, b" din=");
                pos = push_hex8(&mut buf, pos, raw);
                mailbox::pass(&buf[..pos]);
            }
        }
    }
}
