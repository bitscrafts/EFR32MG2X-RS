//! `BURTC`: the backup counter, the one that keeps time when the rest of the part is off.
//!
//! `SYSRTC` is the counter this crate uses for ordinary timekeeping, and it stops in EM4. `BURTC` lives in the backup
//! domain, runs from the low-frequency branch, and is the counter a product reads after a hibernate to learn how long
//! the part was asleep. It is a separate block with its own clock mux (`CMU_EM4GRPACLKCTRL`), its own bus gate
//! (`CMU_CLKEN0.BURTC`) and its own enable/start sequence -- a driver copied from `sysrtc` compiles and counts nothing.
//!
//! # What this driver encodes
//!
//! The sequence is the one measured on the part, and each step is a line of [`Burtc::new`]:
//!
//! 1. **`LOCK` before `CFG`.** A wrong key leaves the configuration registers writable-or-not without saying so; the
//!    unlock key is written first so that the `CFG` write that follows is not silently dropped.
//! 2. **The bus gate is not the clock.** `CMU_CLKEN0.BURTC` lets the module answer on the bus; the counting clock is
//!    `EM4GRPACLK`, selected separately. With the gate set and no source, the registers read back and the counter never
//!    moves -- exactly the state [`remove_source`] puts the block in, and the control the instrument uses.
//! 3. **`EN`, then `CMD.START`.** `CMD.START` is a command that crosses into the counter's clock domain, and
//!    `SYNCBUSY.START` is the part of the crossing that can be seen. The driver waits for it, bounded, and then for
//!    `STATUS.RUNNING`.
//! 4. **The backup domain outlives a core reset.** The block can arrive already enabled and running from the firmware
//!    that ran before, and `CFG` is not writable in that state -- the first boot of this driver died on a precise bus
//!    fault at `BURTC_NS + 0x08` (`CFSR=0x8200`, `BFAR=0x50064008`). `new` and [`Burtc::set_prescaler`] therefore drop
//!    `EN`, wait out `EN.DISABLING`, write `CFG`, and enable again.
//! 5. **`CNTPRESC` is a power of two**, `0` = divide by 1 through `15` = divide by 32768. Written and read back:
//!    [`Burtc::set_prescaler`] refuses rather than returning with a field that did not take.
//! 6. **`COMPTOP` is left clear**, so the top is the whole 32-bit counter and nothing wraps early. The compare channel
//!    is not driven here: there is no instrument for its interrupt in this crate, and a driver without one is a claim.
//!
//! # What this driver does not do
//!
//! EM4 entry and the retention of the counter across it belong to `emu`; the counter is only read here. The overflow
//! interrupt is not exposed for the same reason the compare channel is not.
//!
//! The measured record, the control and what each run rules out are in `docs/04-reference/BURTC-2026-11-27.md`.

use efr32mg2x_pac as pac;

/// Ticks per second of the source this driver selects: LFRCO, 32.768 kHz, the same branch `SYSRTC` runs from.
pub const SOURCE_HZ: u32 = 32_768;

/// The most times a synchronisation wait will look at its status bit before giving up. Each poll is one register read;
/// a wait that never ends is indistinguishable from a hang, which is the defect this bound exists to avoid.
const SYNC_POLLS: u32 = 1_000_000;

/// Why a `BURTC` operation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// `CNTPRESC` is four bits: 0 (divide by 1) to 15 (divide by 32768).
    PrescalerOutOfRange {
        /// The value that was asked for.
        value: u8,
    },
    /// `CFG.CNTPRESC` was written and read back different, so the counter would run at a rate nobody asked for.
    PrescalerRefused {
        /// What was written.
        wrote: u8,
        /// What the register returned.
        read: u8,
    },
    /// `CMD.START` was written and `STATUS.RUNNING` never came up.
    NeverStarted,
    /// `CMD.STOP` was written and `STATUS.RUNNING` never went away.
    NeverStopped,
    /// `EN` was written and `EN.DISABLING` never cleared.
    NeverEnabled,
    /// `EN` was cleared and `EN.DISABLING` never went away, so `CFG` cannot be written.
    NeverDisabled,
}

/// The backup counter, owned by one caller.
pub struct Burtc {
    regs: pac::BurtcNs,
}

/// What the clock tree says about this counter, read back rather than assumed.
///
/// Bit 0 is `CMU_CLKEN0.BURTC`; bits 8..11 are `CMU_EM4GRPACLKCTRL.CLKSEL` (`0` no source, `1` LFRCO, `2` LFXO,
/// `3` ULFRCO). A record that carries both can tell a branch that was never selected from a module that was enabled and
/// did not run -- two defects that look identical from the counter alone.
#[must_use]
pub fn clock_readback() -> u32 {
    // SAFETY: two CMU registers, read only. `Board` owns the CMU for the life of the program and a read changes nothing.
    let cmu = unsafe { pac::CmuS::steal() };
    let gate = u32::from(cmu.clken0().read().burtc().bit_is_set());
    let sel = cmu.em4grpaclkctrl().read().clksel().bits();
    gate | ((u32::from(sel) & 0xF) << 8)
}

/// Point `EM4GRPACLK` at LFRCO, the 32.768 kHz branch `SYSRTC` also runs from.
///
/// Selecting a source the part does not have running is the defect this call exists to avoid: the mux accepts the
/// write, the registers still answer, and the counter stays where it was.
pub fn select_lfrco() {
    // SAFETY: `EM4GRPACLKCTRL` is one write-only-to-this-field register in a CMU block the caller has already handed
    // out through `Board`; the write replaces `CLKSEL` and nothing else.
    let cmu = unsafe { pac::CmuS::steal() };
    cmu.em4grpaclkctrl().modify(|_, w| w.clksel().lfrco());
}

/// Remove the source from `EM4GRPACLK` (`CLKSEL = 0`), leaving the module enabled and its registers reachable.
///
/// This is the control for "the counter is counting because it is clocked": the block is otherwise untouched, and the
/// counter must stop dead. [`select_lfrco`] puts the source back, so the same call also proves the change is the cause
/// in both directions instead of proving that a counter can be stopped.
pub fn remove_source() {
    // SAFETY: same register and same argument as `select_lfrco`: only `CLKSEL` is written.
    let cmu = unsafe { pac::CmuS::steal() };
    cmu.em4grpaclkctrl()
        .modify(|_, w| unsafe { w.clksel().bits(0) });
}

impl Burtc {
    /// Configure and start the counter, with the given prescaler.
    ///
    /// # Safety
    ///
    /// The caller must own `BURTC` for the life of this value: two drivers writing `CFG` interleave into a rate neither
    /// of them asked for. `Board::burtc` is the only caller and goes through the claim table.
    pub(crate) unsafe fn new(regs: pac::BurtcNs, prescaler: u8) -> Result<Self, Error> {
        if prescaler > 0xF {
            return Err(Error::PrescalerOutOfRange { value: prescaler });
        }
        let mut this = Self { regs };
        // `LOCK` first: a wrong key leaves the configuration registers locked and the write after it is silently dropped.
        this.regs.lock().write(|w| w.lockkey().unlock());
        // The backup domain survives a core reset, so the counter may arrive here already enabled and running from
        // whatever firmware ran before -- and `CFG` is not writable in that state. Measured: a precise bus fault at
        // `BURTC_NS + 0x08` (`CFSR=0x8200`, `BFAR=0x50064008`) on the first boot of this driver, because the previous
        // image had left `EN` set. `CMD` is not writable once `EN` is low either (`BFAR=0x5006400c`), so the stop comes
        // first, while the block is still enabled, and only then is the enable dropped and waited out.
        this.stop_if_running()?;
        this.disable()?;
        this.regs.cfg().write(|w| {
            // SAFETY: `prescaler` was range-checked against the four-bit field above, so no write can exceed it.
            unsafe { w.cntpresc().bits(prescaler) }
                // Top is the whole 32-bit counter: nothing wraps early, and the compare channel is not in the path.
                .comptop()
                .disable()
                // Debug mode does not stop time: a record read over the debugger must not be taken on a frozen counter.
                .debugrun()
                .x1()
        });
        this.enable()?;
        this.regs.cmd().write(|w| w.start().set_bit());
        this.wait_started()?;
        Ok(this)
    }

    /// Drop `EN` and wait, bounded, for `EN.DISABLING` to clear. A block that is already disabled is left alone: the
    /// write is not needed and `CMD` is not reachable from the disabled state.
    fn disable(&mut self) -> Result<(), Error> {
        if !self.regs.en().read().en().bit_is_set() {
            return Ok(());
        }
        self.regs.en().write(|w| w.en().clear_bit());
        for _ in 0..SYNC_POLLS {
            if !self.regs.en().read().disabling().bit_is_set() {
                return Ok(());
            }
        }
        Err(Error::NeverDisabled)
    }

    /// Set `EN` and wait, bounded, for `EN.DISABLING` to clear. Already enabled is a no-op.
    fn enable(&mut self) -> Result<(), Error> {
        if self.regs.en().read().en().bit_is_set() {
            return Ok(());
        }
        self.regs.en().write(|w| w.en().set_bit());
        for _ in 0..SYNC_POLLS {
            if !self.regs.en().read().disabling().bit_is_set() {
                return Ok(());
            }
        }
        Err(Error::NeverEnabled)
    }

    /// Stop only if the counter says it is running: `CMD` is not writable with `EN` low, so an unconditional stop is a
    /// fault on a block that arrived disabled.
    fn stop_if_running(&mut self) -> Result<(), Error> {
        if self.is_running() {
            self.stop()?;
        }
        Ok(())
    }

    /// Wait, bounded, for `SYNCBUSY.START` to clear and then for `STATUS.RUNNING`.
    fn wait_started(&self) -> Result<(), Error> {
        for _ in 0..SYNC_POLLS {
            if !self.regs.syncbusy().read().start().bit_is_set()
                && self.regs.status().read().running().bit_is_set()
            {
                return Ok(());
            }
        }
        Err(Error::NeverStarted)
    }

    /// The counter, in ticks of the counting clock after the prescaler.
    ///
    /// Read until two consecutive reads agree (at most four), because `CNT` is a synchronised register: the first read
    /// asks for the crossing, the next one returns the settled value.
    #[must_use]
    pub fn counter(&self) -> u32 {
        let mut value = self.regs.cnt().read().bits();
        for _ in 0..3 {
            let again = self.regs.cnt().read().bits();
            if again == value {
                break;
            }
            value = again;
        }
        value
    }

    /// Whether the counter is running, from `STATUS.RUNNING`.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.regs.status().read().running().bit_is_set()
    }

    /// The prescaler in the register, read back.
    #[must_use]
    pub fn prescaler(&self) -> u8 {
        self.regs.cfg().read().cntpresc().bits()
    }

    /// The counting clock divided by this, in hertz: `SOURCE_HZ / 2^prescaler`.
    #[must_use]
    pub fn tick_hz(&self) -> u32 {
        SOURCE_HZ >> self.prescaler()
    }

    /// Set the prescaler (`0` = divide by 1, `15` = divide by 32768) and check that the field took it.
    ///
    /// The register is written, then read back; a field that comes back different is an error and not a silent run at
    /// the old rate. Stop the counter first if the rate matters from the first tick after this call: nothing in the
    /// hardware stops a write while it runs, and the tick in flight is neither the old period nor the new one.
    pub fn set_prescaler(&mut self, prescaler: u8) -> Result<(), Error> {
        if prescaler > 0xF {
            return Err(Error::PrescalerOutOfRange { value: prescaler });
        }
        let was_running = self.is_running();
        // `CFG` is writable only with `EN` low, and `CMD` only with `EN` high: the enable is cycled around the write and
        // the counter started again if it was running, so the rate changes without the caller having to know the order.
        self.stop_if_running()?;
        self.disable()?;
        // SAFETY: `prescaler` was range-checked against the four-bit field at the top of this function.
        self.regs
            .cfg()
            .modify(|_, w| unsafe { w.cntpresc().bits(prescaler) });
        let read = self.prescaler();
        self.enable()?;
        if was_running {
            self.start()?;
        }
        if read != prescaler {
            return Err(Error::PrescalerRefused {
                wrote: prescaler,
                read,
            });
        }
        Ok(())
    }

    /// Stop the counter, waiting, bounded, for `STATUS.RUNNING` to go low. A counter already stopped is left alone.
    pub fn stop(&mut self) -> Result<(), Error> {
        if !self.is_running() {
            return Ok(());
        }
        self.regs.cmd().write(|w| w.stop().set_bit());
        for _ in 0..SYNC_POLLS {
            if !self.regs.status().read().running().bit_is_set() {
                return Ok(());
            }
        }
        Err(Error::NeverStopped)
    }

    /// Start a counter that was stopped, with the same wait as [`Burtc::new`].
    pub fn start(&mut self) -> Result<(), Error> {
        self.regs.cmd().write(|w| w.start().set_bit());
        self.wait_started()
    }

    /// `IF` as it stands: bit 0 overflow, bit 1 compare. Read only; clearing is not exposed because nothing here arms
    /// the interrupt.
    #[must_use]
    pub fn flags(&self) -> u32 {
        self.regs.if_().read().bits()
    }

    /// `SYNCBUSY` as it stands, for a record that wants to say whether the block was still crossing a domain.
    #[must_use]
    pub fn syncbusy(&self) -> u32 {
        self.regs.syncbusy().read().bits()
    }

    /// `STATUS` as it stands: bit 0 running, bit 1 configuration locked.
    #[must_use]
    pub fn status(&self) -> u32 {
        self.regs.status().read().bits()
    }
}
