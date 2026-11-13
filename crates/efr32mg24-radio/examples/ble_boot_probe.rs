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

use cortex_m_rt::{entry, exception};
use efr32mg24_bluetooth_sys::sleeptimer;
use efr32mg24_hal::board::Board;
use efr32mg24_hal::pin::Port;
use efr32mg24_hal::prelude::*;
use panic_halt as _;

/// What the entropy test found, published for a debugger that can read a symbol address:
/// `[status, flags, requests]`.
///
/// `#[no_mangle]` and volatile stores, because this was optimised away on the first attempt: a private static that
/// the program never *reads* has no observable effect, so LLVM removed both the variable and the stores, and the
/// symbol was simply absent from the ELF.
#[no_mangle]
pub static mut ENTROPY_REPORT: [u32; 3] = [0; 3];

/// Counts how many times the timer callback ran. Written from an interrupt, read from the main loop.
static mut TICKS: u32 = 0;

extern "C" fn on_tick(_data: *mut c_void) {
    unsafe {
        let p = core::ptr::addr_of_mut!(TICKS);
        core::ptr::write_volatile(p, core::ptr::read_volatile(p).wrapping_add(1));
    }
}

// The recording `HardFault` handler lives in the library (`efr32mg24_radio::fault`), because every example needs it
// and an example that forgets it fails silently. Its record is the `FAULT_RECORD` static.

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

/// **The handler the vendor's bare-metal design requires.**
///
/// Without it the host's first `PENDSVSET` lands in `DefaultHandler_` and the firmware stops there with `ICSR`
/// reporting vector 14 — no fault, no status, indistinguishable from a deadlock. Measured on this board before this
/// handler existed.
#[exception]
fn PendSV() {
    efr32mg24_bluetooth_sys::stack_init::service_link_layer();
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

    // **The entropy test that cannot be faked.** Two calls on buffers pre-filled with a sentinel, before the stack
    // exists. Three separate things have to be true: both calls report success, the sentinel is gone (the SE really
    // wrote to memory -- a stub that returned 0 without writing would leave 0xA5 there), and the two outputs differ
    // (a counter that merely incremented would produce equal buffers only by coincidence, but a fixed value would
    // not).
    let mut e1 = [0xA5u8; 32];
    let mut e2 = [0xA5u8; 32];
    let s1 = unsafe { efr32mg24_bluetooth_sys::psa::psa_generate_random(e1.as_mut_ptr(), 32) };
    let s2 = unsafe { efr32mg24_bluetooth_sys::psa::psa_generate_random(e2.as_mut_ptr(), 32) };
    let sentinel_gone = !e1.contains(&0xA5) && !e2.contains(&0xA5);
    let differ = e1 != e2;
    let (req, retries, fails) = efr32mg24_bluetooth_sys::se_trng::stats();
    stage(
        b"entropy",
        s1 as u32,
        (s2 as u32 & 0xFF)
            | ((sentinel_gone as u32) << 8)
            | ((differ as u32) << 9)
            | (retries << 12)
            | (fails << 16),
    );
    stage(b"entropyreq", req, 0);
    // Published through a real static, not a fixed address: `0x2003_F6xx` sits inside the mailbox's own page, and the
    // record write overwrote these words (measured: they read back as 0xFFFFFFFF, a value no code here can produce).
    unsafe {
        let r = core::ptr::addr_of_mut!(ENTROPY_REPORT);
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*r)[0]), s1 as u32);
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*r)[1]),
            (sentinel_gone as u32)
                + ((differ as u32) << 1)
                + ((retries % 16) << 8)
                + ((fails % 16) << 12),
        );
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*r)[2]), req);
    }

    stage(b"boot", 0, 0);

    // **The SYSRTC's clock, before the radio is started.** Measured: with the gates off, RAIL's first write to
    // `SYSRTC->CMD` during its bring-up faults (`bfar=0x500a_8014`); with the module *running*, its `CFG` write
    // faults instead (`bfar=0x500a_800c`). Gated and disabled is the state that satisfies both, and the HAL now
    // exposes exactly that.
    {
        board.prepare_sysrtc_clocks();
    }

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
    let mut _first_header = 0u32;
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
            // **The id is `header & 0xffff00f8`, not a byte of it.** `SL_BGAPI_MSG_ID` in `sl_bgapi.h` masks with
            // `0xffff00f8`, and `sl_bt_evt_system_boot_id` is `0x000100a0`. Reading only the low byte (as this did
            // first) cannot tell one event from another, and it reported "no boot event" while an event was in fact
            // sitting in the record.
            let evt = unsafe { evt.assume_init() };
            let header = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(evt.header)) };
            if header & 0xffff_00f8 == 0x0001_00a0 {
                boot_event += 1;
            }
            if events == 1 {
                _first_header = header;
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
