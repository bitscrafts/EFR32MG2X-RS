#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;
use core::arch::asm;

// Bootloader configuration
const APP_START_ADDR: u32 = 0x08004000;  // Application starts after 16 KB bootloader
const BOOTLOADER_MAGIC: u32 = 0xB00710AD;  // Magic value to enter bootloader mode

// Memory addresses from linker script
extern "C" {
    static mut _sbss: u32;
    static mut _ebss: u32;
    static mut _sdata: u32;
    static mut _edata: u32;
    static _stack_start: u32;
}

/// Entry point - called by Reset handler
#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    // Initialize RAM
    init_ram();

    // Check if we should enter bootloader mode
    if should_enter_bootloader() {
        bootloader_mode();
    }

    // Otherwise, boot the application
    boot_application(APP_START_ADDR);
}

/// Initialize RAM (.data and .bss sections)
unsafe fn init_ram() {
    // Zero-initialize .bss section
    let mut bss = &raw mut _sbss as *mut u32;
    let bss_end = &raw mut _ebss as *mut u32;
    while bss < bss_end {
        ptr::write_volatile(bss, 0);
        bss = bss.offset(1);
    }

    // Note: .data initialization would go here if we had initialized data
    // For minimal bootloader, we avoid initialized data to save space
}

/// Check if bootloader mode should be entered
fn should_enter_bootloader() -> bool {
    // For minimal bootloader, check a magic value in RAM
    // This can be set by application before reset to trigger bootloader

    // Check last word of RAM for magic value
    const BOOTLOADER_FLAG_ADDR: *mut u32 = 0x2003FFFC as *mut u32;

    unsafe {
        let flag = ptr::read_volatile(BOOTLOADER_FLAG_ADDR);
        if flag == BOOTLOADER_MAGIC {
            // Clear flag
            ptr::write_volatile(BOOTLOADER_FLAG_ADDR, 0);
            return true;
        }
    }

    // For now, always boot application directly
    // Future: Add button check, UART detection, etc.
    false
}

/// Bootloader mode - placeholder for future firmware update
fn bootloader_mode() -> ! {
    // TODO: Implement UART firmware update
    // For now, just blink LED slowly to indicate bootloader mode

    unsafe {
        // Initialize peripherals
        init_peripherals();

        // Slow blink to indicate bootloader mode
        loop {
            led_on();
            delay_ms(1000);
            led_off();
            delay_ms(1000);
        }
    }
}

/// Boot the application at given address
fn boot_application(app_addr: u32) -> ! {
    unsafe {
        // Read application's vector table
        let app_vector_table = app_addr as *const u32;
        let app_stack_ptr = ptr::read_volatile(app_vector_table.offset(0));
        let app_reset_vector = ptr::read_volatile(app_vector_table.offset(1));

        // Validate stack pointer (should be in RAM range)
        if app_stack_ptr < 0x20000000 || app_stack_ptr > 0x20040000 {
            // Invalid stack pointer, stay in bootloader
            bootloader_mode();
        }

        // Validate reset vector (should be in application flash range, Thumb mode bit set)
        if (app_reset_vector & 0xFFFFFFF0) < app_addr || (app_reset_vector & 1) == 0 {
            // Invalid reset vector, stay in bootloader
            bootloader_mode();
        }

        // Disable all interrupts
        asm!("cpsid i", options(nomem, nostack));

        // Set Vector Table Offset Register (VTOR)
        const VTOR: *mut u32 = 0xE000ED08 as *mut u32;
        ptr::write_volatile(VTOR, app_addr);

        // Memory barrier to ensure VTOR write completes
        asm!("dsb", options(nostack));
        asm!("isb", options(nostack));

        // Set stack pointer and jump to application
        core::arch::asm!(
            "msr msp, {stack}",    // Set Main Stack Pointer
            "bx {reset}",          // Branch to application reset handler
            stack = in(reg) app_stack_ptr,
            reset = in(reg) app_reset_vector,
            options(noreturn)
        );
    }
}

/// Initialize peripherals needed for bootloader
unsafe fn init_peripherals() {
    // Unlock CMU
    const CMU_LOCK: *mut u32 = 0x40008010 as *mut u32;
    ptr::write_volatile(CMU_LOCK, 0x93F7);

    // Enable GPIO clock (bit 26)
    const CMU_CLKEN0: *mut u32 = 0x40008064 as *mut u32;
    ptr::write_volatile(CMU_CLKEN0, 1 << 26);

    // Configure PA7 as push-pull output (LED)
    const GPIO_PORTA_MODEL: *mut u32 = 0x5003C034 as *mut u32;
    let model = ptr::read_volatile(GPIO_PORTA_MODEL);
    ptr::write_volatile(GPIO_PORTA_MODEL, (model & !0xF0000000) | 0x40000000);
}

/// Turn LED ON (active-low)
unsafe fn led_on() {
    const GPIO_PORTA_DOUT: *mut u32 = 0x5003C03C as *mut u32;
    ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);
}

/// Turn LED OFF (active-low)
unsafe fn led_off() {
    const GPIO_PORTA_DOUT: *mut u32 = 0x5003C03C as *mut u32;
    ptr::write_volatile(GPIO_PORTA_DOUT, 0x80);
}

/// Simple delay (busy-wait, not accurate)
fn delay_ms(ms: u32) {
    // Rough estimate: ~39 MHz / 4 cycles per loop = ~10 NOPs per microsecond
    let cycles = ms * 10000;
    for _ in 0..cycles {
        unsafe { asm!("nop", options(nomem, nostack)); }
    }
}

/// Reset handler - first code to run after boot
#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    main()
}

/// Default exception handler
#[no_mangle]
pub unsafe extern "C" fn DefaultHandler() -> ! {
    loop {
        asm!("nop", options(nomem, nostack));
    }
}

/// HardFault handler - blink LED fast
#[no_mangle]
pub unsafe extern "C" fn HardFault() -> ! {
    // Try to initialize peripherals if not already done
    init_peripherals();

    // Fast blink to indicate fault
    loop {
        led_on();
        delay_ms(100);
        led_off();
        delay_ms(100);
    }
}

/// Panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        HardFault()
    }
}

// Build script will look for this file
#[used]
#[link_section = ".text"]
static KEEP_SYMBOLS: u32 = 0;
