//! PAC-only blink that also reports itself over SWD.
//!
//! Same LED as `blink_xiao` (PA7, active low) plus two things that make the
//! test checkable without a human watching the board:
//!
//! 1. every toggle writes a record to the mailbox at 0x2003_F000
//!    ([`common/mailbox.rs`](common/mailbox.rs)), read back with
//!    `scripts/read-mailbox.sh`;
//! 2. the record carries the chip's factory EUI-64 (`DEVINFO.EUI64H/L`), so a
//!    record read from RAM proves *this* die executed the image — not that some
//!    stale RAM happens to hold a plausible string.
//!
//! What it proves when it runs: PAC typed accessors, GPIO clock gate, pin mode,
//! DOUT toggle, cortex-m-rt entry, the linker map at 0x08006000, and the SWD
//! read path. What it does not prove: that the LED is visible, or that the
//! delay is calibrated (it is a NOP loop, not a timer).
//!
//! Flashing is deliberately not described here — use `scripts/flash-lnx.sh`,
//! which refuses images that would touch the Gecko bootloader.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Append `value` as 8 lowercase hex digits to `buf` at `pos`; returns the new
/// position. Hand-rolled because pulling in `core::fmt` for one integer is not
/// worth the code size on a 1.5 MiB part with a NOP-loop delay.
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

/// Delay loop: roughly a few million cycles per call. Not calibrated; the
/// point is that the LED changes state, not that it does so at exactly 1 Hz.
#[inline(never)]
fn delay() {
    for _ in 0..1_500_000 {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Factory EUI-64: unique per die, read before any radio or flash operation.
    let eui_hi = dp.devinfo.eui64h().read().uniqueh().bits();
    let eui_lo = dp.devinfo.eui64l().read().uniquel().bits();

    // GPIO clock gate (CMU_S.CLKEN0.GPIO, bit 26), then PA7 as push-pull output.
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    mailbox::info(b"boot: pac_mailbox_blink");

    let mut seq: u32 = 0;
    let mut led_on = false;

    loop {
        // Toggle PA7. There is no DOUTTGL register on this part: read-modify-write.
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
        led_on = !led_on;
        seq = seq.wrapping_add(1);

        let mut buf = [0u8; 64];
        let mut pos = push_bytes(&mut buf, 0, b"blink seq=");
        pos = push_hex8(&mut buf, pos, seq);
        pos = push_bytes(
            &mut buf,
            pos,
            if led_on {
                b" led=1 eui="
            } else {
                b" led=0 eui="
            },
        );
        pos = push_hex8(&mut buf, pos, eui_hi.into());
        pos = push_hex8(&mut buf, pos, eui_lo);

        mailbox::pass(&buf[..pos]);

        delay();
    }
}
