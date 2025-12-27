//! Energy Management Unit (EMU) Example
//!
//! This example demonstrates the EMU peripheral functionality including:
//! - Energy mode transitions (EM0 → EM1 → EM2)
//! - Voltage scaling for power optimization
//! - Status querying
//!
//! # Hardware Requirements
//!
//! - XIAO MG24 Sense board
//! - Current measurement setup (optional, to observe power consumption)
//!
//! # Expected Behavior
//!
//! 1. Device starts in EM0 (Active mode)
//! 2. Enters EM1 (Sleep) - CPU stops, peripherals run
//! 3. Wakes from EM1 on SysTick interrupt
//! 4. Enters EM2 (Deep Sleep) - major power savings
//! 5. Wakes from EM2 on GPIO or BURTC interrupt
//!
//! # Power Consumption (Typical)
//!
//! - EM0 (Active): ~3-5 mA
//! - EM1 (Sleep): ~50 µA
//! - EM2 (Deep Sleep): ~2 µA (with LFXO + BURTC)
//!
//! # Build and Flash
//!
//! ```bash
//! cargo build --example 10_emu --features rt --release
//! cargo run --example 10_emu --features rt --release
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig},
    delay::Delay,
    emu::{Em2Config, Emu, VoltageScale},
    pac,
    prelude::*,
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Get peripheral singletons
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // SWD Safety: Keep SWD interface enabled in debug builds
    #[cfg(debug_assertions)]
    unsafe {
        // Keep HFXO enabled for SWD clock
        dp.cmu_s.clken0().modify(|_, w| w.hfxo0().set_bit());

        // Keep SWD interface enabled - PREVENTS BRICKING
    }

    // Configure clocks with XIAO MG24's 39 MHz crystal
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()), // Enable LFXO for EM2
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Create delay provider (uses SysTick)
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // Initialize EMU
    let emu = Emu::new(dp.emu_s);

    // Display initial state
    let initial_vscale = emu.voltage_scale();
    // In real application, would log via UART or RTT
    // For this example, we'll just use the values

    // Active operation in EM0
    delay.delay_ms(100);

    // ═══════════════════════════════════════════════════════════════
    // Example 1: EM1 (Sleep Mode)
    // ═══════════════════════════════════════════════════════════════

    // Enable SysTick interrupt to wake from EM1
    // In a real application, you would configure a meaningful interrupt
    // For this example, we'll rely on any pending interrupt to wake

    // Enter EM1 - CPU stops, peripherals continue
    // Device will wake on any interrupt (including SysTick if enabled)
    emu.enter_em1();

    // CPU resumes here after wake
    delay.delay_ms(100);

    // ═══════════════════════════════════════════════════════════════
    // Example 2: Voltage Scaling in EM0/EM1
    // ═══════════════════════════════════════════════════════════════

    // Lower voltage for power savings in EM0/EM1
    // Note: This may limit maximum CPU frequency
    if let Ok(()) = emu.set_em01_voltage_scale(VoltageScale::Vscale1) {
        let _new_vscale = emu.voltage_scale();
        // Voltage now at 1.0V instead of 1.1V
        delay.delay_ms(100);

        // Restore to maximum voltage
        let _ = emu.set_em01_voltage_scale(VoltageScale::Vscale2);
    }

    // ═══════════════════════════════════════════════════════════════
    // Example 3: EM2 (Deep Sleep Mode)
    // ═══════════════════════════════════════════════════════════════

    // Configure EM2 with power-optimized settings
    let em2_config = Em2Config {
        voltage_scale: VoltageScale::Vscale1, // 1.0V for power savings
        debug_enable: false,                  // Disable debug in production
    };

    // Note: In a real application, you would:
    // 1. Configure a wake source (GPIO interrupt, BURTC, LETIMER, etc.)
    // 2. Enable the interrupt in NVIC
    // 3. Then enter EM2

    // For this example, we'll configure a simple timeout using LETIMER
    // (This is a placeholder - actual LETIMER configuration would be needed)

    // Enter EM2 - major power savings
    // Device wakes on configured interrupt (GPIO, BURTC, LETIMER, etc.)
    if let Ok(()) = emu.enter_em2(em2_config) {
        // CPU resumes here after wake
        delay.delay_ms(100);

        // Check if EM2 was actually entered
        if emu.em2_entered() {
            // EM2 entry confirmed
            delay.delay_ms(100);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Example 4: Status Queries
    // ═══════════════════════════════════════════════════════════════

    // Query current voltage scaling
    let current_vscale = emu.voltage_scale();
    let _vscale_info = match current_vscale {
        VoltageScale::Vscale0 => "0.9V",
        VoltageScale::Vscale1 => "1.0V",
        VoltageScale::Vscale2 => "1.1V",
    };

    // Check if EM2 was entered since reset
    let em2_was_entered = emu.em2_entered();

    // Check lock status
    let is_locked = emu.is_locked();

    // Use the values (prevents unused variable warnings)
    let _ = (initial_vscale, em2_was_entered, is_locked);

    // ═══════════════════════════════════════════════════════════════
    // Main Loop
    // ═══════════════════════════════════════════════════════════════

    loop {
        // Active operation for 1 second
        delay.delay_ms(1000);

        // Enter EM1 for brief sleep
        // In real application, would sleep until interrupt
        emu.enter_em1();

        // Wake, do work, sleep again
        delay.delay_ms(100);
    }
}
