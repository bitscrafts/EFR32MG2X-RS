//! USART on the header pads: a jumper D6–D7 for the on-board proof, or an adapter for an outside witness.
//!
//! ## The two ways to watch a UART
//!
//! A transmitter talking to nothing looks exactly like one talking to a listener, so this example is built to be
//! judged from outside itself, in whichever of two ways the bench allows:
//!
//! - **A jumper between D6 (PC6, TX) and D7 (PC7, RX).** Six bytes go out, the receiver samples them back, and the
//!   verdict compares them one by one. It proves the pins, the routing, the divider, the shifter and the receiver —
//!   everything except that a *different* machine can read it.
//! - **A USB-serial adapter** (`MG24 D6 -> adapter RXD`, `D7 <- adapter TXD`, GND common). Then the bytes the adapter
//!   reads prove TX and the bytes it sends come back echoed, which proves RX. That is the stronger record, and the
//!   same firmware serves both: the loopback test fails without the jumper, and the example then behaves as an echo.
//!
//! D6 and D7 are the pads Seeed's own board support exposes as the header serial. Routing them to USART0 is a
//! crossbar choice, not a restriction of the part.
//!
//! ## Measured: a floating RX line transmits
//!
//! 2026-11-25, with D6/D7 left unconnected and the receiver enabled, this example reported `usart: echo n=271 last=00`
//! within seconds: the input, with no pull and nothing driving it, hands back a byte of `0x00` often enough to look
//! like a partner. Any echo test that counts bytes therefore passes on a board with nothing attached. That is why the
//! verdict here is the loopback comparison and not the byte count, and why the record says `last=00` when it is noise.
//!
//! ## Why the banner goes out last
//!
//! With the jumper fitted, anything transmitted comes straight back into the receiver. Sending a banner first would
//! fill the loopback comparison with the banner's own bytes, so the order here is: drain, test, then talk.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::usart::Config;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// 8N1 at the rate every adapter defaults to.
const BAUDRATE: u32 = 115_200;
/// The bytes the loopback has to return, in order. Two distinct values, so a dead line returning zeros cannot pass.
const PATTERN: &[u8] = b"EFR32!";
/// How long to give the receiver, in SYSRTC ticks (32.768 kHz): a quarter of a second, which is ~1400 byte times at
/// 115200 and therefore generous by three orders of magnitude.
const WINDOW_TICKS: u32 = 8_192;
/// Text for a listener that attached late.
const BANNER: &[u8] = b"EFR32MG24 USART0 @115200 - echo\r\n";

struct Line {
    buf: [u8; 72],
    len: usize,
}

impl Line {
    fn new() -> Self {
        Self {
            buf: [0; 72],
            len: 0,
        }
    }
    fn s(&mut self, text: &[u8]) {
        for b in text {
            if self.len < self.buf.len() {
                self.buf[self.len] = *b;
                self.len += 1;
            }
        }
    }
    fn u(&mut self, mut v: u32) {
        if v == 0 {
            self.s(b"0");
            return;
        }
        let mut digits = [0u8; 10];
        let mut n = 0;
        while v > 0 {
            digits[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            let d = digits[n];
            self.s(&[d]);
        }
    }
    fn h(&mut self, v: u8) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        self.s(&[HEX[(v >> 4) as usize], HEX[(v & 0xF) as usize]]);
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = Board::take().expect("peripherals taken twice");
    let rtc = board.sysrtc();

    // Route first, build second: a driver enabled with its pins unrouted drives nothing while looking alive.
    if board.route_usart0((Port::C, 6), (Port::C, 7)).is_err() {
        mailbox::fail(b"usart: the board refused the pin pair");
        loop {
            core::hint::spin_loop();
        }
    }

    let mut usart = match board.usart0(Config::new(BAUDRATE)) {
        Ok(usart) => usart,
        Err(_) => {
            mailbox::fail(b"usart: the driver refused the configuration");
            loop {
                core::hint::spin_loop();
            }
        }
    };

    // Anything already in the receiver belongs to something that happened before this test.
    while usart.read_byte().is_some() {}

    // The six bytes out.
    for byte in PATTERN {
        usart.write_byte(*byte);
    }
    usart.flush();

    // And back, compared as they arrive.
    let mut volta = [0u8; 6];
    let mut recebidos = 0usize;
    let mut iguais = 0usize;
    let t0 = rtc.now();
    while recebidos < PATTERN.len() && rtc.now().wrapping_sub(t0) < WINDOW_TICKS {
        if let Some(byte) = usart.read_byte() {
            volta[recebidos] = byte;
            if byte == PATTERN[recebidos] {
                iguais += 1;
            }
            recebidos += 1;
        }
    }

    let mut l = Line::new();
    l.s(b"usart: baud=");
    l.u(BAUDRATE);
    l.s(b" out=");
    l.u(PATTERN.len() as u32);
    l.s(b" in=");
    l.u(recebidos as u32);
    l.s(b" ok=");
    l.u(iguais as u32);
    if recebidos > 0 {
        l.s(b" first=");
        l.h(volta[0]);
    }

    if recebidos == PATTERN.len() && iguais == PATTERN.len() {
        // The jumper is fitted and the loop closed. Record, then stay quiet on purpose: an echo here would feed
        // itself through the jumper forever.
        mailbox::pass(&l.buf[..l.len]);
        usart.write(BANNER);
        usart.flush();
        loop {
            core::hint::spin_loop();
        }
    }

    // No jumper: this is an echo, and something outside the board has to be talking. The record says so, so a
    // harness reading `in=0` cannot mistake "nobody attached" for "the UART is broken".
    l.s(b" mode=echo");
    mailbox::fail(&l.buf[..l.len]);
    usart.write(BANNER);
    usart.flush();

    let mut eco: u32 = 0;
    loop {
        if let Some(byte) = usart.read_byte() {
            usart.write_byte(byte);
            usart.flush();
            eco += 1;

            let mut l = Line::new();
            l.s(b"usart: echo n=");
            l.u(eco);
            l.s(b" last=");
            l.h(byte);
            mailbox::pass(&l.buf[..l.len]);
        }
    }
}
