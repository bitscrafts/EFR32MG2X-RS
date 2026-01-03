//! DMA Continuous ADC Sampling Example
//!
//! Demonstrates LDMA peripheral-to-memory transfers with circular buffer mode.
//! Continuously samples analog input using IADC + DMA without CPU intervention.
//!
//! Hardware: Any EFR32MG24 board
//!
//! # What This Example Shows
//!
//! - **C2-DMA.2**: Peripheral-to-memory DMA transfers
//! - **Circular buffer**: Automatic wraparound at buffer end
//! - **LDMAXBAR**: Peripheral request routing to DMA channels
//! - **Zero-copy**: Direct peripheral-to-memory without CPU
//! - **Continuous operation**: No interrupts, no CPU wake-up
//!
//! # Hardware Setup
//!
//! Connect analog signal to:
//! - **Port C, Pin 4 (PC4)** - IADC input channel 0
//! - Voltage range: 0-1.21V (internal reference)
//!
//! # Operation
//!
//! ```text
//! IADC Scan → FIFO → DMA → Circular Buffer
//!                      ↓
//!              [0][1][2]...[255]
//!               ↑________________↓
//!                    LINK
//! ```
//!
//! 1. IADC configured for continuous scan mode
//! 2. Each conversion triggers DMA request
//! 3. DMA transfers FIFO data to buffer
//! 4. Buffer wraps automatically (circular mode)
//! 5. CPU reads buffer without stopping DMA
//!
//! # Expected Output
//!
//! ```text
//! DMA Continuous ADC Sampling
//! Sampling at 1 kHz, 256-sample circular buffer
//! Buffer filled continuously...
//! [Samples would be printed if UART available]
//! ```
//!
//! # Build
//!
//! ```bash
//! cargo build --example 09_dma_adc_continuous --target thumbv8m.main-none-eabihf --release
//! ```

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use efr32mg24_hal as hal;
use hal::dma::{Dma, PeripheralRequest};
use hal::pac;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

// IADC register addresses (from EFR32MG24 reference manual)
const IADC0_BASE: u32 = 0x4808_4000;
const IADC0_SCANFIFODATA: u32 = IADC0_BASE + 0x28; // Scan FIFO data register

// Circular buffer for continuous ADC samples
static mut ADC_BUFFER: [u32; 256] = [0; 256];

#[entry]
fn main() -> ! {
    // ========================================================================
    // Step 1: Initialize Peripherals
    // ========================================================================

    let dp = pac::Peripherals::take().unwrap();

    // Initialize DMA controller
    let mut dma = Dma::new(dp.ldma_s);
    let mut ch0 = dma.channel0();

    // ========================================================================
    // Step 2: Configure IADC for Continuous Scanning
    // ========================================================================

    configure_iadc(&dp);

    // ========================================================================
    // Step 3: Start DMA Circular Buffer Transfer
    // ========================================================================

    // Get mutable reference to circular buffer
    let adc_buffer = unsafe { &mut ADC_BUFFER };

    // Start continuous peripheral-to-memory DMA
    // This will:
    // 1. Route IADC scan complete signal to DMA CH0 (via LDMAXBAR)
    // 2. Configure CH0 for peripheral-to-memory transfer
    // 3. Enable circular buffer mode (LINK descriptor)
    // 4. Wait for IADC scan requests (no CPU intervention)
    ch0.start_peripheral_to_memory(
        IADC0_SCANFIFODATA,
        adc_buffer,
        PeripheralRequest::IadcScan,
    )
    .expect("Failed to start DMA");

    // ========================================================================
    // Step 4: Main Loop - DMA Runs Autonomously
    // ========================================================================

    // DMA now runs continuously:
    // - IADC performs conversions at 1 kHz
    // - Each conversion triggers DMA request
    // - DMA transfers result to buffer
    // - When buffer fills, DMA wraps to beginning (circular)
    // - CPU can read buffer at any time without stopping DMA

    loop {
        // In a real application, you would:
        // 1. Read available samples from buffer
        // 2. Process data (FFT, filtering, etc.)
        // 3. Transmit via UART, BLE, etc.

        // For this example, just sleep
        cortex_m::asm::wfi();

        // Example of reading buffer (commented out - needs position tracking)
        // if let Some(write_pos) = ch0.get_position() {
        //     // Read samples from last_read to write_pos
        //     // Process data...
        // }
    }
}

/// Configure IADC for continuous scanning
///
/// Setup:
/// - Input: PC4 (single-ended)
/// - Reference: Internal 1.21V
/// - Sample rate: ~1 kHz
/// - Mode: Continuous scan
fn configure_iadc(dp: &pac::Peripherals) {
    // SAFETY: We have exclusive access to CMU and IADC peripherals
    unsafe {
        let cmu = &(*pac::CmuS::ptr());
        let iadc = &(*pac::Iadc0S::ptr());

        // Enable IADC clock
        cmu.clken0().modify(|_, w| w.iadc0().set_bit());

        // Enable IADC peripheral
        iadc.en().write(|w| w.en().set_bit());

        // Wait for IADC to be ready
        while !iadc.status().read().adcwarm().bit_is_set() {}

        // Configure IADC for scan mode
        // - 12-bit resolution
        // - Internal 1.21V reference
        // - Continuous conversion
        iadc.cfg0().write(|w| {
            w.adcmode()
                .normal() // Normal mode (not high-speed)
                .osrhs()
                .hispd2() // Oversample ratio for high-speed: 2x
                .analoggain()
                .anagain1() // Analog gain: 1x
                .refsel()
                .vbgr() // Reference: Internal 1.21V bandgap
                .digavg()
                .avg1() // Digital averaging: 1 sample
        });

        // Configure scan table entry 0: PC4 (single-ended)
        iadc.scantable().write(|w| w.bits(0x04)); // PC4 = port C, pin 4

        // Configure scan mode
        iadc.scanfifocfg().write(|w| {
            w.alignment()
                .right12() // Right-align 12-bit data
                .dmawufifosingle()
                .set_bit() // DMA wake-up on single entry
        });

        // Start continuous scanning
        iadc.cmd().write(|w| w.scanstart().set_bit());
    }
}
