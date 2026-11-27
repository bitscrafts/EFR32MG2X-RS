//! Chained (linked) LDMA transfers -- [C2-DMA.3].
//!
//! A descriptor is four words: `CTRL`, `SRC`, `DST`, `LINK`. `LINK` names the address of the next descriptor's
//! `CTRL` word, and with the `LINK` bit set (bit 1) the controller loads that descriptor by itself when the current
//! one's count runs out. A chain is therefore one channel doing two or more legs in order, with no software between
//! them.
//!
//! # What the part actually required (measured 2026-11-27)
//!
//! The first implementation put the second descriptor in **channel 1's register set** -- four contiguous words at
//! `CH1_CTRL`, `CH1_SRC`, `CH1_DST`, `CH1_LINK`, which is exactly the layout a descriptor has -- and pointed the
//! head's link at `CH1_CTRL`. The result, on two boots, was `d1=8 d2=0` with the channel's source register already
//! inside the second leg: the controller fetched the second descriptor and then sat there. Six configurations in one
//! boot separated the possibilities:
//!
//! ```text
//! probe: c1=8/0/1 c2=8/0/1 c3=8/0/1 c4=8/0/1 c5=8/0/1 c6=8/8/1 c7=8/8/1 c8=0/0/0
//!        (leg 1 / leg 2 words moved / source register past the start of leg 2)
//! ```
//!
//! - `c1` register-storage descriptor, nothing extra; `c2` with `STRUCTREQ` on both; `c3` with `STRUCTREQ` and
//!   `REQMODE`; `c4` `REQMODE` only; `c5` `STRUCTREQ` on the loaded descriptor only. All five: fetched, never run.
//! - `c6` the same as `c1`, with one **software** `SWREQ` issued after the fetch: both legs moved. So the fetched
//!   descriptor is valid and runnable, and the only thing missing was a trigger.
//! - `c7` the same four words placed in **RAM**, with `STRUCTREQ` set: both legs moved. `STRUCTREQ` is the trigger.
//! - `c8` the same RAM descriptor without `STRUCTREQ`: nothing moved, which is the control that says `c7`'s success
//!   came from that bit and not from the descriptor simply being in RAM.
//!
//! The bit cannot be stored in a channel's `CTRL`: writing 1 to `CH1_CTRL` bit 3 reads back 0 (the PAC exposes
//! `STRUCTREQ` as a reader only, and the register does not keep it). A descriptor in RAM can. So the second
//! descriptor of a chain lives in RAM, and this module builds it from the encoding the typed writer produced for the
//! head -- only the count differs between the legs -- which keeps every other field out of hand encoding.
//!
//! # Completion
//!
//! `DONE` is one bit per channel (`IF`, RM 25.7.11): the first leg sets it and the second sets the same bit, so it
//! says *a* descriptor finished. This module waits for `DONE`, lets a leg that is still in flight settle, and then
//! compares every destination against its source. A leg that never ran leaves a destination that does not match,
//! which is reported as `Timeout` -- the check fails closed, never open.

use super::{Channel, DmaError, TransferElement};
use crate::pac;

/// One leg of a chained transfer: the `src` elements copied into `dst`, and nothing else.
///
/// The pair is a type rather than two arguments so that a chain reads as a list, and so that the borrow of `dst`
/// lasts for as long as the transfer does. The controller writes into that buffer while the call is running; a
/// caller cannot touch it until the call returns, and the compiler is what enforces it.
pub struct Segment<'a, T> {
    src: &'a [T],
    dst: &'a mut [T],
}

impl<'a, T> Segment<'a, T> {
    /// A leg that copies `src` into `dst`.
    ///
    /// The two must be the same length; a mismatch, an empty leg or a leg longer than 2047 units is refused when
    /// the chain is armed, rather than allowed to copy a prefix and report success.
    pub fn new(src: &'a [T], dst: &'a mut [T]) -> Self {
        Self { src, dst }
    }
}

/// Whether a chain is armed with the head descriptor's `LINK` bit set.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChainMode {
    /// `LINK` set: when the head descriptor's count runs out the controller loads the descriptor the link names and
    /// runs it, because that descriptor carries `STRUCTREQ`.
    Linked,
    /// `LINK` cleared, the same data programmed into the same place. Nothing after the first leg runs, because a
    /// descriptor the controller never fetches is just four words in memory. It exists so that "the second leg moved
    /// because of the link" has the near miss that shows it does not move without it.
    FirstOnly,
}

/// The four descriptor words a channel is holding, plus the status bits that go with them.
///
/// Reading a channel back is how a caller can tell "the controller loaded the next descriptor" from "the copy
/// happened to produce the same bytes": the loaded words are the addresses of the second leg.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChannelState {
    /// `CHn_CTRL`: structure type, transfer count and unit size.
    pub ctrl: u32,
    /// `CHn_SRC`: the source address the channel is currently using.
    pub src: u32,
    /// `CHn_DST`: the destination address the channel is currently using.
    pub dst: u32,
    /// `CHn_LINK`: the link word, 0 when the current descriptor is the last one.
    pub link: u32,
    /// `IF` bit for this channel: a descriptor of this channel finished.
    pub done: bool,
    /// `CHSTATUS` bit for this channel: the channel is enabled.
    pub enabled: bool,
    /// `CHBUSY` bit for this channel: the channel is transferring right now.
    pub busy: bool,
}

/// A descriptor as the controller fetches it: four words, in this order, at a 4-byte aligned address.
///
/// The alignment is 16 rather than 4 because the fetch reads the four words as a group, and a stack slot that
/// happened to straddle a cache line is the kind of detail no one should have to reason about. The offsets are
/// checked at compile time, because the controller reads them at fixed offsets from `ctrl` and cannot be told
/// otherwise.
#[repr(C, align(16))]
struct Descriptor {
    ctrl: u32,
    src: u32,
    dst: u32,
    link: u32,
}

const _: () = {
    assert!(core::mem::offset_of!(Descriptor, src) == 4);
    assert!(core::mem::offset_of!(Descriptor, dst) == 8);
    assert!(core::mem::offset_of!(Descriptor, link) == 12);
    assert!(core::mem::size_of::<Descriptor>() == 16);
};

/// `STRUCTREQ`: "DMA transfer trigger during LINKLOAD" (`em_ldma.h`; bit 3 of `CTRL`).
///
/// The bit that makes a freshly loaded descriptor start. It is set here rather than through the PAC's typed writer
/// because the SVD gives that field no writer -- and because a channel register will not store it at all.
const STRUCTREQ: u32 = 1 << 3;

/// Field `XFERCNT` of `CTRL`: 11 bits at bit 4, holding the count minus one.
const XFERCNT_SHIFT: u32 = 4;
const XFERCNT_MASK: u32 = 0x7FF << XFERCNT_SHIFT;

/// Read back what a channel is currently holding.
///
/// `None` for a channel this HAL does not program, so a caller cannot read a register that no code path in this
/// crate writes.
pub fn channel_state(channel: u8) -> Option<ChannelState> {
    // SAFETY: LDMA's register block is at a fixed address for the life of the program and the pointer comes from the
    // PAC; every access below is a read of a status register.
    let ldma = unsafe { &(*pac::LdmaS::ptr()) };
    let ifs = ldma.if_().read().bits();
    let status = ldma.chstatus().read().chstatus().bits();
    let busy = ldma.chbusy().read().bits();
    let (ctrl, src, dst, link) = match channel {
        0 => (
            ldma.ch0_ctrl().read().bits(),
            ldma.ch0_src().read().bits(),
            ldma.ch0_dst().read().bits(),
            ldma.ch0_link().read().bits(),
        ),
        _ => return None,
    };
    Some(ChannelState {
        ctrl,
        src,
        dst,
        link,
        done: ifs & (1 << channel) != 0,
        enabled: status & (1 << channel) != 0,
        busy: busy & (1 << channel) != 0,
    })
}

impl<const N: u8> Channel<N> {
    /// Copy each segment in order, with the descriptors chained through the controller's own `LINK`.
    ///
    /// The head descriptor is programmed into this channel's registers; the second lives in a four-word descriptor
    /// on this function's stack, which the head's link word names. When the first leg's count runs out the controller
    /// loads that descriptor and runs it, with no software on the path between the legs.
    ///
    /// # Arguments
    ///
    /// * `segments` - one or two legs; each source and destination of the same length, 1 to 2047 units
    /// * `mode` - [`ChainMode::Linked`] for a chain, [`ChainMode::FirstOnly`] for the control that shows the link is
    ///   what makes the second leg run
    ///
    /// # Errors
    ///
    /// `Unsupported` for a channel other than 0 or more than two segments; `InvalidLength` for a bad leg; `Busy` if
    /// the channel will not go idle; `Timeout` if a segment did not end up copied -- which covers both "the
    /// controller never finished" and "a leg never ran", since a leg that did not run is a destination that does not
    /// match its source.
    pub fn transfer_chain<T>(
        &mut self,
        segments: &mut [Segment<'_, T>],
        mode: ChainMode,
    ) -> Result<(), DmaError>
    where
        T: TransferElement + PartialEq,
    {
        if N != 0 {
            return Err(DmaError::Unsupported);
        }
        if segments.is_empty() || segments.len() > 2 {
            return Err(DmaError::Unsupported);
        }
        for segment in segments.iter() {
            if segment.src.len() != segment.dst.len()
                || segment.src.is_empty()
                || segment.src.len() > 2047
            {
                return Err(DmaError::InvalidLength);
            }
        }

        // The second descriptor, filled inside the critical section below: its `CTRL` is the word the typed writer
        // produced for the head with only the count replaced, so no field of it is encoded by hand.
        let mut second = Descriptor {
            ctrl: 0,
            src: 0,
            dst: 0,
            link: 0,
        };

        critical_section::with(|_cs| {
            // SAFETY: the LDMA block lives at a fixed address for the life of the program, the pointer comes from the
            // PAC, and every write below is inside the critical section that owns channel 0.
            let ldma = unsafe { &(*pac::LdmaS::ptr()) };

            if ldma.chbusy().read().bits() & 1 != 0 {
                return Err(DmaError::Busy);
            }

            // Disarm, and wait for it: the control registers may only be rewritten once the channel has stopped, and
            // the disable is not instantaneous (the current burst finishes first). Bounded, so a wedged channel
            // reports `Busy` instead of hanging the caller.
            // SAFETY: CHDIS is write-1-to-disable and one bit touches only this channel.
            unsafe {
                ldma.chdis().write(|w| w.bits(1));
            }
            let mut spins = 0u32;
            while ldma.chstatus().read().chstatus().bits() & 1 != 0 {
                spins += 1;
                if spins > 100_000 {
                    return Err(DmaError::Busy);
                }
            }

            // The head's addresses and count are taken before the second leg is touched, so the two mutable
            // borrows of `segments` do not overlap.
            let head_len = segments[0].src.len();
            let head_src = segments[0].src.as_ptr() as u32;
            let head_dst = segments[0].dst.as_mut_ptr() as u32;

            // SAFETY: as above; channel 0 is disabled and idle, and the writes are the fields `transfer` programs.
            unsafe {
                ldma.ch0_ctrl().write(|w| {
                    w.structtype()
                        .transfer()
                        .xfercnt()
                        .bits(head_len as u16 - 1)
                        .size()
                        .bits(T::SIZE as u8)
                        .srcinc()
                        .one()
                        .dstinc()
                        .one()
                        .blocksize()
                        .all()
                        .doneien()
                        .set_bit()
                });

                if let Some(next) = segments.get_mut(1) {
                    let mut ctrl = ldma.ch0_ctrl().read().bits();
                    ctrl &= !XFERCNT_MASK;
                    ctrl |= ((next.src.len() as u32 - 1) << XFERCNT_SHIFT) & XFERCNT_MASK;
                    ctrl |= STRUCTREQ;
                    second.ctrl = ctrl;
                    second.src = next.src.as_ptr() as u32;
                    second.dst = next.dst.as_mut_ptr() as u32;
                    second.link = 0;
                }

                ldma.ch0_src().write(|w| w.bits(head_src));
                ldma.ch0_dst().write(|w| w.bits(head_dst));

                // Absolute addressing (LINKMODE = 0) because the address of `second` is a real address, not an
                // offset from the current descriptor; `LINK` (bit 1) is what enables the load when this descriptor's
                // count runs out.
                let link_word = match (mode, segments.len()) {
                    (ChainMode::Linked, 2) => {
                        (&second.ctrl as *const u32 as u32 & !0b11) | (1 << 1)
                    }
                    _ => 0,
                };
                ldma.ch0_link().write(|w| w.bits(link_word));

                crate::alias::clear_window(pac::LdmaS::ptr())
                    .if_()
                    .write(|w| w.bits(1));

                ldma.chen().write(|w| w.bits(1));
                ldma.swreq().write(|w| w.bits(1));
            }

            Ok(())
        })?;

        // SAFETY: as above; these are status reads.
        let ldma = unsafe { &(*pac::LdmaS::ptr()) };

        let mut cycles = 0u32;
        let timeout_cycles = 1_000_000; // ~13 ms at 78 MHz
        loop {
            if ldma.if_().read().bits() & 1 != 0 {
                break;
            }
            cycles += 1;
            if cycles > timeout_cycles {
                self.stop();
                return Err(DmaError::Timeout);
            }
        }

        // `DONE` says a descriptor finished. Two memory-to-memory legs of at most 2047 units take a few microseconds
        // at 78 MHz; 20 000 cycles is about 256 us, bounded so that a wedged controller cannot hang the caller. If a
        // leg were still in flight the comparison below finds it: it fails closed, never open.
        for _ in 0..20_000 {
            cortex_m::asm::nop();
        }

        let mut complete = true;
        for segment in segments.iter() {
            let dst: &[T] = segment.dst;
            if segment.src != dst {
                complete = false;
            }
        }

        self.stop();

        if complete {
            Ok(())
        } else {
            Err(DmaError::Timeout)
        }
    }
}
