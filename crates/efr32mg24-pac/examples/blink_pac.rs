#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

// ApplicationProperties structure
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
    let dp = pac::Peripherals::take().unwrap();

    unsafe {
        // Enable all clocks to be safe
        dp.cmu_s.clken0().write(|w| w.bits(0xFFFFFFFF));
        dp.cmu_s.clken1().write(|w| w.bits(0xFFFFFFFF));

        // Configure PA7 as push-pull output (mode 4 = 0b0100)
        dp.gpio_s
            .porta_model()
            .modify(|_, w| w.bits((w.bits() & !0xF0000000) | 0x40000000));

        // Blink loop - toggle by reading and XORing
        loop {
            // Read current value
            let current = dp.gpio_s.porta_dout().read().bits();
            // Toggle bit 7
            dp.gpio_s.porta_dout().write(|w| w.bits(current ^ (1 << 7)));

            // Delay
            for _ in 0..1_000_000 {
                cortex_m::asm::nop();
            }
        }
    }
}
