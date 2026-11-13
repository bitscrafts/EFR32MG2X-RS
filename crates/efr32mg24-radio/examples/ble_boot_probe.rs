//! Does the timer work on its own, and where does the stack's init stop?
//!
//! Two questions that a single run can separate:
//!
//! 1. **Is our timer path real?** Schedule a timer for 100 ms with a Rust callback, spin for a while, and report
//!    whether the callback ran and whether the compare interrupt fired. If this works, the timer is not the
//!    problem, whatever happens next.
//! 2. **Where does `sli_bt_init_stack` stop?** Call it afterwards and report its status next to the interrupt
//!    counter and the heap state. Three outcomes, three different faults: it never returns (it waits for
//!    something), it returns a status (the stack is telling us what it wants), or it returns zero and no
//!    `system_boot` ever arrives (the host started but its event loop is not being serviced).
//!
//! The order matters. Arming first means the interrupt is already enabled and has already been taken at least
//! once before the stack is involved, so a zero interrupt count afterwards cannot be blamed on the arming code.

#![no_std]
#![no_main]

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use core::ffi::c_void;

use cortex_m_rt::entry;
use efr32mg24_bluetooth_sys::sleeptimer;
use efr32mg24_hal::board::Board;
use efr32mg24_hal::prelude::*;
use panic_halt as _;

/// Counts how many times the timer callback ran. `static mut` with volatile access, because it is written from an
/// interrupt handler and read from the main loop.
static mut TICKS: u32 = 0;

/// The callback the stack uses too: it must be `extern "C"` and take the timer's data pointer.
extern "C" fn on_tick(_data: *mut c_void) {
    unsafe {
        let p = core::ptr::addr_of_mut!(TICKS);
        core::ptr::write_volatile(p, core::ptr::read_volatile(p).wrapping_add(1));
    }
}

fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    loop {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
        if value == 0 || n == digits.len() {
            break;
        }
    }
    for i in (0..n).rev() {
        out[at] = digits[i];
        at += 1;
    }
    at
}

fn push_hex(out: &mut [u8], mut at: usize, value: u32) -> usize {
    let hex = b"0123456789ABCDEF";
    for shift in [12u32, 8, 4, 0] {
        out[at] = hex[((value >> shift) & 0xF) as usize];
        at += 1;
    }
    at
}

fn label(out: &mut [u8], mut at: usize, s: &[u8]) -> usize {
    for b in s {
        out[at] = *b;
        at += 1;
    }
    at
}

/// Publish a stage marker. The mailbox keeps the last record, so if the program stops, the record that is there
/// names the stage it stopped in -- which is the whole point of publishing as we go instead of at the end.
fn stage(tag: &[u8], a: u32, b: u32) {
    let mut m = [0u8; 64];
    let mut n = 0;
    n = label(&mut m, n, tag);
    n = label(&mut m, n, b" a=");
    n = push_dec(&mut m, n, a);
    n = label(&mut m, n, b" b=");
    n = push_dec(&mut m, n, b);
    mailbox::write(mailbox::Status::Pass, &m[..n]);
}

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();

    stage(b"boot", 0, 0);

    // 1. The timer on its own, before the stack exists.
    let rtc = board.sysrtc();
    stage(b"sysrtc", 1, 0);
    let mut handle = [0u8; 32];
    let period_ticks = sleeptimer::sl_sleeptimer_ms32_to_tick(50);
    let status = unsafe {
        sleeptimer::sl_sleeptimer_restart_timer(
            handle.as_mut_ptr(),
            period_ticks,
            Some(on_tick),
            core::ptr::null_mut(),
            0,
            0,
        )
    };

    stage(b"sched", status, period_ticks);

    // Read back the three things an interrupt needs: the peripheral's enable bit, the NVIC's enable bit, and the
    // core's global mask. Without these three numbers, "no interrupt" is four different faults wearing one name.
    let iser2 = unsafe { core::ptr::read_volatile(0xE000_E108 as *const u32) };
    let ien = unsafe { core::ptr::read_volatile(0x500A_8064 as *const u32) };
    let cmp0 = unsafe { core::ptr::read_volatile(0x500A_806C as *const u32) };
    let cnt = unsafe { core::ptr::read_volatile(0x500A_8018 as *const u32) };
    let primask = cortex_m::register::primask::read();
    stage(b"iser2", iser2, ien);
    stage(b"cmpcnt", cmp0, cnt);
    stage(b"primask", primask as u32, 0);

    // Wait for at least three periods. `poll()` is deliberately *not* called here: the point is whether the
    // interrupt alone can run the callback.
    let mut waited = 0u32;
    while waited < 200 {
        delay.delay_ms(1);
        waited += 1;
        if unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TICKS)) } >= 3 {
            break;
        }
    }

    let tick_count = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TICKS)) };
    stage(b"waited", tick_count, sleeptimer::irq_stats());
    let irqs = sleeptimer::irq_stats();
    let _ = rtc;

    // 2. Now the stack, with the timer already proven.
    stage(b"preinit", tick_count, sleeptimer::irq_stats());
    let boot = unsafe {
        efr32mg24_bluetooth_sys::stack_init::start(
            efr32mg24_bluetooth_sys::stack_init::default_configuration(),
        )
    };

    stage(b"initdone", boot, sleeptimer::irq_stats());

    // Give the host a moment to emit system_boot, then ask for events. This is where the polled path and the
    // interrupt path differ, so the loop calls both.
    delay.delay_ms(100);
    let mut events = 0u32;
    let mut last_pop = 0u32;
    for _ in 0..200 {
        sleeptimer::poll();
        let mut evt = core::mem::MaybeUninit::<efr32mg24_bluetooth_sys::ffi::sl_bt_msg_t>::uninit();
        let popped = unsafe { efr32mg24_bluetooth_sys::ffi::sl_bt_pop_event(evt.as_mut_ptr()) };
        last_pop = popped as u32;
        if popped == 0 {
            events += 1;
        }
        delay.delay_ms(1);
    }

    let mut m = [0u8; 180];
    let mut n = 0;
    n = label(&mut m, n, b"st=");
    n = push_hex(&mut m, n, status);
    n = label(&mut m, n, b" boot=");
    n = push_hex(&mut m, n, boot);
    n = label(&mut m, n, b" tick=");
    n = push_dec(&mut m, n, tick_count);
    n = label(&mut m, n, b" irq=");
    n = push_dec(&mut m, n, irqs);
    n = label(&mut m, n, b" tick2=");
    n = push_dec(&mut m, n, unsafe {
        core::ptr::read_volatile(core::ptr::addr_of!(TICKS))
    });
    n = label(&mut m, n, b" irq2=");
    n = push_dec(&mut m, n, sleeptimer::irq_stats());
    n = label(&mut m, n, b" evt=");
    n = push_dec(&mut m, n, events);
    n = label(&mut m, n, b" pop=");
    n = push_dec(&mut m, n, last_pop);
    let (used, fails, viol) = efr32mg24_bluetooth_sys::heap::stats();
    n = label(&mut m, n, b" heap=");
    n = push_dec(&mut m, n, used as u32);
    n = label(&mut m, n, b"/");
    n = push_dec(&mut m, n, fails);
    n = label(&mut m, n, b"/");
    n = push_dec(&mut m, n, viol);

    let ok = tick_count >= 1 && irqs >= 1;
    mailbox::write(
        if ok {
            mailbox::Status::Pass
        } else {
            mailbox::Status::Fail
        },
        &m[..n],
    );

    loop {
        cortex_m::asm::wfi();
    }
}
