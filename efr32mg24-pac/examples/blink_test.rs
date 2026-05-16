//! Blink test - LED active-low
//! Toggles PA7 LED with correct polarity

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as _;
use panic_halt as _;
use cortex_m as _;

// Gecko Bootloader Application Properties
#[repr(C)]
pub struct ApplicationProperties {
    magic: [u8; 16],
    struct_version: u32,
    signature_type: u32,
    signature_location: u32,
    app_info: ApplicationData,
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32,
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
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0100,
    signature_type: 0,
    signature_location: 0,
    app_info: ApplicationData {
        app_type: 0,
        version: 0x00010000,
        capabilities: 0,
        product_id: [0u8; 16],
    },
};

#[entry]
fn main() -> ! {
    unsafe {
        // CMU CLKEN0 register - enable GPIO clock
        const CMU_CLKEN0: *mut u32 = 0x40008308 as *mut u32;
        core::ptr::write_volatile(CMU_CLKEN0, 1 << 2);  // Enable GPIO clock (bit 2)

        // GPIO Port A registers
        const GPIO_PORTA_MODEL: *mut u32 = 0x50000004 as *mut u32;
        const GPIO_PORTA_DOUT: *mut u32 = 0x5000000C as *mut u32;

        // Configure PA7 as push-pull output (mode 4)
        // MODEL register: bits [31:28] = mode for pin 7
        core::ptr::write_volatile(GPIO_PORTA_MODEL, 0x40000000);

        // Blink loop - use read-modify-write to preserve other pins
        loop {
            // LED ON (active-low: clear bit 7)
            let val = core::ptr::read_volatile(GPIO_PORTA_DOUT);
            core::ptr::write_volatile(GPIO_PORTA_DOUT, val & !(1 << 7));

            // Delay
            for _ in 0..1_000_000 {
                cortex_m::asm::nop();
            }

            // LED OFF (active-low: set bit 7)
            let val = core::ptr::read_volatile(GPIO_PORTA_DOUT);
            core::ptr::write_volatile(GPIO_PORTA_DOUT, val | (1 << 7));

            // Delay
            for _ in 0..1_000_000 {
                cortex_m::asm::nop();
            }
        }
    }
}
