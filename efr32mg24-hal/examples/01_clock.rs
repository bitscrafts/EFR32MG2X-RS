//! Clock Configuration Example
//!
//! This example demonstrates how to configure the EFR32MG24's clock system
//! using both external crystals and internal RC oscillators.
//!
//! Target: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # Hardware Setup
//!
//! The XIAO MG24 includes:
//! - 39 MHz HFXO crystal for high-frequency clock
//! - 32.768 kHz LFXO crystal for low-frequency clock
//!
//! No external connections required for this example.
//!
//! # Expected Behavior
//!
//! The example configures clocks and prints the configured frequencies.
//! In a real application, you would use these clocks to configure peripherals.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig, LfxoConfig},
    pac,
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Get peripheral access
    let dp = pac::Peripherals::take().unwrap();

    // Example 1: Using external crystals (XIAO MG24 default)
    // This provides the most accurate clock for radio and timing operations
    let _clocks_external = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)), // 39 MHz HFXO
            lfxo: Some(LfxoConfig::default()),         // 32.768 kHz LFXO
        }
    );

    // Access the frozen clock frequencies
    // In a real application, you would use these for peripheral configuration
    // let hfclk = clocks_external.hfclk().0;  // 39,000,000 Hz
    // let lfclk = clocks_external.lfclk().0;  // 32,768 Hz
    // let pclk = clocks_external.pclk().0;    // 39,000,000 Hz (same as HFCLK)
    // let sysclk = clocks_external.sysclk().0; // 39,000,000 Hz (same as HFCLK)

    // For subsequent examples, we need more CMU peripherals
    // In a real application, you would only configure clocks once
    // Here we demonstrate different configurations for educational purposes

    // Since CMU_S was consumed above, we'll just show the remaining examples in comments:

    // Example 2: Using internal RC oscillators
    // This is useful when external crystals are not available
    // Note: Lower accuracy affects radio protocols and precise timing
    // let clocks_internal = Clocks::new(dp.cmu_s, ClockConfig::default());
    // The default configuration uses:
    // - HFRCO at 19 MHz (internal RC oscillator)
    // - LFRCO at 32.768 kHz (internal RC oscillator)

    // Example 3: Mixed configuration
    // Use external HFXO for accuracy but internal LFRCO for simplicity
    // let clocks_mixed = Clocks::new(
    //     dp.cmu_s,
    //     ClockConfig {
    //         hfxo: Some(HfxoConfig::new(39_000_000)), // External 39 MHz
    //         lfxo: None,                               // Use internal LFRCO
    //     }
    // );

    // Example 4: Freezing clocks for use with peripherals
    // Once frozen, clocks cannot be reconfigured, ensuring peripheral stability

    // Get a new CMU peripheral for this example
    let dp2 = unsafe { pac::Peripherals::steal() };
    let frozen_clocks = Clocks::new(
        dp2.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(LfxoConfig::default()),
        }
    ).freeze();

    // Now these frozen clocks can be passed to peripheral drivers
    // let delay = Delay::new(cp.SYST, &frozen_clocks);
    // let serial = Serial::new(dp.USART0, &frozen_clocks, 115200.bps());

    // Get individual clock frequencies from frozen clocks
    let _hfclk = frozen_clocks.hfclk();
    let _lfclk = frozen_clocks.lfclk();
    let _pclk = frozen_clocks.pclk();
    let _sysclk = frozen_clocks.sysclk();

    // In a real application with semihosting or RTT, you could print:
    // rprintln!("HFCLK:  {} Hz", hfclk.0);
    // rprintln!("LFCLK:  {} Hz", lfclk.0);
    // rprintln!("PCLK:   {} Hz", pclk.0);
    // rprintln!("SYSCLK: {} Hz", sysclk.0);

    // Clock configuration complete
    loop {
        // In a real application, your main loop would go here
        cortex_m::asm::nop();
    }
}

// Notes on Clock Selection:
//
// 1. For BLE/Thread/Zigbee/Matter radio operations:
//    - MUST use HFXO (external crystal) for required accuracy
//    - HFRCO is not accurate enough for radio protocols
//
// 2. For accurate timekeeping (RTC):
//    - Use LFXO (32.768 kHz crystal) for best accuracy
//    - LFRCO has ±4% tolerance, which adds up over time
//
// 3. For quick prototyping without external components:
//    - Use ClockConfig::default() for HFRCO + LFRCO
//    - Suitable for GPIO, timers, basic peripherals
//    - Not suitable for radio or precision timing
//
// 4. Power optimization:
//    - LFXO uses less power than LFRCO in deep sleep modes
//    - HFXO uses less power than HFRCO at equivalent frequencies
//    - Consider clock gating for unused peripherals
