//! GPIO external interrupts, delivered to the core instead of to the PRS.
//!
//! [`crate::pin::EdgeWatch`] configures the edge detector and its sticky flag. Until now the only consumer this crate
//! could offer was the PRS, which takes the pad's signal without the CPU (measured 2026-11-26). This module adds the
//! other consumer: the NVIC, with a counter the handler keeps, so a driver can wait for an edge instead of polling.
//!
//! ## The two vectors, and who installs them
//!
//! The part has one vector per channel parity: `GPIO_ODD_IRQHandler` for channels 1, 3, 5 ... and
//! `GPIO_EVEN_IRQHandler` for 0, 2, 4 ... This HAL does **not** define them. A library that takes two of the part's
//! vectors from every application is a library the application cannot configure, and the same crate already keeps the
//! SYSRTC handler in the application for the same reason. Both handlers are one call:
//!
//! ```ignore
//! #[no_mangle]
//! pub extern "C" fn GPIO_ODD_IRQHandler() {
//!     efr32mg2x_hal::extint::service(Parity::Odd);
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn GPIO_EVEN_IRQHandler() {
//!     efr32mg2x_hal::extint::service(Parity::Even);
//! }
//! ```
//!
//! A handler that does not call [`service`] leaves the channel's flag set and the vector re-enters for ever: the line
//! is asserted while any flag of the parity is set. [`service`] acknowledges every flag of its parity and counts only
//! the channels an [`EdgeCounter`] has armed, so a channel nobody is counting costs one read and one write, not a
//! storm.
//!
//! ## Why the flag is cleared before the counters move
//!
//! `IF` is a level that survives the handler, so the acknowledgement comes first and the count after it. An edge that
//! arrives inside that window is lost from the count but is not lost to the vector — it sets the flag again and the
//! handler runs a second time. The other order would count the same flag twice.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::pac;
use crate::pin::EdgeWatch;

/// Offset of the GPIO flag word; the vendor header and the PAC agree on `0x420`.
const IF_OFFSET: usize = 0x420;

/// Offset of the write-one-to-clear alias of a register on this part. LETIMER0 measured the same `+0x2000` window,
/// and so did this block's `IF` (2026-11-26).
const CLR_ALIAS: usize = 0x2000;

/// Channels whose flag the handler counts, bit `n` = channel `n`.
static ARMED: AtomicU32 = AtomicU32::new(0);

/// One counter per external-interrupt channel.
static COUNTS: [AtomicU32; 16] = [const { AtomicU32::new(0) }; 16];

/// Entries the two vectors have made, index 0 = even, 1 = odd.
///
/// Separate from [`COUNTS`] on purpose: a counter proves an edge was counted, this proves **which vector** counted it,
/// and the parity split is the one claim a per-channel counter cannot make on its own.
static ENTRIES: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];

/// Which of the two GPIO vectors a channel uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    /// Channels 0, 2, 4 ...; the `GPIO_EVEN_IRQHandler` vector.
    Even,
    /// Channels 1, 3, 5 ...; the `GPIO_ODD_IRQHandler` vector.
    Odd,
}

impl Parity {
    /// The channels this parity owns, as a mask over `IF`.
    #[must_use]
    pub const fn mask(self) -> u32 {
        match self {
            // 0x5555 and 0xAAAA are the even and odd bit patterns of a 16-bit word.
            Parity::Even => 0x5555,
            Parity::Odd => 0xAAAA,
        }
    }

    /// The vector this parity is delivered on, as the PAC names it.
    #[must_use]
    pub const fn of_channel(channel: u8) -> Self {
        if channel.is_multiple_of(2) {
            Parity::Even
        } else {
            Parity::Odd
        }
    }

    /// The PAC interrupt this parity is delivered on, for `NVIC` calls.
    #[must_use]
    pub fn interrupt(self) -> pac::Interrupt {
        match self {
            Parity::Even => pac::Interrupt::GPIO_EVEN,
            Parity::Odd => pac::Interrupt::GPIO_ODD,
        }
    }
}

/// Serve one vector: acknowledge the pending channels of `parity` and count the armed ones.
///
/// Returns the mask of flags acknowledged, so a handler can be told what it served. Must be called from
/// `GPIO_EVEN_IRQHandler` / `GPIO_ODD_IRQHandler` with the matching [`Parity`].
///
/// ## Every flag of the parity is acknowledged, not only the armed ones
///
/// The vector's line is asserted while any flag of the parity is set, so a handler that leaves one behind is
/// re-entered immediately, for ever — the same trap `SYSRTC_APP` has with its overflow bit. Channels that are not
/// armed are therefore acknowledged and not counted. [`EdgeCounter::arm`] clears the flag again before it starts, so
/// an acknowledgement here cannot swallow an edge a later window is waiting for.
pub fn service(parity: Parity) -> u32 {
    let gpio = crate::board::gpio_static();
    let flags = gpio.if_().read().bits() & parity.mask();
    if flags == 0 {
        return 0;
    }

    // Acknowledge first: see the module note on ordering.
    let address = gpio as *const _ as usize + IF_OFFSET + CLR_ALIAS;
    // SAFETY: the alias window maps the same `IF` word with write-one-to-clear semantics; `flags` holds only bits of
    // this parity's channels, which are the ones this call is serving.
    unsafe { core::ptr::write_volatile(address as *mut u32, flags) };

    let mut bits = flags & ARMED.load(Ordering::Relaxed);
    while bits != 0 {
        let channel = bits.trailing_zeros() as usize;
        COUNTS[channel].fetch_add(1, Ordering::Relaxed);
        bits &= bits - 1;
    }
    ENTRIES[match parity {
        Parity::Even => 0,
        Parity::Odd => 1,
    }]
    .fetch_add(1, Ordering::Relaxed);

    flags
}

/// How many times each vector has entered and found work: `(even, odd)`.
///
/// A vector that enters with nothing to do returns without touching these, so the two numbers say how many edges
/// were actually delivered rather than how often the line was pended.
#[must_use]
pub fn entries() -> (u32, u32) {
    (
        ENTRIES[0].load(Ordering::Relaxed),
        ENTRIES[1].load(Ordering::Relaxed),
    )
}

/// One external-interrupt channel, counted in the handler.
///
/// Built by [`crate::board::Board::count_edges`]. The caller installs the two handlers, calls [`Self::arm`], and reads
/// [`Self::count`] whenever it wants; nothing here polls the flag.
pub struct EdgeCounter<'a> {
    watch: EdgeWatch<'a>,
}

impl<'a> EdgeCounter<'a> {
    pub(crate) fn new(watch: EdgeWatch<'a>) -> Self {
        let counter = Self { watch };
        counter.reset();
        counter
    }

    /// The channel, which is also the pin index on this part.
    #[must_use]
    pub fn channel(&self) -> u8 {
        self.watch.channel()
    }

    /// The vector this channel is delivered on.
    #[must_use]
    pub fn parity(&self) -> Parity {
        Parity::of_channel(self.channel())
    }

    /// Stop the core from entering the channel's vector, leaving the detector armed.
    ///
    /// The flag keeps latching; it is simply never served until [`Self::unmask_vector`]. This is how a measurement
    /// separates the hardware's flag from the handler's count.
    pub fn mask_vector(&self) {
        cortex_m::peripheral::NVIC::mask(self.parity().interrupt());
    }

    /// Let the core enter the channel's vector again.
    ///
    /// A vector left pending while masked is dropped first: it would otherwise be delivered as this window's first
    /// edge, and the flag it came from has already been cleared by [`Self::reset`] or by a manual [`service`].
    pub fn unmask_vector(&self) {
        let interrupt = self.parity().interrupt();
        // SAFETY: the caller has installed the handler for this parity (see the module documentation). The vector
        // is unmasked only for a channel whose flag was just cleared, so the first entry is a real edge.
        unsafe {
            cortex_m::peripheral::NVIC::unpend(interrupt);
            cortex_m::peripheral::NVIC::unmask(interrupt);
        }
    }

    /// The port and pin select words this channel sits in, as they read back.
    #[must_use]
    pub fn selection(&self) -> (u32, u32) {
        self.watch.selection()
    }

    /// The whole `IF` word, for a probe that reports what the hardware latched.
    #[must_use]
    pub fn flags(&self) -> u32 {
        self.watch.flags()
    }

    /// Arm or disarm the edge masks without moving the channel's binding.
    ///
    /// This is the control for a measurement: with both masks off the detector produces no flag, so a handler that
    /// counts nothing is doing the right thing.
    pub fn set_edges(&self, rising: bool, falling: bool) {
        self.watch.set_edges(rising, falling);
    }

    /// Zero the counter and drop any flag the channel had latched.
    pub fn reset(&self) {
        COUNTS[self.channel() as usize].store(0, Ordering::Relaxed);
        self.watch.clear();
    }

    /// How many edges the handler has counted since [`Self::reset`] or [`Self::arm`].
    #[must_use]
    pub fn count(&self) -> u32 {
        COUNTS[self.channel() as usize].load(Ordering::Relaxed)
    }

    /// Start counting: zero the counter, clear the flag, enable the pin's interrupt and unmask its vector.
    ///
    /// The order matters. A flag left over from an earlier configuration, or a vector left pending by one, is an
    /// entry the handler would attribute to this window's first edge; both are dropped before the channel is armed.
    ///
    /// `IEN` is the per-pin gate in front of the vector — the edge masks alone do not deliver anything to the core —
    /// so it is written here and cleared in [`Self::disarm`].
    pub fn arm(&self) {
        self.reset();
        let channel = self.channel();
        let bit = 1u32 << channel;
        ARMED.fetch_or(bit, Ordering::Relaxed);
        // SAFETY: one bit of the enable word, for the channel this counter owns.
        crate::board::gpio_static()
            .ien()
            .modify(|r, w| unsafe { w.bits(r.bits() | bit) });
        let interrupt = Parity::of_channel(channel).interrupt();
        // SAFETY: the caller has installed the handler for this parity (see the module documentation); unmasking a
        // vector whose handler is the weak default would send the first edge into a loop that never returns.
        unsafe {
            cortex_m::peripheral::NVIC::unpend(interrupt);
            cortex_m::peripheral::NVIC::unmask(interrupt);
        }
    }

    /// Stop counting, and stop the pin from requesting an interrupt at all.
    ///
    /// The binding and the edge masks stay as they are, so the flag can still be inspected; only the delivery stops.
    /// The vector stays unmasked: it is inert while no channel of its parity is enabled, and masking it would silence
    /// any other channel that shares it.
    pub fn disarm(&self) {
        let bit = 1u32 << self.channel();
        ARMED.fetch_and(!bit, Ordering::Relaxed);
        // SAFETY: one bit of the enable word, for the channel this counter owns.
        crate::board::gpio_static()
            .ien()
            .modify(|r, w| unsafe { w.bits(r.bits() & !bit) });
    }

    /// True while this channel's flag increments a counter in the handler.
    #[must_use]
    pub fn is_armed(&self) -> bool {
        ARMED.load(Ordering::Relaxed) & (1u32 << self.channel()) != 0
    }
}
