//! A tick timer over SYSRTC.
//!
//! A HAL service, not a vendor glue layer: one peripheral, three protocols (Bluetooth, Zigbee, Thread), one
//! driver. The vendor stack's entry point is a thin adapter over this.
//!
//! # The two offsets that cost forty flash cycles
//!
//! Measured 2026-11-13 with the vendor headers themselves (`offsetof` against `efr32mg24_cmu.h` and
//! `efr32mg24_sysrtc.h`), because both were wrong in the code paths this module first used:
//!
//! ```text
//! CMU.SYSRTC0CLKCTRL   0x240    the PAC's accessor writes 0x1DC -- a different register entirely
//! SYSRTC.EN_SET        0x1004   the alias page mirrors the layout; 0x1000 is IPVERSION_SET (read-only)
//! ```
//!
//! The consequences were not obvious from the symptoms. Writing the clock select to `0x1DC` left
//! `SYSRTC0CLKCTRL` at its disabled value, so the counter's clock never ran; a peripheral with no clock
//! cannot accept a synchronised write, and `CMD = START` came back as `CFSR = PRECISERR` with
//! `BFAR = SYSRTC0->CMD` — a bus error that looked like a fault in the driver's logic. Writing `1` to
//! `0x1000` hit `IPVERSION_SET`, which is read-only, so `EN` stayed 0 and the module never enabled.
//!
//! Both are reached here through raw pointers with the address written next to the write, because the PAC
//! cannot be trusted for these two registers and a comment is cheaper than another forty cycles.
//!
//! # Order
//!
//! The vendor's (`sl_hal_sysrtc_init` + `sleeptimer_hal_init_timer`, Gecko SDK v4.4.5):
//! bus clocks, clock select, wait, `CFG`, group config, interrupts off and cleared, `EN_SET`, `CMD = START`.
//! Every synchronised register is read or written across a `SYNCBUSY` wait.

use efr32mg24_pac::{
    cmu_ns::RegisterBlock as CmuBlock, sysrtc0_ns::RegisterBlock, CmuNs, Sysrtc0Ns,
};

/// Offsets measured against the vendor headers (see the module note).
mod off {
    /// `CMU_SYSRTC0CLKCTRL`, relative to the CMU base.
    pub const CMU_SYSRTC0CLKCTRL: usize = 0x240;
    /// `CMU_CLKEN0`, relative to the CMU base.
    pub const CMU_CLKEN0: usize = 0x64;
    /// `SYSRTC_EN_SET`, the write-1-to-set alias of `EN`.
    pub const EN_SET: usize = 0x1004;
    /// `SYSRTC_EN_CLR`, the write-1-to-clear alias. Disabling must use this; writing 0 to `EN_SET` does nothing.
    pub const EN_CLR: usize = 0x2004;
    /// The group block, at the offsets the RAIL blob uses (`SYSRTC_SEQ_IRQHandler` reads `+0x60` and `+0x64`).
    /// Every SDK header puts these at 0x40 and is wrong for this silicon.
    pub const GRP0_IF: usize = 0x60;
    pub const GRP0_IEN: usize = 0x64;
    pub const GRP0_CTRL: usize = 0x68;
    pub const GRP0_CMP0VALUE: usize = 0x6C;
}

mod bits {
    /// `SYSRTC_EN.EN`
    pub const EN_EN: u32 = 1 << 0;
    /// `SYSRTC_STATUS.RUNNING`
    pub const STATUS_RUNNING: u32 = 1 << 0;
    /// `SYSRTC_CMD.START` (bit 0) and `.STOP` (bit 1).
    pub const CMD_START: u32 = 1 << 0;
    pub const CMD_STOP: u32 = 1 << 1;
    /// `SYSRTC_GRP0_CTRL.CMP0EN`.
    pub const CTRL_CMP0EN: u32 = 1 << 0;
    /// `SYSRTC_GRP0_IF/IEN`, one bit per channel, **shifted by one against every header**. Measured on the board:
    /// a compare on channel 0 sets bit 1 (0x22 read back, with the group-level mirror also set) exactly one count
    /// after the programmed value, and a compare on channel 1 sets bit 2 (0x44). With only CMP0 enabled the flag
    /// appeared in bit 1 and never in bit 0 — so the per-channel flag is bit `1 + n`, and the headers are off by
    /// one here just as they are for the group's base offset.
    pub const GRP0_CMP0: u32 = 1 << 1;
    /// Compare channel 1's flag, same shift.
    pub const GRP0_CMP1: u32 = 1 << 2;
    /// Bits 4..7 mirror the channel flags at group level. Clearing them all is harmless and stops a stale
    /// aggregate from looking like a pending event.
    pub const GRP0_ALL: u32 = 0xFF;
    /// `CMU_CLKEN0` bits: LFRCO (22) and SYSRTC0 (30).
    pub const CLKEN0_LFRCO: u32 = 1 << 22;
    pub const CLKEN0_SYSRTC0: u32 = 1 << 30;
    /// `CMU_SYSRTC0CLKCTRL.CLKSEL`: 0 = disabled, 1 = LFRCO.
    pub const CLKSEL_LFRCO: u32 = 1;
    pub const CLKSEL_MASK: u32 = 0x3;
}

/// One 32-bit register write at an absolute address, for the registers the PAC models wrongly.
#[inline]
fn wr(addr: usize, value: u32) {
    // SAFETY: the caller passes an address inside a peripheral this module owns.
    unsafe { core::ptr::write_volatile(addr as *mut u32, value) };
}

#[inline]
fn rd(addr: usize) -> u32 {
    // SAFETY: as `wr`.
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

/// The SYSRTC peripheral, once.
pub struct Sysrtc {
    regs: *const RegisterBlock,
    cmu: usize,
}

// SAFETY: fixed peripheral addresses for the life of the program; every method takes `&self`, and a second
// value is prevented by `Board::sysrtc` being reachable only from the singleton `Board`.
unsafe impl Send for Sysrtc {}

impl Sysrtc {
    /// Take the peripheral and put it in a known state: LFRCO, compare channel 0 armed, counter running.
    pub fn take(cmu: &CmuNs, rtc: &Sysrtc0Ns) -> Self {
        let cmu_regs: &CmuBlock = cmu;
        let cmu_base = cmu_regs as *const CmuBlock as usize;
        let regs: &RegisterBlock = rtc;
        let this = Self {
            regs,
            cmu: cmu_base,
        };

        // 1. Bus clocks: SYSRTC's, and the LFRCO's own (bit 22) -- emlib's CMU_OscillatorEnable does the
        //    same, and without it a clock source can be selected that is not actually running.
        let clken0 = (this.cmu + off::CMU_CLKEN0) as *mut u32;
        let gates =
            unsafe { core::ptr::read_volatile(clken0) } | bits::CLKEN0_LFRCO | bits::CLKEN0_SYSRTC0;
        unsafe { core::ptr::write_volatile(clken0, gates) };

        // 2. Clock select, at the offset the header gives. 0 would mean disabled, which is what the wrong
        //    offset left behind.
        let clkctrl = (this.cmu + off::CMU_SYSRTC0CLKCTRL) as *mut u32;
        let current = unsafe { core::ptr::read_volatile(clkctrl) };
        unsafe {
            core::ptr::write_volatile(clkctrl, (current & !bits::CLKSEL_MASK) | bits::CLKSEL_LFRCO)
        };

        this.wait_sync();

        // 3. A module left running by a warm reset is stopped and disabled before being reconfigured.
        if this.regs_ref().en().read().bits() & bits::EN_EN != 0 {
            this.write_cmd(bits::CMD_STOP);
            this.wait_sync();
            // Disable through EN_CLR (0x2004). Writing 0 to EN_SET does nothing, which is how a warm reset
            // could leave the module running while we reconfigured it.
            wr(this.regs as usize + off::EN_CLR, bits::EN_EN);
            this.wait_sync();
        }

        // 4. CFG: debug-run off, so the counter stops when the core is halted.
        this.regs_ref().cfg().write(|w| unsafe { w.bits(0) });

        // 5. Group 0, compare channel 0, free-running counter, at the offsets the RAIL blob itself uses:
        //    GRP0_IF 0x60, GRP0_IEN 0x64, GRP0_CTRL 0x68. The headers put the group at 0x40 and are wrong for
        //    this silicon -- writes there were silently lost and the compare channel was never enabled.
        wr(this.regs as usize + off::GRP0_CTRL, bits::CTRL_CMP0EN);

        // 6. Every group interrupt off and cleared, so the first wake is a real deadline.
        wr(this.regs as usize + off::GRP0_IEN, 0);
        wr(this.regs as usize + off::GRP0_IF + 0x2000, bits::GRP0_ALL);

        // 7. Enable the module through EN_SET (0x1004), then start. 0x1000 is IPVERSION_SET and writing
        //    there left EN at zero, which is how the first version of this module failed.
        this.write_en_set(bits::EN_EN);
        this.wait_sync();
        this.write_cmd(bits::CMD_START);
        // The store must actually leave the core before anything reads the counter again: with a buffered
        // store the first later access is where the core runs into it, which reads as "the counter read hangs".
        unsafe { core::arch::asm!("dsb 0xf", "isb 0xf") };

        this.set_counter(0);

        // The compare value resets to zero and the counter starts at zero, so the first match is already a
        // pending event. Clearing it here matters because unmasking with a pending flag would enter the
        // handler before the caller has installed one -- and the weak default handler is an infinite loop,
        // which is indistinguishable from a driver that never returned.
        this.regs_ref()
            .grp0_if()
            .write(|w| unsafe { w.bits(bits::GRP0_ALL) });

        this
    }

    #[inline]
    fn regs_ref(&self) -> &RegisterBlock {
        // SAFETY: fixed peripheral address owned by `Board` for the life of the program.
        unsafe { &*self.regs }
    }

    #[inline]
    fn write_en_set(&self, value: u32) {
        let addr = (self.regs as usize + off::EN_SET) as *mut u32;
        // SAFETY: the alias region belongs to the same peripheral.
        unsafe { core::ptr::write_volatile(addr, value) };
    }

    fn write_cmd(&self, value: u32) {
        self.regs_ref().cmd().write(|w| unsafe { w.bits(value) });
    }

    /// Block until the peripheral has finished its last clock-domain crossing.
    ///
    /// Every synchronised register sets a `SYNCBUSY` bit; while it is set, the next access is either
    /// ignored or stalls the bus. The bound turns a missing clock into a visible fault instead of a hang.
    fn wait_sync(&self) {
        let mut spins = 0u32;
        while self.regs_ref().syncbusy().read().bits() != 0 {
            spins = spins.wrapping_add(1);
            if spins == 200_000 {
                break;
            }
        }
    }

    /// Write the counter, across a synchronisation wait.
    fn set_counter(&self, value: u32) {
        self.wait_sync();
        self.regs_ref().cnt().write(|w| unsafe { w.bits(value) });
        self.wait_sync();
    }

    /// The counter, in ticks of 1/32768 s, read across a synchronisation wait.
    #[inline]
    pub fn now(&self) -> u32 {
        self.wait_sync();
        let value = self.regs_ref().cnt().read().bits();
        self.wait_sync();
        value
    }

    /// True while the counter is running.
    #[inline]
    pub fn is_running(&self) -> bool {
        self.regs_ref().status().read().bits() & bits::STATUS_RUNNING != 0
    }

    /// True while the module is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.regs_ref().en().read().bits() & bits::EN_EN != 0
    }

    /// Fire an interrupt when the counter reaches `deadline` (absolute, not a delay).
    #[inline]
    pub fn schedule_at(&self, deadline: u32) {
        self.wait_sync();
        wr(self.regs as usize + off::GRP0_CMP0VALUE, deadline);
        wr(self.regs as usize + off::GRP0_IEN, bits::GRP0_CMP0);
    }

    /// Schedule `ticks` from now.
    #[inline]
    pub fn schedule_in(&self, ticks: u32) {
        self.schedule_at(self.now().wrapping_add(ticks));
    }

    /// True if compare channel 0 fired, and acknowledge it.
    #[inline]
    pub fn take_compare_event(&self) -> bool {
        let fired = rd(self.regs as usize + off::GRP0_IF) & bits::GRP0_CMP0 != 0;
        if fired {
            // Clear through the write-1-to-clear alias at +0x2000: a plain store of 0 to the flag register leaves it
            // set, which is how a deadline can look "already fired" on the next call.
            wr(
                self.regs as usize + off::GRP0_IF + 0x2000,
                bits::GRP0_CMP0 | bits::GRP0_CMP1,
            );
        }
        fired
    }

    /// Unmask the module's interrupt line.
    ///
    /// Separate from `take` on purpose: unmasking before a handler exists sends the first event to the weak
    /// default handler, which loops forever.
    pub fn unmask_interrupt(&self) {
        // SAFETY: the caller installs `SYSRTC_APP_IRQHandler` before calling this.
        unsafe { cortex_m::peripheral::NVIC::unmask(efr32mg24_pac::Interrupt::SYSRTC_APP) };
    }

    /// Ticks per millisecond at LFRCO, rounded down: 32.768 kHz is not a multiple of 1000, so a thousand
    /// milliseconds lose 768 ticks (about 23 ms/s). A caller that needs better must select the crystal.
    pub const fn ticks_per_ms() -> u32 {
        32
    }

    /// Milliseconds represented by `ticks`, rounded down.
    pub const fn ticks_to_ms(ticks: u32) -> u32 {
        ticks / Self::ticks_per_ms()
    }
}
