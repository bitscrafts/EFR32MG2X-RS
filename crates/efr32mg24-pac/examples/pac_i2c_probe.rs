//! PAC-only I2C0 probe of the Expansion Base devices, following the vendor
//! state machine byte for byte.
//!
//! ## What was wrong before (and why this file exists)
//!
//! Three separate mistakes kept the on-chip master from completing a read, and
//! each one was invisible from the outside:
//!
//! 1. **`TXDATA` must be written *before* `CMD.START`.** `em_i2c.c` says it in
//!    one comment: *"Data not transmitted until the START is sent."* Writing
//!    START first then TXDATA leaves the byte in the FIFO with the bus already
//!    started, so nothing is on the wire and `STATE` parks in START forever.
//! 2. **ACK/NACK is reported in `IF`, not in `STATUS.TXC`.** `TXC` means "the
//!    byte shifted out", which happens for a NACKed address too. Accepting TXC
//!    as success reported every address on the bus as present (`mask=0f`).
//! 3. **A one-byte read must arm `CMD.NACK` *before* the repeated START**,
//!    otherwise the master keeps the bus for another byte and never lets the
//!    transaction close.
//!
//! The sequence below is the same order as `I2C_Transfer`'s
//! `i2cStateStartAddrSend` -> `i2cStateAddrWFAckNack` -> `i2cStateWFData`.
//!
//! Reports: which of 0x3C/0x51/0x76/0x77 answer, the BME680 chip id (must be
//! 0x61) and the PCF8563 seconds/minutes. A device that answers but cannot be
//! read is reported as such instead of being counted as healthy.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Raw engine registers captured at the last data-phase timeout.
static mut DIAG: [u32; 3] = [0; 3];

/// Upper bound for any single peripheral poll (~50 ms at 19 MHz).
const SPIN_LIMIT: u32 = 300_000;

const ADDR_OLED: u8 = 0x3C;
const ADDR_RTC: u8 = 0x51;
const ADDR_BME_A: u8 = 0x76;
const ADDR_BME_B: u8 = 0x77;

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

fn push_hex2(buf: &mut [u8], mut pos: usize, value: u8) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    if pos < buf.len() {
        buf[pos] = HEX[(value >> 4) as usize];
        pos += 1;
    }
    if pos < buf.len() {
        buf[pos] = HEX[(value & 0xF) as usize];
        pos += 1;
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

type I2c = pac::i2c0_s::RegisterBlock;

/// Configure PC04/PC05 as open-drain I2C pins routed to I2C0, then enable it.
fn init(i2c: &I2c, gpio: &pac::gpio_s::RegisterBlock) {
    // Open drain: the bus is driven low or released, never pushed high.
    gpio.portc_model()
        .modify(|_r, w| w.mode4().wiredand().mode5().wiredand());
    gpio.portc_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 4) | (1 << 5)) });

    // SCL on PC05, SDA on PC04 (port field: 0=A, 1=B, 2=C, 3=D).
    gpio.i2c0_sclroute()
        .write(|w| unsafe { w.port().bits(2).pin().bits(5) });
    gpio.i2c0_sdaroute()
        .write(|w| unsafe { w.port().bits(2).pin().bits(4) });
    gpio.i2c0_routeen()
        .modify(|_r, w| w.sclpen().set_bit().sdapen().set_bit());

    i2c.en().write(|w| w.en().set_bit());
    // Master mode, standard clock ratio, AUTOACK off: every received byte is
    // acknowledged deliberately, which is what the vendor driver relies on.
    i2c.ctrl().write(|w| w.slave().disable().clhr().standard());
    // 100 kHz from a 19 MHz HFCLK: CLKDIV = f/(8*scl) - 1
    i2c.clkdiv().write(|w| unsafe { w.div().bits(22) });
    // emlib enables these interrupts and then polls the same flags; some
    // series-2 blocks only update the IF bits that are enabled.
    i2c.ien().write(|w| unsafe { w.bits(0xFFFF_FFFF) });
}

/// Clear pending flags and the TX FIFO so the next byte is the first byte sent.
/// Full engine re-init: disable, re-enable, restore CTRL/CLKDIV, clear flags.
///
/// Measured: flag clearing alone is not enough — the transfer *after* any
/// completed read reported a NACK on its own read address, while the same read
/// on a freshly enabled engine returned a byte. Disabling the block returns it
/// to the state it has at boot, which is the only state this die has shown to
/// be reliable for consecutive transfers.
fn hard_reset(i2c: &I2c) {
    i2c.en().write(|w| w.en().clear_bit());
    i2c.en().write(|w| w.en().set_bit());
    i2c.ctrl().write(|w| w.slave().disable().clhr().standard());
    i2c.clkdiv().write(|w| unsafe { w.div().bits(22) });
    i2c.ien().write(|w| unsafe { w.bits(0xFFFF_FFFF) });
    reset_engine(i2c);
}

fn reset_engine(i2c: &I2c) {
    // Same order as `I2C_TransferInit`: abort if the engine is mid-transfer,
    // empty the FIFOs, then clear every flag. Without the abort a transfer
    // that failed in its data phase leaves the engine BUSY and the next START
    // is silently ignored -- which is why the transfer *after* a failure always
    // reported step 1.
    if i2c.state().read().busy().bit_is_set() {
        i2c.cmd().write(|w| w.abort().set_bit());
        let mut spin = 0u32;
        while i2c.state().read().busy().bit_is_set() {
            spin += 1;
            if spin > SPIN_LIMIT {
                break;
            }
        }
    }
    i2c.cmd()
        .write(|w| w.clearpc().set_bit().cleartx().set_bit());
    // flush the RX FIFO
    while i2c.status().read().rxdatav().bit_is_set() {
        let _ = i2c.rxdata().read().bits();
    }
    // Clear *every* flag, as `I2C_IntClear(i2c, _I2C_IF_MASK)` does. Clearing
    // only ACK/NACK/RXDATAV/RXFULL/TXC left ADDR, RSTART, START, MSTOP and the
    // error flags to leak into the next transfer -- measured: the first read
    // on a fresh engine returned a byte, every read after other traffic
    // reported a NACK on the read address.
    i2c.if_().write(|w| unsafe { w.bits(0xFFFF_FFFF) });
}

/// Wait for ACK or NACK after an address/data byte. `None` on timeout.
fn wait_ack(i2c: &I2c) -> Option<bool> {
    let mut spin = 0u32;
    loop {
        let flags = i2c.if_().read();
        if flags.nack().bit_is_set() {
            i2c.if_().write(|w| w.nack().set_bit());
            return Some(false);
        }
        if flags.ack().bit_is_set() {
            i2c.if_().write(|w| w.ack().set_bit());
            return Some(true);
        }
        spin += 1;
        if spin > SPIN_LIMIT {
            return None;
        }
    }
}

fn stop(i2c: &I2c) {
    i2c.cmd().write(|w| w.stop().set_bit());
    // Wait for the STOP to actually leave the bus and for the leader-stop flag,
    // then make sure the engine is idle before the next transfer starts.
    let mut spin = 0u32;
    while i2c.status().read().pstop().bit_is_set() {
        spin += 1;
        if spin > SPIN_LIMIT {
            break;
        }
    }
    let mut spin = 0u32;
    while i2c.state().read().busy().bit_is_set() {
        spin += 1;
        if spin > SPIN_LIMIT {
            i2c.cmd().write(|w| w.abort().set_bit());
            break;
        }
    }
    // STATUS.PSTART/PSTOP clear while the condition is still being generated;
    // IF.MSTOP is the flag that says the STOP reached the bus (emlib waits for
    // exactly this in i2cStateWFStopSent). Leaving before it lands is what left
    // the slave mid-transaction, so the next write-addressed transfer was
    // NACKed by a device that was still expecting data.
    let mut spin = 0u32;
    while !i2c.if_().read().mstop().bit_is_set() {
        spin += 1;
        if spin > SPIN_LIMIT {
            break;
        }
    }
    i2c.if_().write(|w| w.mstop().set_bit());
}

/// First START + address: TXDATA first, then START.
fn start_addr(i2c: &I2c, addr: u8, read: bool) -> Option<bool> {
    let byte = (addr << 1) | u8::from(read);
    i2c.txdata().write(|w| unsafe { w.txdata().bits(byte) });
    i2c.cmd().write(|w| w.start().set_bit());
    wait_ack(i2c)
}

/// Repeated START + address: **START first, then TXDATA**.
///
/// The order is the opposite of the first START, and getting it wrong is
/// silent: with TXDATA written first the address byte goes out as data of the
/// preceding write phase, the device NACKs it, and the read never starts --
/// which is what every "step 6" timeout was.
fn restart_addr_read_one(i2c: &I2c, addr: u8) -> Option<bool> {
    i2c.cmd().write(|w| w.start().set_bit());
    i2c.txdata()
        .write(|w| unsafe { w.txdata().bits((addr << 1) | 1) });
    let acked = wait_ack(i2c);
    if matches!(acked, Some(true)) {
        // One byte only: NACK the data byte so the transaction can close.
        i2c.cmd().write(|w| w.nack().set_bit());
    }
    acked
}

fn send_byte(i2c: &I2c, byte: u8) -> Option<bool> {
    i2c.txdata().write(|w| unsafe { w.txdata().bits(byte) });
    wait_ack(i2c)
}

/// Read one byte with no register pointer and no repeated START: just
/// START + address+R + one byte. This isolates the two candidates -- a wrong
/// restart sequence, or the read direction never being accepted.
fn read_plain(i2c: &I2c, addr: u8) -> (u8, u8) {
    hard_reset(i2c);
    i2c.txdata()
        .write(|w| unsafe { w.txdata().bits((addr << 1) | 1) });
    i2c.cmd().write(|w| w.start().set_bit());
    match wait_ack(i2c) {
        Some(true) => {}
        Some(false) => {
            stop(i2c);
            return (2, 0);
        }
        None => {
            stop(i2c);
            return (1, 0);
        }
    }
    i2c.cmd().write(|w| w.nack().set_bit());

    let mut spin = 0u32;
    loop {
        let status = i2c.status().read();
        if status.rxdatav().bit_is_set() {
            break;
        }
        if status.rxfull().bit_is_set() {
            let _ = i2c.rxdata().read().bits();
            continue;
        }
        spin += 1;
        if spin > SPIN_LIMIT {
            stop(i2c);
            return (6, 0);
        }
    }
    let value = i2c.rxdata().read().bits() as u8;
    i2c.if_().write(|w| w.rxdatav().set_bit());
    stop(i2c);
    (0, value)
}

/// Does anything answer at this address?
fn probe(i2c: &I2c, addr: u8) -> bool {
    hard_reset(i2c);
    let acked = start_addr(i2c, addr, false).unwrap_or(false);
    stop(i2c);
    acked
}

/// Read one 8-bit register: write the pointer, repeated START, read one byte.
///
/// Steps, as in `I2C_Transfer`: 1 = START, 2 = address+W, 3 = register,
/// 4 = repeated START, 5 = address+R, 6 = data. 0 means success.
fn read_reg(i2c: &I2c, addr: u8, reg: u8) -> (u8, u8) {
    hard_reset(i2c);

    if !matches!(start_addr(i2c, addr, false), Some(true)) {
        stop(i2c);
        return (1, 0);
    }
    if !matches!(send_byte(i2c, reg), Some(true)) {
        stop(i2c);
        return (3, 0);
    }

    if !matches!(restart_addr_read_one(i2c, addr), Some(true)) {
        stop(i2c);
        return (5, 0);
    }

    // Errata I2C_E303 (Series 2): the byte can arrive with RXFULL set and
    // RXDATAV clear. The documented workaround is one dummy read of RXDATA,
    // which restores RXDATAV=1 / RXFULL=0. Waiting only for RXDATAV is what
    // made every read time out at step 6.
    let mut spin = 0u32;
    loop {
        let status = i2c.status().read();
        if status.rxdatav().bit_is_set() {
            break;
        }
        if status.rxfull().bit_is_set() {
            // dummy read per the errata, then the flag pair is consistent
            let _ = i2c.rxdata().read().bits();
            i2c.if_().write(|w| w.rxuf().set_bit());
            continue;
        }
        spin += 1;
        if spin > SPIN_LIMIT {
            unsafe {
                DIAG[0] = i2c.state().read().bits();
                DIAG[1] = i2c.status().read().bits();
                DIAG[2] = i2c.if_().read().bits();
            }
            stop(i2c);
            return (6, 0);
        }
    }
    let value = i2c.rxdata().read().bits() as u8;
    i2c.if_().write(|w| w.rxdatav().set_bit());
    stop(i2c);
    (0, value)
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s
        .clken0()
        .modify(|_r, w| w.gpio().set_bit().i2c0().set_bit());

    let i2c = &dp.i2c0_s;
    init(i2c, &dp.gpio_s);

    // Isolation test: the same read, once as the very first transaction on a
    // freshly enabled engine and once after a burst of other traffic. If only
    // the second one fails, the engine is left dirty by whatever ran before.
    let (first_step, first_val) = read_plain(i2c, ADDR_BME_A);
    // Three reads in a row with no writes in between: if the first is fine and
    // the rest are not, the contamination comes from a *read*; if all three
    // pass, it comes from a write.
    let (second_step, second_val) = read_plain(i2c, ADDR_BME_A);
    let (third_step, third_val) = read_plain(i2c, ADDR_BME_A);

    let mut mask = 0u32;
    for (bit, addr) in [ADDR_OLED, ADDR_RTC, ADDR_BME_A, ADDR_BME_B]
        .iter()
        .enumerate()
    {
        if probe(i2c, *addr) {
            mask |= 1 << bit;
        }
    }

    // BME680 chip id at 0xD0 must read 0x61.
    let bme_addr = if mask & 0b0100 != 0 {
        Some(ADDR_BME_A)
    } else if mask & 0b1000 != 0 {
        Some(ADDR_BME_B)
    } else {
        None
    };
    let (bme_step, bme_id) = match bme_addr {
        Some(a) => read_reg(i2c, a, 0xD0),
        None => (0xFF, 0u8),
    };

    // PCF8563 seconds (0x02) and minutes (0x03), both BCD.
    let (rtc_step, rtc_sec, rtc_min) = if mask & 0b0010 != 0 {
        let (s1, sec) = read_reg(i2c, ADDR_RTC, 0x02);
        let (s2, min) = read_reg(i2c, ADDR_RTC, 0x03);
        (s1.max(s2), sec, min)
    } else {
        (0xFF, 0u8, 0u8)
    };

    let mut buf = [0u8; 96];
    let mut pos = push_bytes(&mut buf, 0, b"mask=");
    pos = push_hex8(&mut buf, pos, mask);
    pos = push_bytes(&mut buf, pos, b" bme=");
    pos = push_hex2(&mut buf, pos, bme_step);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, bme_id);
    pos = push_bytes(&mut buf, pos, b" rtc=");
    pos = push_hex2(&mut buf, pos, rtc_step);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, rtc_sec & 0x7F);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, rtc_min & 0x7F);

    // Control: an address with nothing behind it must NACK. If it "ACKs", the
    // master is reading SDA as a constant low and every ACK above is fiction.
    let dummy_acks = probe(i2c, 0x55);

    let txbuf_after_reset = {
        reset_engine(i2c);
        i2c.status().read().txbufcnt().bits()
    };

    let (plain_step, plain_val) = if mask & 0b1000 != 0 {
        read_plain(i2c, ADDR_BME_B)
    } else {
        (0xFE, 0u8)
    };

    let diag = unsafe { DIAG };
    pos = push_bytes(&mut buf, pos, b" first=");
    pos = push_hex2(&mut buf, pos, first_step);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, first_val);
    pos = push_bytes(&mut buf, pos, b" r=");
    pos = push_hex2(&mut buf, pos, second_step);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, second_val);
    pos = push_bytes(&mut buf, pos, b"/");
    pos = push_hex2(&mut buf, pos, third_step);
    pos = push_bytes(&mut buf, pos, b" ctl=");
    pos = push_hex2(&mut buf, pos, u8::from(dummy_acks));
    pos = push_bytes(&mut buf, pos, b",");
    pos = push_hex2(&mut buf, pos, txbuf_after_reset);
    pos = push_bytes(&mut buf, pos, b" plain=");
    pos = push_hex2(&mut buf, pos, plain_step);
    pos = push_bytes(&mut buf, pos, b"/0x");
    pos = push_hex2(&mut buf, pos, plain_val);
    pos = push_bytes(&mut buf, pos, b" st=");
    pos = push_hex8(&mut buf, pos, diag[0]);
    pos = push_bytes(&mut buf, pos, b",");
    pos = push_hex8(&mut buf, pos, diag[1]);
    pos = push_bytes(&mut buf, pos, b",");
    pos = push_hex8(&mut buf, pos, diag[2]);

    if bme_step == 0 && bme_id == 0x61 && rtc_step == 0 {
        mailbox::pass(&buf[..pos]);
    } else {
        mailbox::fail(&buf[..pos]);
    }

    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());
    loop {
        for _ in 0..2_000_000 {
            cortex_m::asm::nop();
        }
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
    }
}
