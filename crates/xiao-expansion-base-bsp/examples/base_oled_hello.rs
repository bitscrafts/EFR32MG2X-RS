//! OLED hello: text on the Expansion Base panel, HAL only.
//!
//! The HAL twin of `pac_oled_hello`. Differences that matter:
//!
//! - the driver owns the framebuffer, so the example composes lines and calls
//!   `flush()` once instead of hand-building page transfers;
//! - the address and the init sequence live in the driver, not here.
//!
//! Verification: the panel is write-only, so the report is the number of bytes
//! the controller acknowledged plus a fixed string. The pixels need an eye; the
//! bytes do not.
//!
//! It lives in the carrier crate because the devices are on the carrier, and it talks to the
//! HAL rather than to the facade on purpose: this is the driver-level demo, so a HAL
//! regression shows up on the board, next to `base_env_clock` which goes through the facade.

#![no_std]
#![no_main]

use core::cell::RefCell;

use panic_halt as _;
use xiao_expansion_base_bsp::hal::board::Board;
use xiao_expansion_base_bsp::hal::bus::Bus;
use xiao_expansion_base_bsp::hal::bus::SharedBus;
use xiao_expansion_base_bsp::hal::display::{append, format_fixed, Ssd1306};
use xiao_expansion_base_bsp::hal::entry;
use xiao_expansion_base_bsp::hal::pin::Port;

#[path = "common/mailbox.rs"]
mod mailbox;

#[entry]
fn main() -> ! {
    let board = match Board::take() {
        Some(b) => b,
        None => loop {
            cortex_m::asm::nop();
        },
    };
    let led = board.output_on(Port::A, 7, true);
    let bus = RefCell::new(board.i2c_on(Bus::new(Port::C, 1 << 4, 1 << 5)));

    let mut oled = match Ssd1306::new(SharedBus::new(&bus)) {
        Some(o) => o,
        None => {
            mailbox::fail(b"oled ack=0 (nothing at 0x3C)");
            loop {
                led.toggle();
                cortex_m::asm::delay(2_000_000);
            }
        }
    };

    oled.clear();
    oled.text(0, 0, b"HAL SSD1306");
    oled.text(0, 3, b"PAC 0X3C OK");
    oled.text(0, 6, b"TEXT");
    oled.flush();

    let mut msg = [0u8; 48];
    let mut pos = append(&mut msg, 0, b"oled acked=");
    pos += format_fixed(&mut msg[pos..], oled.acked as i32, 0);
    if oled.acked > 500 {
        mailbox::pass(&msg[..pos]);
    } else {
        mailbox::fail(&msg[..pos]);
    }

    loop {
        led.toggle();
        cortex_m::asm::delay(2_000_000);
    }
}
