//! ADC Single-Shot Conversion Example
//!
//! This example demonstrates analog-to-digital conversion using the IADC0 peripheral.
//! It reads voltage from multiple ADC channels and converts the values to millivolts.
//!
//! # Hardware Setup
//!
//! - Target: Seeed Studio XIAO MG24 Sense
//! - ADC: IADC0 peripheral (12-bit resolution)
//! - Reference: Internal 1.21V bandgap (VBGR)
//! - Channels: Ch0-Ch5 (Port A pins PA0-PA5)
//! - Crystal: 39 MHz HFXO
//!
//! # Pin Connections
//!
//! Connect analog voltage sources to:
//! - Ch0: PA0 (measure 0-1.21V range with VBGR reference)
//! - Ch1: PA1
//! - Ch2: PA2
//! - Ch3: PA3
//! - Ch4: PA4
//! - Ch5: PA5
//!
//! # Reference Voltage
//!
//! This example uses VBGR (1.21V internal bandgap) reference:
//! - Input range: 0 - 1.21V
//! - Resolution: 1.21V / 4095 ≈ 0.295 mV per LSB
//! - For higher voltages, use VDD reference (see alternative example)
//!
//! # Expected Behavior
//!
//! - Reads voltage from Ch0 every second
//! - Displays raw ADC value (0-4095) and calculated voltage in mV
//! - Ground reference (Ch::Gnd) should read close to 0 mV
//! - Open channel may read noise (0-50 mV typical)
//!
//! # Build & Run
//!
//! ```bash
//! cargo build --example 08_adc --features rt --release
//! # Flash to device with probe-rs
//! cargo run --example 08_adc --features rt --release
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    adc::{Adc, Channel, Config, Reference},
    clock::{ClockConfig, Clocks, HfxoConfig},
    delay::Delay,
    pac,
    prelude::*,
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Get peripheral singletons
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // CRITICAL SAFETY: Preserve debug access in debug builds
    #[cfg(debug_assertions)]
    unsafe {
        dp.CMU_S.clken0().modify(|_, w| w.hfxo0().set_bit());
        dp.CMU_S.clken1().modify(|_, w| w.swd().set_bit());
    }

    // Configure clocks with XIAO MG24's 39 MHz crystal
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Create delay provider for timing
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // Create ADC with VBGR reference (1.21V)
    // This provides a stable reference voltage independent of VDD
    let mut adc = Adc::new(dp.iadc0_s, Config::default(), &frozen_clocks);

    // Main loop: continuously read ADC channels
    loop {
        // Read from Channel 0 (PA0)
        match adc.read(Channel::Ch0) {
            Ok(value) => {
                // Convert 12-bit value to millivolts using VBGR reference (1.21V)
                // Formula: voltage_mv = (value * 1210) / 4095
                let voltage_mv = (value as u32 * 1210) / 4095;

                // In a real application, you would output this via USART or store it
                // For this example, values are available for debugger inspection
                let _ = (value, voltage_mv);
            }
            Err(_) => {
                // Conversion timeout - hardware issue
                // In production, you might retry or report an error
            }
        }

        // Read from ground reference for calibration check
        // Should read close to 0 mV (typically 0-5 mV due to noise)
        match adc.read(Channel::Gnd) {
            Ok(value) => {
                let voltage_mv = (value as u32 * 1210) / 4095;
                let _ = (value, voltage_mv);
            }
            Err(_) => {}
        }

        // Wait 1 second between readings
        delay.delay_ms(1000);
    }
}

// Example: Using VDD reference for full supply voltage range
#[allow(dead_code)]
fn demonstrate_vdd_reference() {
    let dp = pac::Peripherals::take().unwrap();
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Configure ADC with VDD reference (typically 3.3V)
    let config = Config::default().with_reference(Reference::Vdd);
    let mut adc = Adc::new(dp.iadc0_s, config, &frozen_clocks);

    // Read voltage (assuming VDD = 3.3V)
    if let Ok(value) = adc.read(Channel::Ch0) {
        // Convert to millivolts: voltage_mv = (value * 3300) / 4095
        let voltage_mv = (value as u32 * 3300) / 4095;
        let _ = voltage_mv;
    }
}

// Example: Reading multiple channels
#[allow(dead_code)]
fn demonstrate_multi_channel_reading(adc: &mut Adc) {
    // Array to store readings from all 6 channels
    let mut readings: [Option<u16>; 6] = [None; 6];

    // Read all channels
    for (i, channel) in [
        Channel::Ch0,
        Channel::Ch1,
        Channel::Ch2,
        Channel::Ch3,
        Channel::Ch4,
        Channel::Ch5,
    ]
    .iter()
    .enumerate()
    {
        readings[i] = adc.read(*channel).ok();
    }

    // Process readings
    for (i, reading) in readings.iter().enumerate() {
        if let Some(value) = reading {
            let voltage_mv = (*value as u32 * 1210) / 4095;
            let _ = (i, voltage_mv);
        }
    }
}

// Example: Averaging multiple samples for noise reduction
#[allow(dead_code)]
fn demonstrate_averaging(adc: &mut Adc, samples: u8) -> Option<u16> {
    let mut sum: u32 = 0;
    let mut count = 0u8;

    for _ in 0..samples {
        if let Ok(value) = adc.read(Channel::Ch0) {
            sum += value as u32;
            count += 1;
        }
    }

    if count > 0 {
        Some((sum / count as u32) as u16)
    } else {
        None
    }
}

// Example: Voltage divider calculation
// For measuring voltages higher than reference voltage
#[allow(dead_code)]
fn demonstrate_voltage_divider() {
    // Hardware setup: Input voltage divided by 2 with resistors
    // R1 = 10kΩ to input, R2 = 10kΩ to GND
    // ADC measures voltage at divider midpoint
    //
    // Example: Measure 2.42V input with VBGR (1.21V) reference
    // Divider output = 2.42V / 2 = 1.21V (at ADC range limit)
    //
    // ADC reading ≈ 4095 (full scale)
    // Calculated voltage at divider = 1.21V
    // Actual input voltage = 1.21V * 2 = 2.42V

    let dp = pac::Peripherals::take().unwrap();
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    let mut adc = Adc::new(dp.iadc0_s, Config::default(), &frozen_clocks);

    if let Ok(value) = adc.read(Channel::Ch0) {
        // Calculate voltage at divider output
        let divider_voltage_mv = (value as u32 * 1210) / 4095;

        // Calculate actual input voltage (divider ratio = 2)
        let input_voltage_mv = divider_voltage_mv * 2;

        let _ = input_voltage_mv;
    }
}
