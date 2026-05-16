//! Absolute bare minimum blink - NO clock init, just raw GPIO toggle
//! Test if ANY Rust code executes

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as _;
use panic_halt as _;
use cortex_m as _;

// Gecko Bootloader Application Properties
// CRITICAL: Must match Arduino format AND official Silicon Labs struct EXACTLY!
// Reference: https://docs.silabs.com/mcu-bootloader/latest/gecko-bootloader-api/application-properties-t
// Arduino has 8-byte header BEFORE magic, Word 13 points to magic (not header)
#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],                // 8-byte header (02 01 02 02 02 03 02 04 from Arduino)
    magic: [u8; 16],                // Magic signature (Word 13 points HERE, not to header)
    struct_version: u32,             // Version of this struct
    signature_type: u32,             // Type of signature (0 = none)
    signature_location: u32,         // Pointer to signature (0xFFFFFFFF = none)
    app_info: ApplicationData,       // Application metadata
    cert: u32,                       // Pointer to certificate (NULL = 0)
    long_token_section_address: u32, // Pointer to long token section (NULL = 0)
    decrypt_key: [u8; 16],          // Decryption key (zeros if not encrypted)
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32,           // Arduino uses 0x22 (34)
    version: u32,
    capabilities: u32,
    product_id: [u8; 16],
}

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7,
    0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    header: [0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04],  // Match Arduino exactly
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0201,        // Version 2.1
    signature_type: 0,              // No signature
    signature_location: 0xFFFFFFFF, // No signature
    app_info: ApplicationData {
        app_type: 0x22,             // Match Arduino (34 decimal) - CRITICAL!
        version: 0x00000001,        // Match Arduino byte order (v0.0.1)
        capabilities: 0,
        product_id: [0u8; 16],
    },
    cert: 0,                        // NULL pointer - no certificate
    long_token_section_address: 0,  // NULL pointer - no long token section
    decrypt_key: [0u8; 16],         // All zeros - no encryption
};

#[entry]
fn main() -> ! {
    unsafe {
        // Use CMU_S (secure peripheral) and enable multiple clocks
        const CMU_S_CLKEN0: *mut u32 = 0x50008308 as *mut u32;
        const CMU_S_CLKEN1: *mut u32 = 0x5000830C as *mut u32;

        // Enable HFXO, GPIO, and other necessary clocks
        core::ptr::write_volatile(CMU_S_CLKEN0, 0xFFFFFFFF);  // Enable all CLKEN0 clocks
        core::ptr::write_volatile(CMU_S_CLKEN1, 0xFFFFFFFF);  // Enable all CLKEN1 clocks

        const GPIO_PORTA_MODEL: *mut u32 = 0x50000004 as *mut u32;
        const GPIO_PORTA_DOUT: *mut u32 = 0x5000000C as *mut u32;
        const GPIO_PORTA_DOUTTGL: *mut u32 = 0x50000010 as *mut u32;

        // Configure PA7 as push-pull output (mode 4)
        let model = core::ptr::read_volatile(GPIO_PORTA_MODEL);
        core::ptr::write_volatile(GPIO_PORTA_MODEL, (model & !0xF0000000) | 0x40000000);

        // Blink LED
        loop {
            // Toggle LED (bit 7)
            core::ptr::write_volatile(GPIO_PORTA_DOUTTGL, 1 << 7);

            // Delay
            for _ in 0..1_000_000 {
                cortex_m::asm::nop();
            }
        }
    }
}
