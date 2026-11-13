//! Where does the Bluetooth bring-up stop? One run, one flow, one answer.
//!
//! This probe exists because the questions kept changing while the instrument kept accumulating patches: at one
//! point it called `stack_init::start` **twice**, and the second call's trace overwrote the first call's, so a
//! stage that had failed read as a stage that had passed. It was rewritten from scratch with a single flow, and
//! every diagnostic slot is cleared before anything runs.
//!
//! The sequence, and what each step proves:
//!
//! 1. **HFXO, RAIL, channel table, host init, host start** — with a trace word per step, so a stall names itself.
//! 2. **The timer, after the stack** — RAIL configures the SYSRTC during bring-up and `CFG` cannot be written
//!    while the counter runs, so an application that starts it first makes the stack fault.
//! 3. **A BGAPI command** — a running host answers one. This is the difference between "wait longer for
//!    `system_boot`" and "the host is not running".
//! 4. **The event queue** — `system_boot` should be the first event.

#![no_std]
#![no_main]

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use core::ffi::c_void;

use cortex_m_rt::entry;
use efr32mg24_bluetooth_sys::sleeptimer;
use efr32mg24_hal::board::Board;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use panic_halt as _;

/// Counts how many times the timer callback ran. Written from an interrupt, read from the main loop.
static mut TICKS: u32 = 0;

extern "C" fn on_tick(_data: *mut c_void) {
    unsafe {
        let p = core::ptr::addr_of_mut!(TICKS);
        core::ptr::write_volatile(p, core::ptr::read_volatile(p).wrapping_add(1));
    }
}

/// A HardFault handler that reports instead of hanging. `cortex-m-rt`'s default is an empty loop, which is
/// indistinguishable from a slow init; this one records the exception frame's program counter plus `CFSR` and
/// `BFAR`, so the fault names the instruction that caused it.
///
/// # Safety
///
/// Reads the exception stack frame, which the architecture guarantees at this point, and writes a fixed RAM slot
/// that nothing else in this example uses.
#[no_mangle]
pub unsafe extern "C" fn HardFault() -> ! {
    // The handler has its own prologue, so the exception frame is not exactly at `msp`. Dump sixteen words and let
    // the frame be recognised by its own shape (`xPSR` has bit 0 set, `pc` sits two words above it).
    let sp = cortex_m::register::msp::read() as *const u32;
    let slot = 0x2003_F200 as *mut u32;
    slot.write_volatile(0xFA17_0000);
    slot.add(1).write_volatile(sp as u32);
    slot.add(2)
        .write_volatile(core::ptr::read_volatile(0xE000_ED28 as *const u32)); // CFSR
    slot.add(3)
        .write_volatile(core::ptr::read_volatile(0xE000_ED38 as *const u32)); // BFAR
    for i in 0..16 {
        slot.add(4 + i).write_volatile(sp.add(i).read_volatile());
    }
    loop {
        core::hint::spin_loop();
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
/// names the stage it stopped in.
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
    let board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();

    // Every diagnostic record survives a reset, and a stale one reads exactly like a fresh one. Clear all three
    // before anything runs: the RAIL assertion record, the fault record, and the bring-up trace.
    unsafe {
        let assert_slot = 0x2003_F500 as *mut u32;
        let fault_slot = 0x2003_F200 as *mut u32;
        let trace_slot = 0x2003_F600 as *mut u32;
        for i in 0..8 {
            trace_slot.add(i).write_volatile(0xAAAA_AAAA);
        }
        for i in 0..20 {
            fault_slot.add(i).write_volatile(0);
        }
        for i in 0..4 {
            assert_slot.add(i).write_volatile(0);
        }
    }

    // **NVM3 on its own, before the stack.** `sli_bt_start_stack` treats a failed `nvm3_open` as fatal, so knowing
    // whether NVM3 can open at all separates "the storage region or the flash driver is wrong" from "the stack
    // rejected something else". `nvm3_initDefault` is the vendor's own entry point and is compiled here.
    extern "C" {
        fn nvm3_initDefault() -> u32;
        fn nvm3_open(handle: *mut core::ffi::c_void, init: *const core::ffi::c_void) -> u32;
        static nvm3_defaultHandle: *mut core::ffi::c_void;
        static nvm3_defaultInit: *const core::ffi::c_void;
    }
    // The two globals the stack itself loads before calling `nvm3_open`, published so a null one is visible rather
    // than inferred: the crash this replaced was a read at `0x138`, which is a null handle plus a field offset.
    let (h, i) = unsafe { (nvm3_defaultHandle, nvm3_defaultInit) };
    stage(b"nvmptrs", h as usize as u32, i as usize as u32);
    let rc = unsafe { nvm3_open(h, i) };
    stage(b"nvmopen", rc, 0);
    let nvm = unsafe { nvm3_initDefault() };
    stage(b"nvm3", nvm, 0);

    stage(b"boot", 0, 0);

    // 1. The whole bring-up, traced.
    let boot = unsafe {
        efr32mg24_bluetooth_sys::stack_init::start(
            efr32mg24_bluetooth_sys::stack_init::default_configuration(),
        )
    };
    stage(b"initdone", boot, sleeptimer::irq_stats());

    // **The documented way to start the host, tried before anything else.** Its absence is why the stack is meant to
    // start automatically; if it answers here, it answers the question of who is supposed to run the stack.
    let start_cmd = unsafe { efr32mg24_bluetooth_sys::ffi::sl_bt_system_start_bluetooth() };
    stage(b"startcmd", start_cmd as u32, 0);

    // 2. The timer, now that the radio owns the counter. `board.sysrtc()` returns immediately on a running module.
    let rtc = board.sysrtc();
    let mut handle = [0u8; 32];
    let period_ticks = sleeptimer::sl_sleeptimer_ms32_to_tick(50);
    let sched = unsafe {
        sleeptimer::sl_sleeptimer_restart_timer(
            handle.as_mut_ptr(),
            period_ticks,
            Some(on_tick),
            core::ptr::null_mut(),
            0,
            0,
        )
    };
    stage(b"sched", sched, period_ticks);

    let iser2 = unsafe { core::ptr::read_volatile(0xE000_E108 as *const u32) };
    let ien = unsafe { core::ptr::read_volatile(0x500A_8064 as *const u32) };
    stage(b"nvic", iser2, ien);

    let mut waited = 0u32;
    while waited < 300 {
        delay.delay_ms(1);
        waited += 1;
        if unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TICKS)) } >= 3 {
            break;
        }
    }
    let ticks = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TICKS)) };
    let irqs = sleeptimer::irq_stats();
    stage(b"timer", ticks, irqs);
    let _ = rtc;

    // 3. A BGAPI command: if the host runs, this answers.
    let mut major: u16 = 0;
    let mut minor: u16 = 0;
    let mut patch: u16 = 0;
    let mut build: u16 = 0;
    let mut bootloader: u32 = 0;
    let mut hash: u32 = 0;
    let cmd = unsafe {
        efr32mg24_bluetooth_sys::ffi::sl_bt_system_get_version(
            &mut major,
            &mut minor,
            &mut patch,
            &mut build,
            &mut bootloader,
            &mut hash,
        )
    };
    stage(b"bgapi", cmd as u32, 0);

    // 4. The event queue.
    let mut events = 0u32;
    let mut last_pop = 0u32;
    let mut boot_event = 0u32;
    for _ in 0..300 {
        // **The missing pump.** The host asks for link-layer work through a callback; nothing happens in this build
        // unless the application answers it, and an unanswered host queues no events at all.
        efr32mg24_bluetooth_sys::stack_init::pump();
        sleeptimer::poll();
        let mut evt = core::mem::MaybeUninit::<efr32mg24_bluetooth_sys::ffi::sl_bt_msg_t>::uninit();
        let popped = unsafe { efr32mg24_bluetooth_sys::ffi::sl_bt_pop_event(evt.as_mut_ptr()) };
        last_pop = popped as u32;
        if popped == 0 {
            events += 1;
            // `sl_bt_evt_system_boot_id` is the event that proves the host finished coming up.
            let evt = unsafe { evt.assume_init() };
            let id =
                unsafe { core::ptr::read_volatile(core::ptr::addr_of!(evt.header) as *const u8) };
            if id == 0x00 {
                boot_event += 1;
            }
        }
        delay.delay_ms(1);
    }

    let (heap_in_use, heap_failures, heap_violations) = efr32mg24_bluetooth_sys::heap::stats();
    let assert = efr32mg24_bluetooth_sys::rail_assert::last_assert();

    let mut m = [0u8; 200];
    let mut n = 0;
    n = label(&mut m, n, b"boot=");
    n = push_hex(&mut m, n, boot);
    let (ll_cb, st_cb, _) = efr32mg24_bluetooth_sys::stack_init::adaptation_stats();
    n = label(&mut m, n, b" cb=");
    n = push_dec(&mut m, n, ll_cb as u32);
    n = label(&mut m, n, b"/");
    n = push_dec(&mut m, n, st_cb as u32);
    n = label(&mut m, n, b" stcmd=");
    n = push_hex(&mut m, n, start_cmd as u32);
    n = label(&mut m, n, b" cmd=");
    n = push_dec(&mut m, n, cmd as u32);
    n = label(&mut m, n, b" ver=");
    n = push_dec(&mut m, n, major as u32);
    n = label(&mut m, n, b".");
    n = push_dec(&mut m, n, minor as u32);
    n = label(&mut m, n, b" evt=");
    n = push_dec(&mut m, n, events);
    n = label(&mut m, n, b"/");
    n = push_dec(&mut m, n, boot_event);
    n = label(&mut m, n, b" pop=");
    n = push_dec(&mut m, n, last_pop);
    n = label(&mut m, n, b" tick=");
    n = push_dec(&mut m, n, ticks);
    n = label(&mut m, n, b"/");
    n = push_dec(&mut m, n, irqs);
    n = label(&mut m, n, b" heap=");
    n = push_dec(&mut m, n, heap_in_use as u32);
    n = label(&mut m, n, b"/");
    n = push_dec(&mut m, n, heap_failures);
    n = label(&mut m, n, b"/");
    n = push_dec(&mut m, n, heap_violations);
    n = label(&mut m, n, b" asrt=");
    let (assert_code, assert_handle, _) = assert.unwrap_or((0, 0, 0));
    n = push_dec(&mut m, n, assert_code);
    n = label(&mut m, n, b" h=");
    n = push_hex(&mut m, n, assert_handle);

    // PASS only when the host really came up: a boot event was seen and a command answered.
    let ok = boot == 0 && cmd == 0 && boot_event >= 1;
    mailbox::write(
        if ok {
            mailbox::Status::Pass
        } else {
            mailbox::Status::Fail
        },
        &m[..n],
    );

    let mut tick = false;
    loop {
        delay.delay_ms(250);
        tick = !tick;
        board.output_on(Port::A, 7, !tick);
        sleeptimer::poll();
        let mut evt = core::mem::MaybeUninit::<efr32mg24_bluetooth_sys::ffi::sl_bt_msg_t>::uninit();
        unsafe {
            efr32mg24_bluetooth_sys::ffi::sl_bt_pop_event(evt.as_mut_ptr());
        }
    }
}
