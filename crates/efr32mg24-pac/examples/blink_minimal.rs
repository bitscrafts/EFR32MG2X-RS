//! Minimal blink with proper clock initialization
//! Based on EFR32MG24 reference manual clock setup

#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use efr32mg24_pac as _;
use panic_halt as _;

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
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7, 0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
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
        // CMU base address: 0x40008000
        const CMU_BASE: u32 = 0x40008000;

        // CMU registers
        const CMU_CLKEN0: *mut u32 = (CMU_BASE + 0x308) as *mut u32; // Clock enable
        const CMU_SYSCLKCTRL: *mut u32 = (CMU_BASE + 0x100) as *mut u32; // System clock control

        // Step 1: Ensure HFRCO (High Frequency RC Oscillator) is enabled and ready
        // HFRCO is typically enabled by default after reset

        // Step 2: Select HFRCODPLL as SYSCLK source
        // SYSCLKCTRL: bits [1:0] = clock source
        //   0x1 = HFRCODPLL (default)
        //   0x2 = HFXO
        //   0x3 = CLKIN0
        core::ptr::write_volatile(CMU_SYSCLKCTRL, 0x1); // Use HFRCODPLL

        // Step 3: Enable GPIO clock
        core::ptr::write_volatile(CMU_CLKEN0, 1 << 2); // Bit 2 = GPIO

        // Small delay for clock to stabilize
        for _ in 0..1000 {
            cortex_m::asm::nop();
        }

        // GPIO base address: 0x50000000
        const GPIO_BASE: u32 = 0x50000000;

        // Port A registers
        const GPIO_PORTA_MODEL: *mut u32 = (GPIO_BASE + 0x004) as *mut u32; // Pin 0-7 mode
        const GPIO_PORTA_DOUT: *mut u32 = (GPIO_BASE + 0x00C) as *mut u32; // Data out

        // Configure PA7 as push-pull output (mode 4)
        // MODEL register: bits [31:28] = mode for pin 7
        core::ptr::write_volatile(GPIO_PORTA_MODEL, 0x40000000);

        // Blink loop
        loop {
            // LED ON (active-low: clear bit 7)
            let val = core::ptr::read_volatile(GPIO_PORTA_DOUT);
            core::ptr::write_volatile(GPIO_PORTA_DOUT, val & !(1 << 7));

            // Delay (~0.5s at 78 MHz)
            for _ in 0..5_000_000 {
                cortex_m::asm::nop();
            }

            // LED OFF (active-low: set bit 7)
            let val = core::ptr::read_volatile(GPIO_PORTA_DOUT);
            core::ptr::write_volatile(GPIO_PORTA_DOUT, val | (1 << 7));

            // Delay (~0.5s at 78 MHz)
            for _ in 0..5_000_000 {
                cortex_m::asm::nop();
            }
        }
    }
}
