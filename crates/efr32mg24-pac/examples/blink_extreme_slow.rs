#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as _;
use panic_halt as _;

// ApplicationProperties (same as before)
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

#[entry]
fn main() -> ! {
    unsafe {
        // Unlock CMU
        const CMU_LOCK: *mut u32 = 0x40008010 as *mut u32;
        core::ptr::write_volatile(CMU_LOCK, 0x93F7);

        // Enable GPIO clock (bit 26)
        const CMU_CLKEN0: *mut u32 = 0x40008064 as *mut u32;
        core::ptr::write_volatile(CMU_CLKEN0, 1 << 26);

        // Configure PA7 as push-pull output
        const GPIO_PORTA_MODEL: *mut u32 = 0x5003C034 as *mut u32;
        let model = core::ptr::read_volatile(GPIO_PORTA_MODEL);
        core::ptr::write_volatile(GPIO_PORTA_MODEL, (model & !0xF0000000) | 0x40000000);

        // EXTREME SLOW BLINK - 10 second delays
        // If 39 MHz: 400M NOPs = ~10 seconds
        // If 19 MHz: 400M NOPs = ~20 seconds
        // Either way, should be VERY visible
        const GPIO_PORTA_DOUT: *mut u32 = 0x5003C03C as *mut u32;

        loop {
            // LED ON (active-low: write 0)
            core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);

            // VERY LONG DELAY (~10 seconds if 39 MHz)
            for _ in 0..400_000_000 {
                cortex_m::asm::nop();
            }

            // LED OFF (active-low: write 1 to bit 7)
            core::ptr::write_volatile(GPIO_PORTA_DOUT, 1 << 7);

            // VERY LONG DELAY (~10 seconds if 39 MHz)
            for _ in 0..400_000_000 {
                cortex_m::asm::nop();
            }
        }
    }
}
