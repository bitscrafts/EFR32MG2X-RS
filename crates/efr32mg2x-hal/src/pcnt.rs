//! `PCNT0`: pulses counted by a peripheral, with the core doing nothing per event.
//!
//! ## Why this module exists
//!
//! The PRS producer measured on 2026-11-26 carries a pad's level, and an IADC converts once per rising edge of it
//! (`docs/04-reference/PRS-GPIO-PRODUCER-2026-11-26.md`). That proves the signal reaches a consumer; it does not prove
//! anything **counts** it, and the converter is a poor counter: each conversion costs a FIFO slot, a pop and a
//! microsecond of settling. `PCNT0` is the block built for it — a pulse counter whose S0IN is one of the PRS consumers
//! (`CONSUMER_PCNT0_S0IN`), so the same edge that converted a sample also increments a register.
//!
//! ## What was measured, and what follows from it
//!
//! - **The PRS path counts.** Eight pull moves of a pad (four rising edges) gave four, an odd number of moves split
//!   between the two polarities gave the split exactly, and the four controls gave zero
//!   (`docs/04-reference/PCNT-GPIO-2026-11-26.md`).
//! - **The ceiling is a pulse-width limit, not a bandwidth.** `EM23GRPACLK` has only LFRCO, LFXO and ULFRCO in this
//!   part, so the sampler cannot be moved faster than 32.768 kHz: a pulse narrower than 30.5 us is seen with probability
//!   `width / 30.5 us` and usually not at all. A timer's overflow event is one clock wide (53 ns at 19 MHz) and gave
//!   **zero** counts at 1 kHz, where 13 edges were produced -- measured twice, with two producers
//!   (`docs/04-reference/PCNT-CEILING-2026-11-26.md`).
//! - **The GPIO producer is the pad's input.** A pad the timer drives as an output is not on the path: driving PA0 at
//!   50 % duty and reading `GpioPin(0)` back gave zero at every rate from 1 kHz to 25 kHz, with a high time seventeen
//!   sampling periods long. A producer has to be a signal presented *to* the pin.
//! - **It is sampled, so it has a ceiling.** `CFG.MODE = OVSSINGLE` samples S0IN with the `EM23GRPACLK` branch, which
//!   this board runs at 32.768 kHz. The ladder in the example finds the rate at which edges start being lost.
//! - **`CFG` is write-protected while `EN` is set**: writing it there raises a precise bus error at `PCNT0 + 0x0C`
//!   (`CFSR = 0x00008200`, `BFAR = 0x5903_000C`) and halts the core. Every configuration write below therefore happens
//!   with `EN` low, and the module waits for `EN.DISABLING` to clear both before and after.
//! - **`CMD` is synchronised and needs the wait too**: a second command inside one synchroniser period faults at
//!   `PCNT0 + 0x14`. `SYNCBUSY.CMD` is the handshake.
//! - **`CFG.MODE = EXTCLKSINGLE` with `CLKSEL = 1` does not count S0IN**: it counts the `EM23GRPACLK` branch itself
//!   (measured: 88 and 139 counts in windows sized at 2.6 and 4 ms, both ≈ 33 kHz, whatever the pad did). The
//!   external-clock mode needs the dedicated `PCNT_S0` pin as the block's clock (`CLKSEL = 2`), and pointing that at a
//!   routed GPIO left the block with no clock at all: `EN.DISABLING` stayed set and the next `CFG` write faulted. That
//!   path is not offered here, because this crate only ships what the board demonstrated.
//! - **There is no capture command.** The plan for this item said `CNT` is read through `CMD.CAPTURE`; `CMD` has reset,
//!   load, start and stop bits only and `CNT` has no writer. A running read returned the same value as the read after
//!   `CMD.STOPCNT`, so the read is live.
//! - **`TOP` resets to `0x00FF`, not `0xFFFF`**: a counter left at reset wraps after 255 events, which on a fast
//!   producer looks like a counter that cannot keep up. [`Pcnt::set_top`] is called before any count is believed.

use crate::pac;

/// Which edge of S0IN increments the counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// A low-to-high transition counts one.
    Rising,
    /// A high-to-low transition counts one.
    Falling,
}

/// Why the counter refused what it was asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The PRS consumer's channel field is four bits; there is no asynchronous channel above 15.
    NoSuchChannel,
}

/// `CMU.CONSUMER_PCNT0_S0IN` lives in PRS, not in the PCNT, but it is the counter's own routing word: it says which
/// asynchronous channel arrives at S0IN. Reaching it from here keeps the PRS block out of this crate's public API, and
/// the address is the PAC's (`prs_s`, 0x4003_8000) rather than a number chosen here — the same arrangement
/// `adc::Adc::use_prs_trigger` uses for `CONSUMER_IADC0_SINGLETRIGGER`.
const PRS_BASE: usize = 0x4003_8000;

/// `CMD.CORERST`: reset the counter's clock domain.
const CORERST: u32 = 0x001;

/// `CMD.CNTRST`: zero the count.
const CNTRST: u32 = 0x002;

/// `CMD.STARTCNT`: start the main counter.
const STARTCNT: u32 = 0x100;

/// `CMD.STOPCNT`: stop the main counter.
const STOPCNT: u32 = 0x400;

/// `PCNT0`, handed out once by [`Board::pcnt`](crate::board::Board::pcnt).
pub struct Pcnt<'a> {
    regs: pac::Pcnt0Ns,
    cmu: &'a pac::cmu_s::RegisterBlock,
}

impl<'a> Pcnt<'a> {
    /// SAFETY: `Board` holds the PAC singletons and hands this out through its claim table, so no second `Pcnt` exists.
    pub(crate) unsafe fn new(regs: pac::Pcnt0Ns, cmu: &'a pac::cmu_s::RegisterBlock) -> Self {
        Self { regs, cmu }
    }

    /// Bring the block up clean: a core reset, and the module left **disabled**.
    ///
    /// Disabled on purpose: `CFG` is refused with `EN` set (see the module documentation), so the block takes its
    /// configuration with `EN` low and is enabled before it starts counting.
    pub fn init(&mut self) {
        self.write_cmd(CORERST);
        self.wait_disabled();
    }

    /// Clear `EN` and wait for the block to report that it has stopped.
    fn disable(&mut self) {
        self.regs.en().write(|w| w.en().clear_bit());
        self.wait_disabled();
    }

    /// Wait, bounded, for `EN.DISABLING` to clear.
    ///
    /// The first read is too early by construction. Measured 2026-11-26: `EN` cleared, the very next read returned
    /// `0x2` (DISABLING set) and a `CFG` write right after it still faulted, because the first read had happened
    /// before the flag was raised. So the wait starts with a margin long enough for the branch's own clock (30 µs at
    /// 32.768 kHz) and only then polls the handshake.
    fn wait_disabled(&mut self) {
        cortex_m::asm::delay(4_000);
        let mut spin = 0u32;
        while self.regs.en().read().disabling().bit_is_set() {
            spin += 1;
            if spin > 200_000 {
                break;
            }
            cortex_m::asm::delay(64);
        }
        // The flag dropping is the last crossing to complete; give the register interface one more margin.
        cortex_m::asm::delay(64);
    }

    /// Point S0IN at a PRS channel instead of the pad, and select `channel` as its source.
    ///
    /// Both writes are needed and they live in different blocks: `CFG.S0PRSEN` tells the counter to listen to the PRS
    /// at all, and `CONSUMER_PCNT0_S0IN.PRSSEL` says which of the sixteen asynchronous channels arrives. Without the
    /// second the counter listens on channel 0 whatever the caller asked for — the same defect the IADC's trigger had
    /// and had to be fixed for.
    ///
    /// The clock select is pinned to the `EM23GRPACLK` branch here, which is the source the sampled mode uses and the
    /// source the record was taken with.
    ///
    /// # Errors
    ///
    /// [`Error::NoSuchChannel`] for a channel above 15. Refused before anything is written.
    pub fn use_prs_input(&mut self, channel: u8) -> Result<(), Error> {
        if channel > 15 {
            return Err(Error::NoSuchChannel);
        }
        let was_enabled = self.regs.en().read().en().bit_is_set();
        if was_enabled {
            self.disable();
        }
        self.cmu
            .pcnt0clkctrl()
            .modify(|_, w| w.clksel().em23grpaclk());
        self.regs.cfg().modify(|_, w| w.s0prsen().set_bit());
        // SAFETY: PRS is at a fixed address for the life of the program and this write targets the PCNT0's own consumer
        // word. `Board::pcnt` hands out one `Pcnt`, so no second owner writes it.
        let prs = unsafe { &*(PRS_BASE as *const pac::prs_s::RegisterBlock) };
        prs.consumer_pcnt0_s0in()
            // SAFETY: `PRSSEL` is a four-bit field and `channel` was checked against 15 above.
            .write(|w| unsafe { w.prssel().bits(channel) });
        if was_enabled {
            self.regs.en().write(|w| w.en().set_bit());
        }
        Ok(())
    }

    /// Count `edge`s of S0IN, from zero, starting now.
    ///
    /// `CFG.MODE = OVSSINGLE` samples the input with `EM23GRPACLK`; the counter therefore sees at most one edge per
    /// branch period, and the example's ladder measures where that ceiling bites.
    pub fn count_edges(&mut self, edge: Edge) {
        self.disable();
        self.regs.cfg().modify(|_, w| w.mode().ovssingle());
        // `CTRL` is a synchronised register: it is written while the block is disabled and the write is given the
        // synchroniser's time before anything else touches the block.
        self.sync(|s| s.ctrl().bit_is_set());
        self.regs.ctrl().modify(|_, w| {
            let w = match edge {
                Edge::Rising => w.edge().pos(),
                Edge::Falling => w.edge().neg(),
            };
            // Only up-count events increment; the direction bit is left at reset (up).
            w.cntev().up()
        });
        self.sync(|s| s.ctrl().bit_is_set());
        self.regs.en().write(|w| w.en().set_bit());
        self.write_cmd(CNTRST | STARTCNT);
    }

    /// Wait, bounded, until `busy` says the register's synchroniser is free.
    ///
    /// Measured 2026-11-26: two accesses to a synchronised register inside one synchroniser period raise a precise bus
    /// error (`BFAR = PCNT0 + 0x14`, the `CMD` register) — the block is clocked by the 32.768 kHz branch and the core
    /// writes at 19 MHz, so a second write lands while the first is still crossing. The fault is not reported as a busy
    /// flag: it halts the core. Hence the wait before every command.
    fn sync(&mut self, busy: impl Fn(&pac::pcnt0_ns::syncbusy::R) -> bool) {
        let mut spin = 0u32;
        while busy(&self.regs.syncbusy().read()) {
            spin += 1;
            if spin > 2_000_000 {
                break;
            }
        }
    }

    /// One command word, written only when `SYNCBUSY.CMD` is clear.
    fn write_cmd(&mut self, word: u32) {
        self.sync(|s| s.cmd().bit_is_set());
        // SAFETY: the word is a combination of the documented single-bit command fields; no other bit is set.
        self.regs.cmd().write(|w| unsafe { w.bits(word) });
        self.sync(|s| s.cmd().bit_is_set());
    }

    /// The counter's wrap value. Reset is `0x00FF`; call this before trusting a count above 255.
    ///
    /// Written with `EN` low, like every other configuration write.
    pub fn set_top(&mut self, top: u16) {
        let was_enabled = self.regs.en().read().en().bit_is_set();
        if was_enabled {
            self.disable();
        }
        // SAFETY: `TOP` is a 16-bit field and `top` is a `u16`.
        self.sync(|s| s.top().bit_is_set());
        self.regs.top().write(|w| unsafe { w.top().bits(top) });
        self.sync(|s| s.top().bit_is_set());
        if was_enabled {
            self.regs.en().write(|w| w.en().set_bit());
        }
    }

    /// Zero the count without touching the running state or the configuration.
    pub fn reset(&mut self) {
        self.write_cmd(CNTRST);
    }

    /// Resume counting with the configuration already in place.
    pub fn start(&mut self) {
        self.write_cmd(STARTCNT);
    }

    /// Stop counting. The count and the configuration stay.
    pub fn stop(&mut self) {
        self.write_cmd(STOPCNT);
    }

    /// The current count.
    ///
    /// A plain read: this part has no capture command (`CMD` has no such bit and `CNT` has no writer), and the example
    /// measured a running read against the read after `CMD.STOPCNT` and got the same value.
    pub fn count(&self) -> u16 {
        self.regs.cnt().read().cnt().bits()
    }

    /// Whether the counter is running right now (`STATUS.CNTRUNNING`).
    pub fn running(&self) -> bool {
        self.regs.status().read().cntrunning().bit_is_set()
    }

    /// Whether the counter has wrapped past `TOP` since the last clear (`IF.OF`).
    pub fn overflowed(&self) -> bool {
        self.regs.if_().read().of().bit_is_set()
    }

    /// The raw `IF` word, for a record that has to show it rather than summarise it.
    pub fn flags(&self) -> u32 {
        self.regs.if_().read().bits()
    }

    /// `(CFG, CTRL, STATUS)`, so a probe can report the words it configured instead of the ones it meant to configure.
    pub fn words(&self) -> (u32, u32, u32) {
        (
            self.regs.cfg().read().bits(),
            self.regs.ctrl().read().bits(),
            self.regs.status().read().bits(),
        )
    }
}
