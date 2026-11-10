//! PAC-only BME680/BME688 read on the Expansion Base I2C bus (0x76 / 0x77).
//!
//! ## Why the first version reported -1 C
//!
//! Three things were wrong, all of them learned by reading the working
//! `crates/bme688` implementation in `/projects/t-dongle-c5`, which runs the
//! same sensor family on real hardware:
//!
//! 1. **par_t1 is not at 0x8A.** Bosch's coefficient array starts at 0x8A but
//!    its first bytes are `par_t2` (index 0/1) and `par_t3` (index 2);
//!    `par_t1` sits at index 31/32, which lands inside the *second* block at
//!    0xE9/0xEA. Reading 0x8A as par_t1 therefore fed par_t2 into the T1 slot
//!    and the compensation came out nonsensical (0x9467 -> -1.01 C).
//! 2. **The field block starts at 0x1D, not 0x22.** Byte 0 is the status byte
//!    whose bit 7 is `new_data`, then press_msb/lsb/xlsb at offsets 2..5 and
//!    temp_msb/lsb/xlsb at 5..8. Reading from 0x22 and treating the first three
//!    bytes as pressure shifted everything by three bytes.
//! 3. **A mode change is only accepted from sleep.** Bosch's `set_op_mode`
//!    contract is explicit about it; skipping the sleep step makes the mode
//!    change silently ignored, which is why `ctrl_meas` read back as 0x24
//!    (sleep) right after being written as 0x25 (forced).
//!
//! ## Verification
//!
//! The report has to be physically meaningful: chip id 0x61, a variant byte
//! (0x00 = BME680, 0x01 = BME688), a temperature a room can have, and a
//! pressure near 1013 hPa at sea level. Any of those failing is reported as a
//! failure with the raw numbers attached, never smoothed into a plausible
//! looking value.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/i2c_bitbang.rs"]
mod i2c_bitbang;
#[path = "common/mailbox.rs"]
mod mailbox;

const REG_STATUS: u8 = 0x1D; // also FIELD0: status + press + temp + hum + gas
const REG_CTRL_MEAS: u8 = 0x74;
const REG_VARIANT_ID: u8 = 0xF0;
const REG_CHIP_ID: u8 = 0xD0;
const REG_COEFF1: u8 = 0x8A; // 23 bytes
const REG_COEFF2: u8 = 0xE1; // 14 bytes
const REG_COEFF3: u8 = 0x00; // 5 bytes
const NEW_DATA: u8 = 0x80;

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

/// The 42 calibration bytes, concatenated in Bosch's order.
struct Calib {
    par_t1: u16,
    par_t2: i16,
    par_t3: i8,
    par_p1: u16,
    par_p2: i16,
    par_p3: i8,
    par_p4: i16,
    par_p5: i16,
    par_p6: i8,
    par_p7: i8,
    par_p8: i16,
    par_p9: i16,
    par_p10: i8,
}

fn be16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());
    let bus = i2c_bitbang::Bus::new(&dp.gpio_s);

    let addr = if bus.probe(0x76) {
        0x76u8
    } else if bus.probe(0x77) {
        0x77u8
    } else {
        mailbox::fail(b"bme68x ack=0 (nothing at 0x76/0x77)");
        idle(&dp);
    };

    let chip_id = bus.read_reg(addr, REG_CHIP_ID).unwrap_or(0);
    let variant = bus.read_reg(addr, REG_VARIANT_ID).unwrap_or(0xFF);
    if chip_id != 0x61 {
        let mut buf = [0u8; 64];
        let mut pos = push_bytes(&mut buf, 0, b"id=0x");
        pos = push_hex8(&mut buf, pos, chip_id as u32);
        pos = push_bytes(&mut buf, pos, b" (not 0x61)");
        mailbox::fail(&buf[..pos]);
        idle(&dp);
    }

    // Three calibration blocks, concatenated exactly as Bosch lays them out.
    let mut coeff = [0u8; 42];
    if !bus.read_regs(addr, REG_COEFF1, &mut coeff[..23])
        || !bus.read_regs(addr, REG_COEFF2, &mut coeff[23..37])
        || !bus.read_regs(addr, REG_COEFF3, &mut coeff[37..42])
    {
        mailbox::fail(b"calibration read failed");
        idle(&dp);
    }
    let c = Calib {
        par_t2: be16(coeff[1], coeff[0]) as i16,
        par_t3: coeff[2] as i8,
        par_p1: be16(coeff[5], coeff[4]),
        par_p2: be16(coeff[7], coeff[6]) as i16,
        par_p3: coeff[8] as i8,
        par_p4: be16(coeff[11], coeff[10]) as i16,
        par_p5: be16(coeff[13], coeff[12]) as i16,
        par_p6: coeff[15] as i8,
        par_p7: coeff[14] as i8,
        par_p8: be16(coeff[19], coeff[18]) as i16,
        par_p9: be16(coeff[21], coeff[20]) as i16,
        par_p10: coeff[22] as i8,
        par_t1: be16(coeff[32], coeff[31]),
    };

    // Bosch's set_op_mode: sleep first, then the new mode. Without the sleep
    // step the write is accepted by the bus and ignored by the sensor.
    let ctrl = bus.read_reg(addr, REG_CTRL_MEAS).unwrap_or(0);
    if !bus.write_reg(addr, REG_CTRL_MEAS, ctrl & !0x03) {
        mailbox::fail(b"sleep write failed");
        idle(&dp);
    }
    // osrs_t = x1 (001), osrs_p = x1 (001), mode = forced (01)
    if !bus.write_reg(addr, REG_CTRL_MEAS, 0b001_001_01) {
        mailbox::fail(b"forced-mode write failed");
        idle(&dp);
    }

    // A single x1/x1 conversion is ~10 ms; give it 40 ms and poll.
    cortex_m::asm::delay(200_000);
    let mut fresh = false;
    for _ in 0..20 {
        if let Some(status) = bus.read_reg(addr, REG_STATUS) {
            if status & NEW_DATA != 0 {
                fresh = true;
                break;
            }
        }
        cortex_m::asm::delay(50_000);
    }

    // Field block: [0]=status, [2..5]=pressure, [5..8]=temperature.
    let mut field = [0u8; 17];
    let read_ok = bus.read_regs(addr, REG_STATUS, &mut field);
    let press_adc = ((field[2] as u32) << 12) | ((field[3] as u32) << 4) | ((field[4] as u32) >> 4);
    let temp_adc = ((field[5] as u32) << 12) | ((field[6] as u32) << 4) | ((field[7] as u32) >> 4);

    // Bosch compensation, float form (no libm needed: only +, -, *, /).
    let t1 = c.par_t1 as f32;
    let t2 = c.par_t2 as f32;
    let t3 = c.par_t3 as f32;
    let adc = temp_adc as f32;
    let var1 = ((adc / 16384.0) - (t1 / 1024.0)) * t2;
    let dt = (adc / 131072.0) - (t1 / 8192.0);
    let var2 = dt * dt * (t3 * 16.0);
    let t_fine = var1 + var2;
    let temp_c = t_fine / 5120.0;

    let p1 = c.par_p1 as f32;
    let p2 = c.par_p2 as f32;
    let p3 = c.par_p3 as f32;
    let p4 = c.par_p4 as f32;
    let p5 = c.par_p5 as f32;
    let p6 = c.par_p6 as f32;
    let p7 = c.par_p7 as f32;
    let p8 = c.par_p8 as f32;
    let p9 = c.par_p9 as f32;
    let p10 = c.par_p10 as f32;
    let mut pv1 = (t_fine / 2.0) - 64000.0;
    let mut pv2 = pv1 * pv1 * (p6 / 131072.0);
    pv2 += pv1 * p5 * 2.0;
    pv2 = (pv2 / 4.0) + (p4 * 65536.0);
    pv1 = (((p3 * pv1 * pv1) / 16384.0) + (p2 * pv1)) / 524288.0;
    pv1 = (1.0 + (pv1 / 32768.0)) * p1;
    let press_pa = if pv1 == 0.0 {
        0.0
    } else {
        let mut calc = 1048576.0 - press_adc as f32;
        calc = ((calc - (pv2 / 4096.0)) * 6250.0) / pv1;
        let p9t = (p9 * calc * calc) / 2147483648.0;
        let p8t = calc * (p8 / 32768.0);
        let c3 = calc / 256.0;
        let p10t = (c3 * c3 * c3) * (p10 / 131072.0);
        calc + (p9t + p8t + p10t + (p7 * 128.0)) / 16.0
    };

    // Report: temperature in centi-degrees, pressure in hPa, and the variant.
    let t_centi = (temp_c * 100.0) as i32;
    let p_hpa = (press_pa / 100.0) as i32;
    let mut buf = [0u8; 64];
    let mut pos = push_bytes(&mut buf, 0, b"id=61 v=");
    pos = push_hex8(&mut buf, pos, variant as u32);
    pos = push_bytes(&mut buf, pos, b" T=");
    pos = push_dec(
        &mut buf,
        pos,
        if t_centi < 0 { 0 } else { t_centi as u32 / 100 },
    );
    pos = push_bytes(&mut buf, pos, b".");
    let frac = (t_centi % 100).unsigned_abs();
    if frac < 10 {
        pos = push_bytes(&mut buf, pos, b"0");
    }
    pos = push_dec(&mut buf, pos, frac);
    pos = push_bytes(&mut buf, pos, if t_centi < 0 { b"C-" } else { b"C" });
    pos = push_bytes(&mut buf, pos, b" P=");
    pos = push_dec(&mut buf, pos, p_hpa.unsigned_abs());
    pos = push_bytes(&mut buf, pos, b"hPa t1=");
    pos = push_dec(&mut buf, pos, c.par_t1 as u32);

    // Passing needs: a fresh conversion, a room-plausible temperature
    // (-20..60 C) and a pressure a building can have (800..1100 hPa).
    let ok = read_ok && fresh && (-2000..=6000).contains(&t_centi) && (800..=1100).contains(&p_hpa);
    if ok {
        mailbox::pass(&buf[..pos]);
    } else {
        mailbox::fail(&buf[..pos]);
    }

    idle(&dp);
}

fn idle(dp: &pac::Peripherals) -> ! {
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
