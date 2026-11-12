//! PAC-level LED blink for the XIAO MG24 Sense (EFR32MG24B220F1536IM48-B).
//!
//! **VERIFIED ON HARDWARE (2026-06-29):** The onboard LED blinks when this
//! example is run via `probe-rs run`. This proves the full PAC infrastructure
//! works end-to-end: PAC typed register accessors -> CMU_S clock gate ->
//! GPIO_S port config -> DOUT read-modify-write toggle -> cortex-m-rt entry ->
//! linker memory map (0x08006000) -> probe-rs SWD flash. See "Flashing" below
//! for the exact confirmed-working command.
//!
//! Minimal "does anything we built actually run on silicon" test. Uses ONLY
//! the PAC (no HAL, no BSP): the PAC's typed register accessors, cortex-m-rt
//! entry, the Gecko bootloader `ApplicationProperties` struct, and the onboard
//! CMSIS-DAP via probe-rs/openocd. If the LED blinks, the entire PAC
//! infrastructure and the flashing recipe are proven end-to-end.
//!
//! # LED pin (confirmed)
//!
//! Drives **PA7** (Arduino pin 20, `LED_BUILTIN`, active low). This is
//! confirmed by TWO independent sources:
//! - The official Silicon Labs Arduino core variant
//!   `variants/xiao_mg24/arduino_variant.cpp:75`: `PA7, // LED - 20`
//! - The `pins_arduino.h` line 38: `#define LED_BUILTIN (20u)` with
//!   `LED_BUILTIN_ACTIVE LOW` (line 39).
//! - Hardware-verified: the `microphone_volume_with_bootloader.hex` example
//!   dims this same LED (via PWM on `LED_BUILTIN`) and was confirmed working.
//!
//! NOTE: the BSP's `xiao-mg24-bsp/src/pins.rs:70` claims PB2/D10 is the LED.
//! That is **wrong** - PB2 is Arduino D13 (SDA1) per `arduino_variant.cpp:68`.
//! The BSP `pins.rs` must be fixed (follow-on task).
//!
//! # Why PAC typed accessors, not raw pointers
//!
//! The older `blink.rs`/`blink_bare.rs` examples used hardcoded raw pointer
//! addresses that are WRONG for the EFR32MG24:
//!
//! - `CmuS` (secure CMU) is at `0x4000_8000` (PAC `lib.rs:346`); the old
//!   examples used `0x50008308` which is `CmuNs + 0x308` (wrong instance AND
//!   wrong offset - CLKEN0 is at `+0x064`, not `+0x308`).
//! - `GpioS` (secure GPIO) is at `0x4003_C000` (PAC `lib.rs:508`); the old
//!   examples used `0x5000_0000` which is `ScratchpadNs`, not GPIO.
//! - The GPIO port registers are packed (port stride ~0x30), NOT at a `0x100`
//!   per-port stride. `PORTB_MODEL` is at block offset `0x064`, `PORTA_MODEL`
//!   at `0x034`, `PORTA_DOUT` at `0x040` (verified against the SVD).
//! - EFR32MG24 has **no `DOUTTGL` register** (HAL `gpio/README.md`: "EFR32MG24
//!   does not have atomic DOUTSET/DOUTCLR/DOUTTGL registers"). The toggle is a
//!   read-modify-write on `DOUT`.
//!
//! Using the PAC's typed accessors (`cmu_s.clken0()`, `gpio_s.porta_model()`,
//! `gpio_s.porta_dout()`) uses the SVD-correct addresses and offsets, exactly
//! as the HAL does internally.
//!
//! # Clock
//!
//! No clock init in this example: the Gecko bootloader (restored from
//! `microphone_volume_with_bootloader.hex`) configures the clock tree
//! (HFXO/HFRCODPLL as SYSCLK) before jumping to the app, so HFCLK and the
//! GPIO bus clock are already running. This example only enables the GPIO
//! clock gate in `CMU_S.CLKEN0` (bit 26, the `gpio` field), matching the
//! HAL's `gpio/mod.rs:97`.
//!
//! IMPORTANT: booting via the Gecko bootloader (power-cycle) is REQUIRED for
//! this example to blink. If you flash with `probe-rs run` (which bypasses
//! the bootloader and runs the app directly from the reset vector), the
//! clock tree is in its default FSRCO state. GPIO should still work on
//! FSRCO, but the delay loop timing differs. For the canonical test, flash
//! to 0x08006000 and power-cycle so the bootloader runs first.
//!
//! # Flashing
//!
//! ```bash
//! cd efr32mg24-pac
//! cargo build --release --example blink_xiao --features rt
//!
//! # Flash the app at 0x08006000 (does NOT touch the bootloader at 0x08000000):
//! probe-rs download --chip EFR32MG24B220F1536IM48 \
//!   target/thumbv8m.main-none-eabihf/release/examples/blink_xiao
//!
//! # Then power-cycle the board (unplug/replug USB) so the Gecko bootloader
//! # runs, configures clocks, and jumps to the app. The LED blinks at ~1 Hz.
//! ```
//!
//! `probe-rs download` writes the ELF at its linked addresses (vector table
//! at `0x08006000`). It does not erase the bootloader region
//! (`0x08000000-0x08005FFF`) because the ELF's sections are linked above
//! `0x08006000`.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

// Gecko Bootloader Application Properties
// CRITICAL: Must match Arduino format AND official Silicon Labs struct EXACTLY!
// Reference: https://docs.silabs.com/mcu-bootloader/latest/gecko-bootloader-api/application-properties-t
// Arduino has 8-byte header BEFORE magic, Word 13 points to magic (not header)
#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],           // 8-byte header (02 01 02 02 02 03 02 04 from Arduino)
    magic: [u8; 16],           // Magic signature (Word 13 points HERE, not to header)
    struct_version: u32,       // Version of this struct
    signature_type: u32,       // Type of signature (0 = none)
    signature_location: u32,   // Pointer to signature (0xFFFFFFFF = none)
    app_info: ApplicationData, // Application metadata
    cert: u32,                 // Pointer to certificate (NULL = 0)
    long_token_section_address: u32, // Pointer to long token section (NULL = 0)
    decrypt_key: [u8; 16],     // Decryption key (zeros if not encrypted)
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32, // Arduino uses 0x22 (34)
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
    header: [0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04], // Match Arduino exactly
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0201,         // Version 2.1
    signature_type: 0,              // No signature
    signature_location: 0xFFFFFFFF, // No signature
    app_info: ApplicationData {
        app_type: 0x22,      // Match Arduino (34 decimal) - CRITICAL!
        version: 0x00000001, // Match Arduino byte order (v0.0.1)
        capabilities: 0,
        product_id: [0u8; 16],
    },
    cert: 0,                       // NULL pointer - no certificate
    long_token_section_address: 0, // NULL pointer - no long token section
    decrypt_key: [0u8; 16],        // All zeros - no encryption
};

/// Delay loop: roughly ~1 Hz toggle at the default FSRCO (~19 MHz) or the
/// bootloader-configured SYSCLK. Each iteration is several cycles (nop + loop
/// overhead), so 1_000_000 iterations is on the order of a few million cycles.
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
    //    CLKEN0 is at offset 0x064; the `gpio` field is bit 26 (verified in the
    //    SVD). This matches the HAL's gpio/mod.rs:97.
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // 2. Configure PA7 as push-pull output. PA7 is the XIAO MG24 user LED
    //    (Arduino pin 20, active low). MODEL holds pin mode fields for pins
    //    0-7; mode7() is the field for pin 7. pushpull() selects push-pull
    //    output drive (mode 4 in the EFR32 GPIO MODE enum).
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    // 3. Blink: toggle PA7 by read-modify-write on PORTA_DOUT.
    //    EFR32MG24 has no DOUTTGL, so XOR the bit in software. The LED is active
    //    low, so clearing bit 7 turns it ON and setting bit 7 turns it OFF; the
    //    XOR toggle alternates between the two regardless of polarity.
    loop {
        // Toggle PA7 (bit 7).
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });

        delay();
    }
}
