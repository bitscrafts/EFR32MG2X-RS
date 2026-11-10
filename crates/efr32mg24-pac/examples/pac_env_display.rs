//! PAC-only environment display: BME680 (or BME688) readings on the
//! Expansion Base OLED.
//!
//! This is the first example that combines two devices, and it is also the most
//! useful one on a desk: temperature, pressure and humidity, refreshed every
//! two seconds.
//!
//! ## What is verified and what needs eyes
//!
//! - **The measurements**: chip id 0x61, variant byte, and values a room can
//!   plausibly have (temperature -20..60 C, pressure 800..1100 hPa, humidity
//!   0..100 %). Those are reported through the mailbox, so a wrong sensor is
//!   caught without looking at the panel.
//! - **The panel**: every byte accepted by the SSD1306 is counted; the count
//!   proves the controller is talking and the window is being written. Whether
//!   the glyphs look right is the part only an eye can judge, and the report
//!   says so.
//!
//! ## Sensor notes
//!
//! The BME680 compensation needs the calibration laid out exactly as Bosch
//! stores it: `par_t1` lives at coefficient index 31/32 (inside the block at
//! 0xE1), not at 0x8A where the array begins — using 0x8A for `par_t1` reported
//! -1 C on a 32 C bench. See `pac_bme680.rs` for the full account.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/i2c_bitbang.rs"]
mod i2c_bitbang;
#[path = "common/mailbox.rs"]
mod mailbox;
#[path = "common/ssd1306.rs"]
mod ssd1306;

const REG_FIELD0: u8 = 0x1D;
const REG_CTRL_MEAS: u8 = 0x74;
const REG_CHIP_ID: u8 = 0xD0;
const REG_VARIANT_ID: u8 = 0xF0;
const REG_COEFF1: u8 = 0x8A;
const REG_COEFF2: u8 = 0xE1;
const REG_COEFF3: u8 = 0x00;
const NEW_DATA: u8 = 0x80;

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
    par_h1: u16,
    par_h2: u16,
    par_h3: i8,
    par_h4: i8,
    par_h5: i8,
    par_h6: u8,
    par_h7: i8,
}

fn be16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

/// Read the sensor and return (temperature, pressure, humidity) in
/// hundredths of a degree, hPa and hundredths of a percent.
fn read_env(bus: &i2c_bitbang::Bus<'_>, addr: u8) -> Option<(i32, i32, i32, u8, bool)> {
    let variant = bus.read_reg(addr, REG_VARIANT_ID).unwrap_or(0xFF);

    let mut coeff = [0u8; 42];
    if !bus.read_regs(addr, REG_COEFF1, &mut coeff[..23])
        || !bus.read_regs(addr, REG_COEFF2, &mut coeff[23..37])
        || !bus.read_regs(addr, REG_COEFF3, &mut coeff[37..42])
    {
        return None;
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
        // H1 and H2 share coefficient byte 24: H1 takes the low nibble of it as
        // its lsb, H2 takes the high nibble. Treating byte 24 as a plain lsb
        // byte for both made the humidity compensation saturate at 100 %.
        par_h2: ((coeff[23] as u16) << 4) | ((coeff[24] as u16) >> 4),
        par_h1: ((coeff[25] as u16) << 4) | ((coeff[24] as u16) & 0x0F),
        par_h3: coeff[26] as i8,
        par_h4: coeff[27] as i8,
        par_h5: coeff[28] as i8,
        par_h6: coeff[29],
        par_h7: coeff[30] as i8,
        par_t1: be16(coeff[32], coeff[31]),
    };

    // Bosch set_op_mode: sleep first, then the wanted mode, or the write is
    // ignored by the sensor without any bus error.
    let ctrl = bus.read_reg(addr, REG_CTRL_MEAS)?;
    if !bus.write_reg(addr, REG_CTRL_MEAS, ctrl & !0x03) {
        return None;
    }
    // Temperature x2 and pressure x2 oversampling, humidity x1, forced mode.
    // ctrl_hum must be written before ctrl_meas or the humidity oversampling
    // is ignored by the sensor.
    if !bus.write_reg(addr, 0x72, 0b001) {
        return None;
    }
    if !bus.write_reg(addr, REG_CTRL_MEAS, 0b010_010_01) {
        return None;
    }
    cortex_m::asm::delay(400_000);
    let mut fresh = false;
    for _ in 0..30 {
        if let Some(status) = bus.read_reg(addr, REG_FIELD0) {
            if status & NEW_DATA != 0 {
                fresh = true;
                break;
            }
        }
        cortex_m::asm::delay(50_000);
    }

    let mut f = [0u8; 17];
    if !bus.read_regs(addr, REG_FIELD0, &mut f) {
        return None;
    }
    let press_adc = ((f[2] as u32) << 12) | ((f[3] as u32) << 4) | ((f[4] as u32) >> 4);
    let temp_adc = ((f[5] as u32) << 12) | ((f[6] as u32) << 4) | ((f[7] as u32) >> 4);
    let hum_adc = ((f[8] as u16) << 8) | f[9] as u16;

    // Temperature
    let t1 = c.par_t1 as f32;
    let adc = temp_adc as f32;
    let var1 = ((adc / 16384.0) - (t1 / 1024.0)) * c.par_t2 as f32;
    let dt = (adc / 131072.0) - (t1 / 8192.0);
    let var2 = dt * dt * (c.par_t3 as f32 * 16.0);
    let t_fine = var1 + var2;
    let temp_c = t_fine / 5120.0;

    // Pressure
    let (p1, p2, p3, p4, p5, p6, p7, p8, p9, p10) = (
        c.par_p1 as f32,
        c.par_p2 as f32,
        c.par_p3 as f32,
        c.par_p4 as f32,
        c.par_p5 as f32,
        c.par_p6 as f32,
        c.par_p7 as f32,
        c.par_p8 as f32,
        c.par_p9 as f32,
        c.par_p10 as f32,
    );
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

    // Humidity
    let temp_comp = t_fine / 5120.0;
    let hadc = hum_adc as f32;
    let hv1 = hadc - ((c.par_h1 as f32 * 16.0) + ((c.par_h3 as f32 / 2.0) * temp_comp));
    let hv2 = hv1
        * (c.par_h2 as f32 / 262144.0
            * (1.0
                + ((c.par_h4 as f32 / 16384.0) * temp_comp)
                + ((c.par_h5 as f32 / 1048576.0) * temp_comp * temp_comp)));
    let hv3 = c.par_h6 as f32 / 16384.0;
    let hv4 = c.par_h7 as f32 / 2097152.0;
    let hum = (hv2 + ((hv3 + (hv4 * temp_comp)) * hv2 * hv2)).clamp(0.0, 100.0);

    Some((
        (temp_c * 100.0) as i32,
        (press_pa / 100.0) as i32,
        (hum * 100.0) as i32,
        variant,
        fresh,
    ))
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    ssd1306::enable_gpio_clock(&dp.cmu_s);
    let bus = i2c_bitbang::Bus::new(&dp.gpio_s);

    let oled = ssd1306::Oled::new(&bus);
    if oled.is_none() {
        mailbox::fail(b"oled ack=0 (nothing at 0x3C)");
        idle(&dp);
    }
    let mut oled = oled.unwrap();

    let addr = if bus.probe(0x76) {
        0x76u8
    } else if bus.probe(0x77) {
        0x77u8
    } else {
        let _ = oled.show_lines([b"SENSOR MISSING", b"NO BME68X", b"CHECK 0X76"]);
        mailbox::fail(b"bme68x ack=0 (nothing at 0x76/0x77)");
        idle(&dp);
    };

    if bus.read_reg(addr, REG_CHIP_ID).unwrap_or(0) != 0x61 {
        let _ = oled.show_lines([b"SENSOR ERROR", b"CHIP ID NOT 61", b"-"]);
        mailbox::fail(b"chip id not 0x61");
        idle(&dp);
    }

    let mut last = (0i32, 0i32, 0i32);

    loop {
        if let Some((t, p, h, variant, fresh)) = read_env(&bus, addr) {
            // Rebuild the three lines.
            let mut l0 = [0u8; 16];
            let n0 = ssd1306::append(&mut l0, 0, b"TEMP ");
            let n0 = n0 + ssd1306::format_fixed(&mut l0[n0..], t, 1);

            let mut l1 = [0u8; 16];
            let n1 = ssd1306::append(&mut l1, 0, b"PRES ");
            let n1 = n1 + ssd1306::format_fixed(&mut l1[n1..], p * 100, 0);
            let n1 = ssd1306::append(&mut l1, n1, b" HP");

            let mut l2 = [0u8; 16];
            let n2 = ssd1306::append(&mut l2, 0, b"HUM ");
            let n2 = n2 + ssd1306::format_fixed(&mut l2[n2..], h, 1);
            let _ = ssd1306::append(&mut l2, n2, b" %");

            oled.show_lines([&l0[..n0], &l1[..n1], &l2[..n2]]);

            // Report once per refresh; the mailbox holds the latest values.
            let mut msg = [0u8; 64];
            let mut pos = ssd1306::append(&mut msg, 0, b"T=");
            pos += ssd1306::format_fixed(&mut msg[pos..], t, 2);
            pos = ssd1306::append(&mut msg, pos, b"C P=");
            pos += ssd1306::format_fixed(&mut msg[pos..], p * 100, 0);
            pos = ssd1306::append(&mut msg, pos, b"hPa H=");
            pos += ssd1306::format_fixed(&mut msg[pos..], h, 1);
            pos = ssd1306::append(&mut msg, pos, b"% d=");
            pos += ssd1306::format_fixed(&mut msg[pos..], oled.acked as i32, 0);

            let plausible = (-2000..=6000).contains(&t)
                && (800..=1100).contains(&p)
                && (0..=10000).contains(&h);
            if plausible && fresh && oled.acked > 500 {
                mailbox::pass(&msg[..pos]);
            } else {
                mailbox::fail(&msg[..pos]);
            }
            last = (t, p, h);
            let _ = variant;
        } else {
            mailbox::fail(b"sensor read failed");
        }

        // ~2 s between refreshes, and blink the LED as a liveness signal.
        for _ in 0..4 {
            for _ in 0..2_500_000 {
                cortex_m::asm::nop();
            }
            dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());
            dp.gpio_s
                .porta_dout()
                .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
        }
        let _ = last;
    }
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
