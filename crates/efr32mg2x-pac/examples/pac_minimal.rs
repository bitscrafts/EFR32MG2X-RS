//! The smallest program that runs on the board, from the PAC alone.
//!
//! Everything a first program needs is here and nothing else: the bootloader's `ApplicationProperties` block (the Gecko
//! bootloader checks it before jumping — without it the image is flashed and never started), the GPIO clock gate in the
//! secure CMU, PA7 as a push-pull output, and a toggle by read-modify-write on `DOUT`.
//!
//! This is the same program shape as `pac_led_blink`, with the helper module inlined so that a reader can paste it as
//! one file. Whether it was watched on the board is recorded in `docs/examples.md`, generated from the files themselves.
//!
//! Two things this program shows about the silicon, both learned by breaking them:
//!
//! * there is no `DOUTTGL` on this part, so a toggle is a read-modify-write of `DOUT`;
//! * the GPIO clock lives in the **secure** CMU (`0x4000_8000`), which is why the field is `cmu_s` and not `cmu_ns`.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg2x_pac as pac;
use panic_halt as _;

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7, 0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

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

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    header: [0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04],
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0201,
    signature_type: 0,
    signature_location: 0xFFFF_FFFF,
    app_info: ApplicationData {
        app_type: 0x22,
        version: 0x0000_0001,
        capabilities: 0,
        product_id: [0u8; 16],
    },
    cert: 0,
    long_token_section_address: 0,
    decrypt_key: [0u8; 16],
};

#[entry]
fn main() -> ! {
    // One line, no module, no dependency: the diagnostic slot belongs to the previous binary until something says
    // otherwise, and a reader would take its record as this program's own. Measured on 2026-11-15: without this,
    // `pac_minimal` was reported as "mic rst=ok rms=504" -- the microphone example that had run before it.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let dp = pac::Peripherals::take().unwrap();

    // 1. The GPIO clock, gated by bit 26 of CLKEN0 in the secure CMU.
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // 2. PA7 is the LED (active low). MODEL holds the pins 0-7 mode fields; mode7() is this one.
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    // 3. Toggle it. No DOUTTGL on this part: XOR the bit in software.
    loop {
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });

        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}
