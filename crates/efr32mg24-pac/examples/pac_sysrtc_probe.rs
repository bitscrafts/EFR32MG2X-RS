//! Which SYSRTC write faults, in the vendor's own order.
//!
//! The HAL's `Sysrtc::take` faults and reports nothing; this probe publishes a marker after every write so
//! the last marker names the step that died. Bit positions and order are the vendor's
//! (`sl_hal_sysrtc_init` + `sleeptimer_hal_init_timer`, Gecko SDK v4.4.5), not inferred.

#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

#[entry]
fn main() -> ! {
    let dp = match pac::Peripherals::take() {
        Some(d) => d,
        None => loop {
            core::hint::spin_loop();
        },
    };
    mailbox::info(b"s0: pac taken");

    // 1. Bus clock, then counter clock.
    dp.cmu_s.clken0().modify(|_, w| w.sysrtc0().set_bit());
    mailbox::info(b"s1: clken0 set");

    dp.cmu_s
        .sysrtc0clkctrl()
        .modify(|r, w| unsafe { w.bits((r.bits() & !0x3) | 1) });
    mailbox::info(b"s2: clksel=LFRCO");

    let rtc = &dp.sysrtc0_s;

    // 2. Is the module enabled? A read of EN also proves the bus clock took effect: an unclocked
    //    peripheral faults on read, it does not return zero.
    let en = rtc.en().read().bits() & 1;
    if en != 0 {
        mailbox::info(b"s3: en=1");
    } else {
        mailbox::info(b"s3: en=0");
    }

    // 3. Wait for synchronisation, then configure.
    let mut spins = 0u32;
    while rtc.syncbusy().read().bits() != 0 && spins < 100_000 {
        spins += 1;
    }
    mailbox::info(b"s4: syncbusy clear");

    rtc.cfg().write(|w| unsafe { w.bits(0) });
    mailbox::info(b"s5: cfg=0");

    rtc.grp0_ctrl().write(|w| unsafe { w.bits(0x1) });
    mailbox::info(b"s6: grp0_ctrl cmp0en");

    rtc.grp0_ien().write(|w| unsafe { w.bits(0) });
    rtc.grp0_if().write(|w| unsafe { w.bits(0x7) });
    mailbox::info(b"s7: ien=0 if=all");

    // 4. Enable the module through EN_SET (base + 0x1000), then start.
    let en_set = (pac::Sysrtc0S::ptr() as usize + 0x1000) as *mut u32;
    unsafe { core::ptr::write_volatile(en_set, 1) };
    mailbox::info(b"s8: en_set=1");

    rtc.cmd().write(|w| unsafe { w.bits(0x1) });
    mailbox::info(b"s9: cmd=start");

    rtc.cnt().write(|w| unsafe { w.bits(0) });
    mailbox::info(b"s10: cnt=0");

    unsafe { NVIC::unmask(pac::Interrupt::SYSRTC_APP) };
    mailbox::info(b"s11: nvic unmasked");

    // 5. Does it count?
    let a = rtc.cnt().read().bits();
    for _ in 0..300_000 {
        core::hint::spin_loop();
    }
    let b = rtc.cnt().read().bits();
    let running = rtc.status().read().bits() & 1 != 0;

    let mut l = [0u8; 40];
    l[0..4].copy_from_slice(b"s12:");
    l[4..10].copy_from_slice(b" a=0x");
    for i in 0..8 {
        let nib = (a >> (28 - 4 * i)) & 0xf;
        l[10 + i] = if nib < 10 {
            b'0' + nib as u8
        } else {
            b'a' + (nib - 10) as u8
        };
    }
    l[18..23].copy_from_slice(b" b=0x");
    for i in 0..8 {
        let nib = (b >> (28 - 4 * i)) & 0xf;
        l[23 + i] = if nib < 10 {
            b'0' + nib as u8
        } else {
            b'a' + (nib - 10) as u8
        };
    }
    l[31] = b' ';
    l[32] = b'r';
    l[33] = b'u';
    l[34] = b'n';
    l[35] = b'=';
    l[36] = b'0' + running as u8;

    if b != a && running {
        mailbox::pass(&l[..37]);
    } else {
        mailbox::fail(&l[..37]);
    }

    loop {
        core::hint::spin_loop();
    }
}
