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

#![no_std]
#![no_main]

use core::cell::RefCell;

use efr32mg24_hal::board::Board;
use efr32mg24_hal::bus::Bus;
use efr32mg24_hal::bus::SharedBus;
use efr32mg24_hal::display::{append, format_fixed, Ssd1306};
use efr32mg24_hal::entry;
use efr32mg24_hal::pin::Port;
use panic_halt as _;

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
