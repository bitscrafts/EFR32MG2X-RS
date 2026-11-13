//! A tick timer over SYSRTC.
//!
//! This is a HAL service, not a vendor glue layer, and the distinction is the whole point: SYSRTC is one
//! peripheral and Bluetooth, Zigbee and Thread all schedule their own events on it. A timer written as a
//! `sl_sleeptimer` shim would make Zigbee need a second driver for the same hardware, so the timer lives
//! here and the vendor stack's entry point is a thin adapter over this.
//!
//! The initialisation order is the vendor's (`sl_hal_sysrtc_init` + `sleeptimer_hal_init_timer` from
//! Gecko SDK v4.4.5), because the first version of this module was written from the register map alone and
//! was wrong in a way worth recording: it never set `EN`, so `CMD` and `STATUS` were written to a disabled
//! module and the core faulted. A disabled peripheral does not ignore you politely.
//!
//! Order, and why each step exists:
//!
//! 1. bus clock (`CMU_CLKEN0.SYSRTC0`) -- without it every later access is a bus fault;
//! 2. counter clock (`CMU_SYSRTC0CLKCTRL.CLKSEL = LFRCO`), 32.768 kHz;
//! 3. wait for `SYNCBUSY` to clear, then stop and clear `EN` if the module was already running;
//! 4. `CFG` (debug-run off);
//! 5. group 0: compare channel 0 enabled, free-running counter;
//! 6. interrupts: all disabled and cleared, so the first event is a real one;
//! 7. `EN_SET`, then `CMD = START`;
//! 8. counter to zero;
//! 9. `SYSRTC_APP_IRQn` unmasked in the NVIC.
//!
//! One tick is 1/32768 s. This module exposes ticks rather than microseconds on purpose: converting is a
//! policy decision for whoever has a reason to prefer one clock, and hiding it here would hide the decision.

use cortex_m::peripheral::NVIC;
use efr32mg24_pac::{
    cmu_s::RegisterBlock as CmuBlock, sysrtc0_s::RegisterBlock, CmuS, Interrupt, Sysrtc0S,
};

/// Bit positions from `efr32mg24_sysrtc.h` and `efr32mg24_cmu.h`.
mod bits {
    /// `SYSRTC_EN.EN`
    pub const EN_EN: u32 = 1 << 0;
    /// `SYSRTC_STATUS.RUNNING`
    pub const STATUS_RUNNING: u32 = 1 << 0;
    /// `SYSRTC_CMD.START` (bit 0) and `.STOP` (bit 1); write 1 to act.
    pub const CMD_START: u32 = 1 << 0;
    pub const CMD_STOP: u32 = 1 << 1;
    /// `SYSRTC_GRP0_CTRL.CMP0EN`, bit 0.
    pub const CTRL_CMP0EN: u32 = 1 << 0;
    /// `SYSRTC_GRP0_IF/IEN` bit 0: compare channel 0 event; 0x7 is every channel of the group.
    pub const GRP0_CMP0: u32 = 1 << 0;
    pub const GRP0_ALL: u32 = 0x7;
    /// `CMU_SYSRTC0CLKCTRL.CLKSEL`: 1 = LFRCO.
    pub const CLKSEL_LFRCO: u32 = 1 << 0;
    pub const CLKSEL_MASK: u32 = 0x3;
    /// The `EN_SET` / `EN_CLR` aliases live one page above the register block.
    pub const ALIAS_OFFSET: usize = 0x1000;
}

/// The SYSRTC peripheral, once.
///
/// Holds a pointer rather than a borrow because `Board` owns the PAC for the life of the program: a
/// `Sysrtc<'_>` tied to `&self` could not be stored in the static slot the vendor stack's timer callbacks
/// need, and storing a borrow there is the same thing with extra ceremony.
pub struct Sysrtc {
    regs: *const RegisterBlock,
}

// SAFETY: the register block is at a fixed address for the life of the program and every method takes
// `&self`. A second value is prevented by `Board::take()` being the only constructor.
unsafe impl Send for Sysrtc {}

impl Sysrtc {
    /// Take the peripheral and put it in a known state: LFRCO, compare channel 0 armed, counter running.
    pub fn take(cmu: &CmuS, rtc: &Sysrtc0S) -> Self {
        let cmu_regs: &CmuBlock = cmu;
        cmu_regs.clken0().modify(|_, w| w.sysrtc0().set_bit());
        cmu_regs
            .sysrtc0clkctrl()
            .modify(|r, w| unsafe { w.bits((r.bits() & !bits::CLKSEL_MASK) | bits::CLKSEL_LFRCO) });

        let regs: &RegisterBlock = rtc;
        let this = Self { regs };

        this.wait_ready();

        // A module left running by a warm reset is stopped and disabled before being reconfigured.
        if this.regs_ref().en().read().bits() & bits::EN_EN != 0 {
            this.write_cmd(bits::CMD_STOP);
            this.write_en_alias(bits::ALIAS_OFFSET + 4, 0); // EN_CLR
            this.wait_ready();
        }

        // CFG: debug-run off, so the counter stops when the core is halted by the debugger.
        this.regs_ref().cfg().write(|w| unsafe { w.bits(0) });

        // Group 0, compare channel 0 on a free-running counter: the event fires once at the value the
        // caller programmes, which is what a scheduler needs.
        this.regs_ref()
            .grp0_ctrl()
            .write(|w| unsafe { w.bits(bits::CTRL_CMP0EN) });

        // Every group interrupt off and cleared, so the first wake is a real deadline and not a leftover.
        this.regs_ref().grp0_ien().write(|w| unsafe { w.bits(0) });
        this.regs_ref()
            .grp0_if()
            .write(|w| unsafe { w.bits(bits::GRP0_ALL) });

        // Enable the module, then start the counter: two writes, in this order. Starting before enabling
        // is what the first version of this function did, and it faults.
        this.write_en_alias(bits::ALIAS_OFFSET, bits::EN_EN); // EN_SET
        this.wait_ready();
        this.write_cmd(bits::CMD_START);

        this.regs_ref().cnt().write(|w| unsafe { w.bits(0) });

        // SAFETY: the handler in this crate is the only owner of this interrupt line.
        unsafe { NVIC::unmask(Interrupt::SYSRTC_APP) };

        this
    }

    #[inline]
    fn regs_ref(&self) -> &RegisterBlock {
        // SAFETY: fixed peripheral address, owned by `Board` for the life of the program.
        unsafe { &*self.regs }
    }

    /// Write one of the `EN_SET` / `EN_CLR` aliases (register block offset plus the alias page).
    #[inline]
    fn write_en_alias(&self, offset: usize, value: u32) {
        let addr = (self.regs as usize + offset) as *mut u32;
        // SAFETY: the alias region is part of the same peripheral's address space.
        unsafe { core::ptr::write_volatile(addr, value) };
    }

    /// Block until the peripheral has finished its last clock-domain crossing.
    ///
    /// Every write that crosses into the counter's clock domain sets a `SYNCBUSY` bit; the next write is
    /// ignored if it lands while that bit is set. This is why the vendor wraps each step in a wait.
    fn wait_ready(&self) {
        let mut spins = 0u32;
        while self.regs_ref().syncbusy().read().bits() != 0 {
            spins = spins.wrapping_add(1);
            if spins == 100_000 {
                // A peripheral that never synchronises is a clock problem, and spinning forever would hide
                // it behind a hang. Leaving the loop lets the caller's next access fault visibly.
                break;
            }
        }
    }

    fn write_cmd(&self, value: u32) {
        self.regs_ref().cmd().write(|w| unsafe { w.bits(value) });
    }

    /// The counter, in ticks of 1/32768 s.
    #[inline]
    pub fn now(&self) -> u32 {
        self.regs_ref().cnt().read().bits()
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

    /// Fire an interrupt when the counter reaches `deadline`.
    ///
    /// Absolute, not a delay: a scheduler computes `now + timeout` once, so the handler cannot be raced by
    /// the arithmetic.
    #[inline]
    pub fn schedule_at(&self, deadline: u32) {
        self.regs_ref()
            .grp0_cmp0value()
            .write(|w| unsafe { w.bits(deadline) });
        self.regs_ref()
            .grp0_ien()
            .write(|w| unsafe { w.bits(bits::GRP0_CMP0) });
    }

    /// Schedule `ticks` from now.
    #[inline]
    pub fn schedule_in(&self, ticks: u32) {
        self.schedule_at(self.now().wrapping_add(ticks));
    }

    /// The compare value currently programmed.
    #[inline]
    pub fn deadline(&self) -> u32 {
        self.regs_ref().grp0_cmp0value().read().bits()
    }

    /// True if compare channel 0 fired, and acknowledge it.
    #[inline]
    pub fn take_compare_event(&self) -> bool {
        let fired = self.regs_ref().grp0_if().read().bits() & bits::GRP0_CMP0 != 0;
        if fired {
            self.regs_ref()
                .grp0_if()
                .write(|w| unsafe { w.bits(bits::GRP0_CMP0) });
        }
        fired
    }

    /// Ticks per millisecond at the LFRCO rate, rounded down.
    ///
    /// 32.768 kHz is not a multiple of 1000: 1 ms is 32 ticks, and a thousand of them lose 768 ticks, about
    /// 23 ms per second. The error is stated rather than hidden, because a caller that needs better must
    /// select the crystal (LFXO) instead of adjusting this number.
    pub const fn ticks_per_ms() -> u32 {
        32
    }

    /// Milliseconds represented by `ticks`, rounded down.
    pub const fn ticks_to_ms(ticks: u32) -> u32 {
        ticks / Self::ticks_per_ms()
    }
}
