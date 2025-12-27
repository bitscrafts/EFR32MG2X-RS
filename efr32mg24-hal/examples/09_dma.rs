//! DMA (LDMA) Memory-to-Memory Transfer Example
//!
//! This example demonstrates using the LDMA controller for efficient
//! memory-to-memory data transfers without CPU intervention.
//!
//! # Hardware Setup
//!
//! No external hardware required - this example performs internal memory transfers.
//!
//! # Expected Behavior
//!
//! 1. Initializes the system clock with HFXO (39 MHz crystal)
//! 2. Creates source and destination buffers in memory
//! 3. Performs DMA transfer from source to destination
//! 4. Verifies data integrity after transfer
//! 5. Demonstrates different transfer sizes (byte, halfword, word)
//! 6. Uses delay between tests for observation
//!
//! # Test Cases
//!
//! - **Test 1**: Transfer u8 array (byte transfers)
//! - **Test 2**: Transfer u16 array (halfword transfers)
//! - **Test 3**: Transfer u32 array (word transfers)
//! - **Test 4**: Large buffer transfer (1024 bytes)
//!
//! # Success Criteria
//!
//! All destination buffers match their source buffers after DMA transfer.
//!
//! # Build and Flash
//!
//! ```bash
//! cargo build --example 09_dma --features rt --release
//! cargo run --example 09_dma --features rt --release
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    clock::{ClockConfig, Clocks, HfxoConfig},
    delay::Delay,
    dma::Dma,
    pac,
    prelude::*,
};
use panic_halt as _;

// Test data patterns
const PATTERN_U8: [u8; 16] = [
    0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10,
];

const PATTERN_U16: [u16; 8] = [
    0x0123, 0x4567, 0x89AB, 0xCDEF, 0xFEDC, 0xBA98, 0x7654, 0x3210,
];

const PATTERN_U32: [u32; 4] = [0x01234567, 0x89ABCDEF, 0xFEDCBA98, 0x76543210];

#[entry]
fn main() -> ! {
    // Get peripheral instances
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // CRITICAL SAFETY: Preserve debug access in debug builds
    // This prevents bricking the board when debugging with probe-rs/J-Link
    #[cfg(debug_assertions)]
    unsafe {
        // Keep HFXO enabled for SWD clock
        dp.cmu_s.clken0().modify(|_, w| w.hfxo0().set_bit());

        // Keep SWD enabled - PREVENTS BRICKING
        dp.cmu_s.clken1().modify(|_, w| w.swd().set_bit());
    }

    // Configure system clock with external 39 MHz crystal (XIAO MG24 Sense)
    let (clocks, cmu) = Clocks::new(
        dp.cmu_s,
        ClockConfig {
            hfxo: Some(HfxoConfig::new(39_000_000)),
            lfxo: Some(Default::default()),
        },
    )
    .expect("Clock configuration failed");

    let frozen_clocks = clocks.freeze(cmu);

    // Create delay provider
    let mut delay = Delay::new(cp.SYST, &frozen_clocks);

    // Initialize DMA controller
    let mut dma = Dma::new(dp.ldma_s);
    let mut ch0 = dma.channel0();

    // ========================================================================
    // Test 1: Byte (u8) Transfer
    // ========================================================================

    let mut dst_u8 = [0u8; 16];

    // Perform DMA transfer
    ch0.transfer(&PATTERN_U8, &mut dst_u8)
        .expect("Byte transfer failed");

    // Verify transfer
    assert_eq!(dst_u8, PATTERN_U8, "Byte transfer verification failed");

    delay.delay_ms(100);

    // ========================================================================
    // Test 2: Halfword (u16) Transfer
    // ========================================================================

    let mut dst_u16 = [0u16; 8];

    ch0.transfer(&PATTERN_U16, &mut dst_u16)
        .expect("Halfword transfer failed");

    assert_eq!(
        dst_u16, PATTERN_U16,
        "Halfword transfer verification failed"
    );

    delay.delay_ms(100);

    // ========================================================================
    // Test 3: Word (u32) Transfer
    // ========================================================================

    let mut dst_u32 = [0u32; 4];

    ch0.transfer(&PATTERN_U32, &mut dst_u32)
        .expect("Word transfer failed");

    assert_eq!(dst_u32, PATTERN_U32, "Word transfer verification failed");

    delay.delay_ms(100);

    // ========================================================================
    // Test 4: Large Buffer Transfer (1024 bytes)
    // ========================================================================

    let mut src_large = [0u32; 256]; // 256 * 4 = 1024 bytes
    let mut dst_large = [0u32; 256];

    // Fill source with sequential pattern
    for (i, val) in src_large.iter_mut().enumerate() {
        *val = i as u32;
    }

    ch0.transfer(&src_large, &mut dst_large)
        .expect("Large buffer transfer failed");

    assert_eq!(
        src_large, dst_large,
        "Large buffer transfer verification failed"
    );

    delay.delay_ms(100);

    // ========================================================================
    // Test 5: Edge Case - Single Element Transfer
    // ========================================================================

    let src_single = [0xDEADBEEFu32];
    let mut dst_single = [0u32];

    ch0.transfer(&src_single, &mut dst_single)
        .expect("Single element transfer failed");

    assert_eq!(
        dst_single[0], 0xDEADBEEF,
        "Single element transfer verification failed"
    );

    // ========================================================================
    // All Tests Passed - Enter Success Loop
    // ========================================================================

    loop {
        // All DMA tests passed successfully
        // In a real application, you would continue with your logic here
        delay.delay_ms(1000);
    }
}
