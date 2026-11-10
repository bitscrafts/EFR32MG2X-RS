//! Microphone Audio Streaming Example
//!
//! Demonstrates continuous audio capture from MSM381ACT001 analog microphone
//! using IADC + DMA circular buffer.
//!
//! Hardware: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//!
//! # What This Example Shows
//!
//! - **Analog microphone**: MSM381ACT001 driver
//! - **Continuous capture**: DMA circular buffer (512 samples)
//! - **16 kHz sampling**: CD-quality audio for voice
//! - **Zero-copy**: Direct IADC → DMA → Buffer
//! - **Power management**: GPIO control of microphone power
//!
//! # Hardware
//!
//! - MSM381ACT001 analog MEMS microphone (onboard)
//! - PC8: Microphone power control
//! - PC9: Analog audio input
//!
//! # Operation
//!
//! ```text
//! MSM381ACT001 → PC9 → IADC0 → DMA → [512 samples]
//!      ↑                                    ↓
//!     PC8 (power)                    Read continuously
//! ```
//!
//! # Expected Output
//!
//! ```text
//! Microphone Streaming at 16 kHz
//! Buffer: 512 samples (32ms latency)
//! Capturing...
//! [Audio samples would be processed here]
//! ```
//!
//! # Build
//!
//! ```bash
//! cargo run --example 06_microphone_streaming --target thumbv8m.main-none-eabihf --release
//! ```
//!
//! # Real-World Applications
//!
//! - Voice recognition
//! - Sound level monitoring
//! - Audio recording
//! - Voice commands
//! - Acoustic event detection

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use xiao_mg24_bsp::{sensors::Microphone, Board};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

#[entry]
fn main() -> ! {
    // ========================================================================
    // Step 1: Initialize Board
    // ========================================================================

    let mut board = Board::take().unwrap();
    let mut delay = board.into_delay();

    // ========================================================================
    // Step 2: Initialize Microphone
    // ========================================================================

    // Convert pins to correct modes
    let power_pin = board.pins.pc8.into_push_pull_output();
    let analog_pin = board.pins.pc9.into_analog();

    // Create microphone driver
    // - 512-sample buffer (32ms at 16 kHz)
    // - Uses IADC0 + DMA CH0
    let mut mic = Microphone::new(
        power_pin,
        analog_pin,
        &board.clocks,
        board.peripherals.ldma_s,
        512, // Buffer size
    )
    .expect("Failed to create microphone");

    // ========================================================================
    // Step 3: Start Audio Capture
    // ========================================================================

    // Start continuous capture at 16 kHz
    mic.start(16_000).expect("Failed to start microphone");

    // Small delay to allow first samples to accumulate
    delay.delay_ms(50);

    // ========================================================================
    // Step 4: Main Loop - Process Audio
    // ========================================================================

    loop {
        // Read available audio samples
        if let Some(samples) = mic.read() {
            if samples.len() > 0 {
                // In a real application, you would:
                // 1. Calculate RMS level for volume detection
                // 2. Run FFT for frequency analysis
                // 3. Feed to voice recognition engine
                // 4. Stream over BLE to smartphone
                // 5. Detect keywords/wake words
                //
                // Example: Simple volume detection
                // let rms = calculate_rms(samples);
                // if rms > THRESHOLD {
                //     trigger_action();
                // }
            }
        }

        // Process at ~100 Hz (10ms intervals)
        // This gives time for new samples to accumulate
        delay.delay_ms(10);
    }
}

/// Calculate RMS (Root Mean Square) audio level
///
/// Returns audio level in range 0-4095 (12-bit ADC)
#[allow(dead_code)]
fn calculate_rms(samples: &[u16]) -> u32 {
    if samples.is_empty() {
        return 0;
    }

    // Calculate mean
    let mean: u32 = samples.iter().map(|&s| s as u32).sum::<u32>() / samples.len() as u32;

    // Calculate sum of squared differences
    let sum_sq: u32 = samples
        .iter()
        .map(|&s| {
            let diff = (s as i32) - (mean as i32);
            (diff * diff) as u32
        })
        .sum();

    // Return RMS (integer square root)
    integer_sqrt(sum_sq / samples.len() as u32)
}

/// Integer square root (simplified)
#[allow(dead_code)]
fn integer_sqrt(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }

    let mut x = n;
    let mut y = (x + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }

    x
}
