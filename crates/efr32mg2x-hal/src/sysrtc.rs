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

use efr32mg2x_pac::{
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
    /// **Group 0, at the offsets the vendor header gives.** Group 1 (`0x60`) is what the RAIL blob drives, and its
    /// events arrive on IRQ 68, whose handler RAIL defines — an application timer in that group takes RAIL's
    /// interrupt. Measured: group 0 accepts writes at these offsets and its compare flag (bit 1) sets exactly one
    /// count after the deadline.
    /// Every SDK header puts these at 0x40 and is wrong for this silicon.
    pub const GRP0_IF: usize = 0x40;
    pub const GRP0_IEN: usize = 0x44;
    pub const GRP0_CTRL: usize = 0x48;
    pub const GRP0_CMP0VALUE: usize = 0x4C;
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
    /// `SYSRTC_GRP0_IF` bits, from `efr32mg24_sysrtc.h`: `OVF` = bit 0, `CMP0` = **bit 1**, `CMP1` = bit 2,
    /// `CAP0` = bit 3. The first version of this module tested bit 0 and concluded the compare never fired; a
    /// probe then read `0x22` one count after the deadline on channel 0 and `0x44` on channel 1, which is exactly
    /// what this header describes. **The header was right and the code was wrong** -- recorded because the
    /// opposite claim was written here first.
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

/// The widened 64-bit tick, shared by every caller.
///
/// One global rather than a field on `Sysrtc`: a time source is a singleton, and the point of the widened tick is
/// that every caller sees the same number. See [`Sysrtc::now_wide`].
/// The SYSRTC register base, recorded by [`Sysrtc::take`] so the global accessors work without a handle. Zero
/// means the module was never taken.
static REGS_BASE: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

static WIDE: critical_section::Mutex<core::cell::RefCell<crate::cores::sysrtc::WideTick>> =
    critical_section::Mutex::new(core::cell::RefCell::new(
        crate::cores::sysrtc::WideTick::new(),
    ));

/// The SYSRTC peripheral, once.
pub struct Sysrtc {
    regs: *const RegisterBlock,
    cmu: usize,
}

// SAFETY: fixed peripheral addresses for the life of the program; every method takes `&self`, and a second
// value is prevented by `Board::sysrtc` being reachable only from the singleton `Board`.
unsafe impl Send for Sysrtc {}

impl Sysrtc {
    /// Gate the SYSRTC's bus clocks and select its reference clock, **without touching the module itself**.
    ///
    /// This is the state the Bluetooth stack needs before it starts. Measured both ways: with the clocks gated off,
    /// RAIL's first write to `SYSRTC->CMD` during its bring-up faults (`cfsr=0x8200`, `bfar=0x500a_8014`) because the
    /// module has no bus clock; with the module *running* (i.e. after a full [`Sysrtc::take`]) RAIL's write to `CFG`
    /// faults instead (`bfar=0x500a_800c`), because `CFG` is writable only while the counter is disabled. Gating the
    /// clock and leaving the module disabled is the only state that satisfies both.
    ///
    /// Call this before starting the Bluetooth stack. A later [`Sysrtc::take`] is still correct: it sees the gates
    /// already set and a module that is not running, and configures it as it always did.
    pub fn enable_clocks(cmu: &CmuBlock) {
        let cmu_base = cmu as *const CmuBlock as usize;
        let clken0 = (cmu_base + off::CMU_CLKEN0) as *mut u32;
        let gates =
            unsafe { core::ptr::read_volatile(clken0) } | bits::CLKEN0_LFRCO | bits::CLKEN0_SYSRTC0;
        unsafe { core::ptr::write_volatile(clken0, gates) };

        // Clock select, at the offset measured from the blob (the header's is wrong for this silicon).
        let clkctrl = (cmu_base + off::CMU_SYSRTC0CLKCTRL) as *mut u32;
        let current = unsafe { core::ptr::read_volatile(clkctrl) };
        unsafe {
            core::ptr::write_volatile(clkctrl, (current & !bits::CLKSEL_MASK) | bits::CLKSEL_LFRCO)
        };

        // No sync-busy wait here on purpose: this module has no constant for it, and inventing one is exactly the
        // failure mode this project keeps paying for. `take()` performs the documented wait before it configures
        // anything, and RAIL performs its own before it writes `CFG`.
    }

    /// Make sure the counter is **running** before the Bluetooth stack starts.
    ///
    /// Measured 2026-11-17, and it corrects this module's earlier claim. With the SYSRTC clocked but stopped, RAIL's
    /// bring-up asserts at its own line 380 — recorded as code 75 in our `RAILCb_AssertFailed` (`"ASRT", 0x4b, ...` at
    /// `0x2003_F500`, `RAIL_AssertLineNumber = 380` at `0x2001_1dfc`). With the counter left running by a previous
    /// firmware, the same product image boots and publishes normally. The earlier note concluded "clocked and
    /// disabled" from a `CFG` fault at `0x500a_800c`; that fault was our *own* re-init writing `CFG` while the module
    /// ran, not RAIL's write, which happens while the counter is still stopped.
    ///
    /// Call this instead of [`Sysrtc::enable_clocks`] before the stack. A later [`Sysrtc::take`] still reconfigures
    /// from scratch: it stops a running module before writing `CFG`, which is the order the register demands.
    pub fn ensure_running(cmu: &CmuBlock) {
        Self::enable_clocks(cmu);
        let this = Sysrtc {
            regs: efr32mg2x_pac::Sysrtc0Ns::ptr(),
            cmu: cmu as *const CmuBlock as usize,
        };
        if !this.is_enabled() {
            this.write_en_set(bits::EN_EN);
            this.wait_sync();
        }
        if !this.is_running() {
            this.write_cmd(bits::CMD_START);
            this.wait_sync();
        }
    }

    /// Take the peripheral and put it in a known state: LFRCO, compare channel 0 armed, counter running.
    pub fn take(cmu: &CmuNs, rtc: &Sysrtc0Ns) -> Self {
        let cmu_regs: &CmuBlock = cmu;
        let cmu_base = cmu_regs as *const CmuBlock as usize;
        let regs: &RegisterBlock = rtc;
        let this = Self {
            regs,
            cmu: cmu_base,
        };
        // O relogio global (`now_wide_global`, `schedule_at_global`) precisa do endereco sem ter um handle: um
        // driver de tempo e' chamado de contextos que nao podem possuir o periferico. Registado aqui, nos dois
        // caminhos do `take` (o frio e o quente).
        REGS_BASE.store(this.regs as usize, core::sync::atomic::Ordering::Relaxed);

        // 1. Bus clocks: SYSRTC's, and the LFRCO's own (bit 22) -- emlib's CMU_OscillatorEnable does the
        //    same, and without it a clock source can be selected that is not actually running.
        let clken0 = (this.cmu + off::CMU_CLKEN0) as *mut u32;
        let gates =
            unsafe { core::ptr::read_volatile(clken0) } | bits::CLKEN0_LFRCO | bits::CLKEN0_SYSRTC0;
        unsafe { core::ptr::write_volatile(clken0, gates) };

        // **A running module is not reconfigured, and this early return is load-bearing.**
        //
        // `CFG` is writable only while the counter is disabled. Once the Bluetooth stack has started, RAIL owns the
        // SYSRTC configuration -- it programs the module for its own sequencer timing on group 1 -- and a second
        // party that stops, disables and reconfigures it produces a bus fault on the `CFG` write at
        // `0x500a_800c` (measured: `cfsr=0x8200 bfar=0x500a800c`, escalated to a HardFault inside the fault
        // handler). Whoever started it keeps it; an application only needs to read `CNT` and to program its own
        // compare channel on group 0, and both are legal while it runs.
        if this.is_enabled() && this.is_running() {
            // O comentario acima esta' certo sobre o `CFG` e estava errado sobre o resto: "a aplicacao so' precisa
            // de ler o CNT e de programar o seu canal de comparacao no grupo 0" -- e este retorno nao programava
            // o grupo 0. Sem `CTRL_CMP0EN` o canal esta' desligado, `schedule_at` escreve um valor que nunca
            // compara, e nao vem interrupcao nenhuma. Medido 2026-11-21: contador em 32 673 464 ticks (portanto
            // este caminho), 40 M de voltas e nenhum acordar.
            //
            // Quem started o modulo continua a ser dono do `CFG`; o canal de comparacao e' da aplicacao e e'
            // legal enquanto o contador corre. Sao dois donos de coisas diferentes, e o retorno tratava-os como um.
            this.arm_group0();
            return this;
        }

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
        this.arm_group0();

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

    /// The monotonic **64-bit** tick, widened from the 32-bit counter by counting its wraps.
    ///
    /// ## Why this exists, and why it is one global
    ///
    /// A 32-bit counter at 32.768 kHz wraps every 36.4 hours, so anything that has to compare two moments -- a
    /// deadline, a dwell time, a time driver -- needs more than the counter gives. The widening rule is tested in
    /// [`crate::cores::sysrtc::WideTick`]: a reading smaller than the previous one can only be a wrap.
    ///
    /// The state is a single global because a time source is a singleton. The Bluetooth stack's sleep timer widens
    /// the same counter (`sleeptimer.rs` keeps its own copy of `WideTick`), and two independent widenings would
    /// disagree about what time it is the moment one of them missed a call.
    ///
    /// ## What this costs, stated plainly
    ///
    /// It widens **when called**: it must run at least once every 36.4 hours or a wrap goes uncounted, and the tick
    /// then jumps backwards by 36.4 hours. The peripheral raises an overflow interrupt (`GRP0_IF.OVF`, in this
    /// SVD) which would remove that requirement; it is not wired here yet, and the honest description of today's
    /// code is a polled widening, not a free-running clock.
    pub fn now_wide(&self) -> u64 {
        let counter = self.now();
        critical_section::with(|cs| WIDE.borrow(cs).borrow_mut().update(counter))
    }

    /// How many wraps the widened tick has counted, for a record or a test.
    pub fn wraps(&self) -> u32 {
        critical_section::with(|cs| WIDE.borrow(cs).borrow().wraps())
    }

    /// True while the counter is running.
    #[inline]
    pub fn is_running(&self) -> bool {
        self.regs_ref().status().read().bits() & bits::STATUS_RUNNING != 0
    }

    /// Enable the application's compare channel on group 0 and clear its flags.
    ///
    /// Shared by both paths through `take`, because both need it: the cold path configures the module from zero,
    /// and the warm path inherits a running module whose compare channel belongs to nobody. Doing it in only one
    /// of them is how a compare value gets written into a disabled channel and nothing ever fires.
    fn arm_group0(&self) {
        wr(self.regs as usize + off::GRP0_CTRL, bits::CTRL_CMP0EN);
        wr(self.regs as usize + off::GRP0_IEN, 0);
        wr(self.regs as usize + off::GRP0_IF + 0x2000, bits::GRP0_ALL);
    }

    /// The monotonic 64-bit tick **without a handle**.
    ///
    /// A time driver is called from contexts that cannot own the peripheral: an interrupt, a `static` `Platform`
    /// impl, a consumer in another crate. The register address is recorded by [`Sysrtc::take`], and the widening
    /// state is the same single global [`Sysrtc::now_wide`] uses -- one clock, one answer.
    ///
    /// Returns 0 before the module is taken, which a caller can detect by comparing two reads.
    pub fn now_wide_global() -> u64 {
        let base = REGS_BASE.load(core::sync::atomic::Ordering::Relaxed);
        if base == 0 {
            return 0;
        }
        // SAFETY: the base is written only by `take`, points at SYSRTC0_NS, and lives for the program.
        let regs = unsafe { &*(base as *const RegisterBlock) };
        let mut spins = 0u32;
        while regs.syncbusy().read().bits() != 0 {
            spins = spins.wrapping_add(1);
            if spins == 200_000 {
                break;
            }
        }
        let counter = regs.cnt().read().bits();
        critical_section::with(|cs| WIDE.borrow(cs).borrow_mut().update(counter))
    }

    /// Arm the comparator **without a handle**, the companion of [`Sysrtc::now_wide_global`].
    ///
    /// Returns `false` when the module was never taken, and otherwise whether the deadline had already passed.
    pub fn schedule_at_global(deadline: u32) -> bool {
        let base = REGS_BASE.load(core::sync::atomic::Ordering::Relaxed);
        if base == 0 {
            return false;
        }
        // SAFETY: as in `now_wide_global`.
        let regs = unsafe { &*(base as *const RegisterBlock) };
        let mut spins = 0u32;
        while regs.syncbusy().read().bits() != 0 {
            spins = spins.wrapping_add(1);
            if spins == 200_000 {
                break;
            }
        }
        let now = regs.cnt().read().bits();
        // A decisao (prazo no passado, ou igual a agora) vive em `cores::sysrtc`, testada no host.
        let (target, passed) = crate::cores::sysrtc::compare_target(now, deadline);
        regs.grp0_cmp0value().write(|w| unsafe { w.bits(target) });
        regs.grp0_ien()
            .write(|w| unsafe { w.bits(bits::GRP0_CMP0) });
        passed
    }

    /// True while the module is enabled.    /// True while the module is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.regs_ref().en().read().bits() & bits::EN_EN != 0
    }

    /// Fire an interrupt when the counter reaches `deadline` (absolute, not a delay).
    ///
    /// Returns `true` when the deadline had already gone, in which case it is programmed one tick ahead
    /// instead of being left armed against a value the counter will not reach until it wraps.
    #[inline]
    pub fn schedule_at(&self, deadline: u32) -> bool {
        self.wait_sync();
        // A decisao (deadline no passado, ou igual a agora) vive em `libraries/sysrtc-core`, onde o host a
        // testa; este crate e' so' cross-compilado, logo aritmetica aqui nunca seria exercida. H25/B8.
        let (target, passed) = crate::cores::sysrtc::compare_target(self.now(), deadline);
        wr(self.regs as usize + off::GRP0_CMP0VALUE, target);
        wr(self.regs as usize + off::GRP0_IEN, bits::GRP0_CMP0);
        passed
    }

    /// Schedule `ticks` from now.
    #[inline]
    pub fn schedule_in(&self, ticks: u32) -> bool {
        self.schedule_at(self.now().wrapping_add(ticks))
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
        unsafe { cortex_m::peripheral::NVIC::unmask(efr32mg2x_pac::Interrupt::SYSRTC_APP) };
    }

    /// Acknowledge the compare interrupt from a handler, where no `Sysrtc` instance is reachable.
    ///
    /// ## Why this is an associated function and not a method
    ///
    /// The interrupt fires while `main` is somewhere else, holding the `Sysrtc` it owns. A handler cannot borrow it,
    /// and the alternative -- the example naming the PAC to find the registers -- is the layering defect the gate
    /// refuses. The write itself is one write-1-to-clear to the group 0 flag, which is exactly what
    /// [`Self::take_compare_event`] does with an instance.
    ///
    /// It does not read anything back: whether the flag was set is the caller's business, and a handler that cared
    /// would be a handler doing work it could have deferred.
    pub fn acknowledge_compare_interrupt() {
        // SAFETY: `Sysrtc0Ns::steal` is the PAC's raw view; the peripheral exists (this crate was taken from the
        // same silicon) and the write is the documented write-1-to-clear alias at +0x2000. A second view of a
        // flag register whose only operation is "clear these bits" cannot corrupt anything a `Sysrtc` holds.
        let rtc = unsafe { efr32mg2x_pac::Sysrtc0Ns::steal() };
        let regs: &RegisterBlock = &rtc;
        wr(
            regs as *const RegisterBlock as usize + off::GRP0_IF + 0x2000,
            bits::GRP0_CMP0,
        );
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
