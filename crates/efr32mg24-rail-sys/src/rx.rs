//! Receiving: start the radio, and take one packet out of the circular FIFO.
//!
//! ## Where the difficulty actually is
//!
//! Not in the calls -- `RAIL_StartRx`, `RAIL_GetRxPacketInfo`, `RAIL_ReleaseRxPacket` are three lines.
//! The difficulty is that RAIL's receive buffer is **circular**: a packet may start near the end and
//! wrap to the beginning, so it arrives as two portions with two pointers and two lengths. Copying
//! that wrong reads memory that is not the packet, silently.
//!
//! So the arithmetic of "how much goes in each portion, and does it fit in my buffer" is pure and
//! lives in [`portions`], with host tests. The unsafe copy does nothing but memcpy what that function
//! decided.
//!
//! ## What still blocks a real run
//!
//! `RAIL_StartRx` needs a channel number configured through `RAIL_ConfigChannels`, and that table is
//! not in this repository (the same gap that stops transmission: see `examples/bring_up.rs`). This
//! module is the receiving half of the plumbing, ready for the day the table exists.

use core::ptr;

use crate::{
    RAIL_Handle_t, RAIL_RxPacketHandle_t, RAIL_RxPacketInfo_t, RAIL_RxPacketStatus_t, RAIL_Status_t,
};

/// A received packet, as far as this crate is concerned: bytes and where they came from.
#[derive(Debug, PartialEq, Eq)]
pub struct Packet<'a> {
    /// The packet's bytes, copied out of RAIL's circular buffer.
    pub payload: &'a [u8],
    /// RAIL's own verdict on the packet (CRC, sync word, ...); `RAIL_RX_PACKET_READY_SUCCESS` for a good frame.
    pub status: RAIL_RxPacketStatus_t,
}

/// How many bytes come from the first portion and how many from the last.
///
/// `total` is what the packet says it is, `first` is how much of it sits contiguously before the
/// wrap. The packet must fit in `out_len` in full: a partial copy of a frame is worse than a refused
/// one, because the layers above would checksum a truncated frame and blame the air.
pub fn portions(total: usize, first: usize, out_len: usize) -> Option<(usize, usize)> {
    if total == 0 || total > out_len || first > total {
        return None;
    }
    Some((first, total - first))
}

/// Starts receiving on `channel`.
///
/// `scheduler` is null: this crate does not own a RAIL scheduler, so the caller must run
/// `RAIL_RuntimeAssert`/`RAIL_HandleEvents` from its own loop or interrupt. Passing a scheduler this
/// crate does not manage would be a lie about who drains the radio's events.
/// # Safety
///
/// `handle` must be a live RAIL handle with the radio configured, and must not be used by another
/// caller at the same time.
pub unsafe fn start(handle: RAIL_Handle_t, channel: u16) -> RAIL_Status_t {
    // SAFETY: `handle` comes from RAIL_Init; a null scheduler is explicitly supported by RAIL and
    // means "the application will poll".
    unsafe { crate::RAIL_StartRx(handle, channel, ptr::null()) }
}

/// Copies the payload of `info` into `out`, and releases the packet back to RAIL.
///
/// Returns `None` when the frame does not fit, in which case the packet is released anyway -- holding
/// it would stall the receive FIFO for every frame behind it.
/// # Safety
///
/// `handle` must be live; `info` must have come from `RAIL_GetRxPacketInfo` for that handle and not
/// yet be released; `packet` must be the handle that call returned.
pub unsafe fn take(
    handle: RAIL_Handle_t,
    packet: RAIL_RxPacketHandle_t,
    info: &RAIL_RxPacketInfo_t,
    out: &mut [u8],
) -> Option<usize> {
    let total = info.packetBytes as usize;
    let first = info.firstPortionBytes as usize;
    let (n_first, n_last) = match portions(total, first, out.len()) {
        Some(p) => p,
        None => {
            release(handle, packet);
            return None;
        }
    };

    // SAFETY: the two portions are RAIL-owned buffers valid until the packet is released, which
    // happens below; `portions` has already established that both copies stay inside `out`, and the
    // last portion is null exactly when its length is zero.
    unsafe {
        if n_first > 0 {
            ptr::copy_nonoverlapping(info.firstPortionData, out.as_mut_ptr(), n_first);
        }
        if n_last > 0 {
            ptr::copy_nonoverlapping(info.lastPortionData, out.as_mut_ptr().add(n_first), n_last);
        }
    }

    release(handle, packet);
    Some(total)
}

/// Gives the packet back to RAIL. Forgetting this stalls the FIFO, which looks like a radio that
/// stopped receiving.
/// # Safety
///
/// `handle` must be live and `packet` must be a held packet for it, released once.
pub unsafe fn release(handle: RAIL_Handle_t, packet: RAIL_RxPacketHandle_t) {
    // The struct carries no packet handle: RAIL identifies a packet by the handle it was fetched
    // with, and the newest one is the one this crate takes. Releasing by that constant is the
    // documented way to give back what `RAIL_GetRxPacketInfo(handle, RAIL_RX_PACKET_HANDLE_NEWEST, ..)`
    // returned, and forgetting it stalls the FIFO -- which looks like a radio that stopped receiving.
    // SAFETY: the handle comes from RAIL_Init; releasing a packet that is not held is a no-op in RAIL.
    unsafe {
        crate::RAIL_ReleaseRxPacket(handle, packet);
    }
}

#[cfg(test)]
mod tests {
    use super::portions;

    #[test]
    fn a_packet_that_does_not_wrap_is_all_first() {
        assert_eq!(portions(20, 20, 64), Some((20, 0)));
    }

    #[test]
    fn a_wrapped_packet_splits_in_two() {
        assert_eq!(portions(30, 18, 64), Some((18, 12)));
    }

    #[test]
    fn a_packet_too_big_for_the_buffer_is_refused() {
        assert_eq!(
            portions(100, 60, 64),
            None,
            "nao se entrega um frame truncado"
        );
    }

    #[test]
    fn an_empty_packet_is_refused() {
        assert_eq!(portions(0, 0, 64), None);
    }

    #[test]
    fn a_first_portion_larger_than_the_packet_is_refused() {
        assert_eq!(
            portions(10, 11, 64),
            None,
            "um first maior que o total e um descritor corrompido"
        );
    }

    #[test]
    fn exactly_the_buffer_size_fits() {
        assert_eq!(portions(64, 64, 64), Some((64, 0)));
    }
}
