//! Bare-minimum LED blink using PAC typed register accessors.
//!
//! Toggles PA7 (the XIAO MG24 onboard LED, active low) via read-modify-write
//! on `PORTA_DOUT`. Uses SVD-correct addresses through the PAC's typed
//! accessors (no raw pointers).
//!
//! # Hardware
//!
//! - XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
//! - LED on PA7 (Arduino pin 20, `LED_BUILTIN`, active low)
//!
//! # Register addresses (SVD-verified)
//!
//! - `CmuS` (secure CMU): `0x4000_8000`, `CLKEN0` at offset `0x064`, `gpio` = bit 26
//! - `GpioS` (secure GPIO): `0x4003_C000`, port registers are packed (stride ~0x30)
//! - `PORTA_MODEL` at block offset `0x034`, `PORTA_DOUT` at `0x040`
//! - EFR32MG24 has NO `DOUTTGL` register (toggle via read-modify-write on `DOUT`)
//!
//! # Flashing
//!
//! ```bash
//! cd efr32mg24-pac
//! cargo build --release --example blink_bare --features rt
//! probe-rs run --chip EFR32MG24B220F1536IM48 \
//!   target/thumbv8m.main-none-eabihf/release/examples/blink_bare
//! ```
//!
//! See `blink_xiao.rs` for the full verified blink example with detailed docs.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

// Gecko Bootloader Application Properties (required for bootloader boot).
// See blink_xiao.rs for full documentation.
#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],
    magic: [u8; 16],
    struct_version: u32,
    signature_type: u32,
    signature_location: u32,
    app_info: ApplicationData,
    cert: u32,
    long_token_section_address: u32,
    decrypt_key: [u8; 16],
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32,
    version: u32,
    capabilities: u32,
    product_id: [u8; 16],
}

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7, 0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    header: [0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04],
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0201,
    signature_type: 0,
    signature_location: 0xFFFFFFFF,
    app_info: ApplicationData {
        app_type: 0x22,
        version: 0x00000001,
        capabilities: 0,
        product_id: [0u8; 16],
    },
    cert: 0,
    long_token_section_address: 0,
    decrypt_key: [0u8; 16],
};

/// Delay loop: roughly ~1 Hz toggle at the default FSRCO (~19 MHz) or the
/// bootloader-configured SYSCLK.
#[inline(never)]
fn delay() {
    for _ in 0..1_000_000 {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // 1. Enable the GPIO clock in the secure CMU (CmuS at 0x4000_8000).
    //    CLKEN0 is at offset 0x064; the `gpio` field is bit 26.
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // 2. Configure PA7 as push-pull output (LED pin, active low).
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    // 3. Blink: toggle PA7 by read-modify-write on PORTA_DOUT.
    //    EFR32MG24 has no DOUTTGL, so XOR the bit in software.
    loop {
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });

        delay();
    }
}
