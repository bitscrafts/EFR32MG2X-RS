//! The 802.15.4 frame rules a transmitter has to get right, in pure logic.
//!
//! ## The defect this closes, measured on the board
//!
//! The first byte of the transmit FIFO is **not** the frame control field: it is the length. 802.15.4 carries a
//! one-byte PHY header holding the payload size, and on transmit the radio writes it from the first byte it
//! finds in the FIFO. Handing the radio the MAC frame alone meant it read the frame control (`0x41`) as a
//! length of 65, asked the FIFO for sixty-five bytes that were not there, and raised
//! `RAIL_EVENT_TX_UNDERFLOW` -- bit 30, `ev=1073741824` on the wire. The API had returned `NO_ERROR` for every
//! call, the FIFO was correctly sized at 128 bytes and correctly full at 14, and the air stayed empty.
//!
//! Nothing about that failure is visible in a status code, so it is not something a comment can prevent. It
//! belongs here, where a host test can fail on it: `the_first_byte_is_the_length` is the whole lesson, and it
//! costs nothing to run.
//!
//! ## Why this is a crate and not a function in the radio binding
//!
//! The binding links ARM-only vendor blobs, so `cargo test` there fails at the linker. A test that cannot run
//! is a certificate nobody validated. The rules below are arithmetic over bytes, so they live where the gate
//! builds and tests them on the host.
//!
//! ## What this deliberately does not do
//!
//! No radio, no timing, no interrupts, no allocation. It decides four things and nothing else:
//!
//! - [`build_data_frame`] -- what bytes to put in the transmit FIFO, length byte first (the defect that cost a
//!   night is the first test in this file);
//! - [`csma`] -- whether the channel allows transmitting at all, and how long to wait if not;
//! - [`is_ack_for`] -- whether an acknowledgement belongs to the frame just sent;
//! - [`receive`] -- whether an arriving frame is addressed to this node, and what to acknowledge with.
//!
//! Backoff randomness is an input rather than a source the crate reaches for, so a test can pin it and a caller
//! can feed it what the radio's generator produced.
//!
//! It also does not claim the frame is received. A transmitter reporting its own send is the strongest
//! evidence a bench with one radio can produce, and that evidence says nothing about the other end.

#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod csma;
pub mod receive;

/// `aUnitBackoffPeriod` at 2.4 GHz: 20 symbol periods, one symbol being 16 us, so one unit is 320 us.
pub const SYMBOLS_PER_BACKOFF_UNIT: u16 = 20;

/// The largest payload an 802.15.4 PHY header can describe, per the standard's one-byte length field.
pub const MAX_PHY_PAYLOAD: usize = 127;

/// Frame control: data frame, PAN ID compression, destination and source addresses short (2 bytes each).
///
/// `0x8841` little-endian is `[0x41, 0x88]` on the wire, which is what the board measured.
pub const FCF_DATA_SHORT_SHORT: u16 = 0x8841;

/// Frame control for an acknowledgement: frame type 0b010, no addresses, no payload.
pub const FCF_ACK: u16 = 0x0002;

/// A MAC address: what a frame is addressed to, and where it claims to come from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Address {
    /// The PAN's broadcast address, `0xFFFF`, in a broadcast PAN.
    Broadcast,
    /// A specific short address on a specific PAN.
    Short {
        /// The PAN the address lives on.
        pan: u16,
        /// The 16-bit address inside that PAN.
        address: u16,
    },
}

/// Sequence numbers run 0..=255 and wrap; a duplicate is how a receiver spots a retransmission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sequence(u8);

impl Sequence {
    /// Start a sequence at a given value, which is what a freshly powered node does from its own storage.
    pub const fn new(start: u8) -> Self {
        Self(start)
    }

    /// The value to put on the wire now.
    pub const fn current(self) -> u8 {
        self.0
    }

    /// Advance for the next frame, wrapping at 255 the way the standard requires.
    ///
    /// Named `advance` and not `next`: a method called `next` on a counter reads like `Iterator::next`, and
    /// clippy is right that a reader can be misled by it.
    pub fn advance(&mut self) -> u8 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }
}

/// Why a frame could not be built. Each variant is a caller mistake, not a radio condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildError {
    /// The payload cannot be described by the one-byte PHY length field.
    PayloadTooLong,
    /// The caller's buffer cannot hold the frame, and a short write silently truncates a packet.
    BufferTooSmall,
}

/// Build a data frame ready for the transmit FIFO, **length byte first**.
///
/// Returns the number of bytes written to `out`, which is `1 + 9 + payload.len()`: one byte of PHY header, nine
/// bytes of MAC header, then the payload. The `1` is the entire point of this function -- see the crate docs.
///
/// # Errors
///
/// [`BuildError::PayloadTooLong`] if the payload does not fit the PHY length field, and
/// [`BuildError::BufferTooSmall`] if `out` cannot hold the whole frame. A partial frame is never written.
pub fn build_data_frame(
    sequence: u8,
    source: Address,
    destination: Address,
    payload: &[u8],
    out: &mut [u8],
) -> Result<usize, BuildError> {
    // The PHY length counts the MAC frame that follows it: FCF (2) + sequence (1) + addresses.
    let mac_len = 9 + payload.len();
    if mac_len > MAX_PHY_PAYLOAD {
        return Err(BuildError::PayloadTooLong);
    }
    let total = 1 + mac_len;
    if out.len() < total {
        return Err(BuildError::BufferTooSmall);
    }

    out[0] = mac_len as u8;
    out[1..3].copy_from_slice(&FCF_DATA_SHORT_SHORT.to_le_bytes());
    out[3] = sequence;

    let (dst_pan, dst_addr) = match destination {
        Address::Broadcast => (0xFFFF, 0xFFFF),
        Address::Short { pan, address } => (pan, address),
    };
    out[4..6].copy_from_slice(&dst_pan.to_le_bytes());
    out[6..8].copy_from_slice(&dst_addr.to_le_bytes());

    // PAN ID compression is on, so the source PAN is the destination PAN and is not repeated on the wire. The
    // source's own PAN is therefore irrelevant to the bytes; only its address is written.
    let src_addr = match source {
        Address::Broadcast => 0xFFFF,
        Address::Short { address, .. } => address,
    };
    out[8..10].copy_from_slice(&src_addr.to_le_bytes());

    out[10..10 + payload.len()].copy_from_slice(payload);
    Ok(total)
}

/// Whether `frame` -- a frame as the radio delivered it -- acknowledges `sequence`.
///
/// The length byte counts the MAC frame but **not** the hardware's two-byte frame check sequence, which the
/// radio appends and delivers along with it. So an acknowledgement arrives as
/// `[3, 0x02, 0x00, sequence, fcs0, fcs1]`: three bytes of MAC frame counted, two more present. Requiring the
/// two to be equal would reject every real acknowledgement, which is how a receiver decides that every frame it
/// sent went unanswered.
///
/// A receiver that cannot make this decision precisely will either retransmit a frame that was received -- the
/// air fills with duplicates -- or accept silence as success, and the frame is lost with no error anywhere.
pub fn is_ack_for(frame: &[u8], sequence: u8) -> bool {
    // PHY header (1) plus the two bytes of frame control and one of sequence.
    if frame.len() < 4 {
        return false;
    }
    // The length byte must not claim more MAC bytes than actually arrived: that is a frame we misread.
    if 1 + usize::from(frame[0]) > frame.len() {
        return false;
    }
    let fcf = u16::from_le_bytes([frame[1], frame[2]]);
    let frame_type = fcf & 0b111;
    frame_type == FCF_ACK & 0b111 && frame[3] == sequence
}

/// What to do after sending a frame and waiting for its acknowledgement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    /// An ACK arrived: the frame is done.
    Done,
    /// No ACK yet and attempts remain: wait `backoff_symbols` then transmit the same frame again.
    Retry {
        /// How long to wait before retransmitting, in 802.15.4 symbol periods (16 us each at 2.4 GHz).
        backoff_symbols: u16,
        /// Which attempt the retransmission will be, counting the original as attempt 0.
        attempt: u8,
    },
    /// Attempts are spent. The frame is lost; this is a reported failure, not a silent one.
    Exhausted,
}

/// The retransmission policy: bounded attempts, and a backoff that grows with each failure.
///
/// `max_attempts` counts **retransmissions**, so a frame gets `max_attempts + 1` transmissions in total. The
/// backoff is exponential over a caller-supplied random value, which keeps this crate free of any randomness
/// source of its own -- a test can pin it, and the caller can feed it what the radio's random generator
/// produced.
#[derive(Clone, Copy, Debug)]
pub struct RetryPolicy {
    /// Retransmissions allowed after the first transmission.
    pub max_attempts: u8,
    /// The upper bound of the first backoff window, in symbol periods.
    pub unit_backoff_symbols: u16,
}

impl RetryPolicy {
    /// A policy matching the standard's default of at most three retransmissions.
    pub const DEFAULT: Self = Self {
        max_attempts: 3,
        unit_backoff_symbols: 20,
    };

    /// Decide what happens after an attempt whose ACK did or did not arrive.
    ///
    /// `attempt` is the attempt that just finished, counting the original transmission as 0. `random` is any
    /// value the caller has; only its low bits are consumed, uniformly per window.
    pub fn after(&self, attempt: u8, acked: bool, random: u32) -> Next {
        if acked {
            return Next::Done;
        }
        if attempt >= self.max_attempts {
            return Next::Exhausted;
        }
        // Exponential window: 1, 2, 4, 8 units. Shifting by the attempt keeps the first retry short, which is
        // what matters when the ACK was merely in flight rather than lost.
        let units: u32 = 1u32 << core::cmp::min(attempt, 4);
        let window = u32::from(self.unit_backoff_symbols).saturating_mul(units);
        let backoff_symbols = if window == 0 {
            0
        } else {
            (random % window) as u16
        };
        Next::Retry {
            backoff_symbols,
            attempt: attempt + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: Address = Address::Short {
        pan: 0x1234,
        address: 0x1234,
    };
    const DST: Address = Address::Broadcast;

    fn frame_of(payload: &[u8]) -> [u8; 64] {
        let mut out = [0u8; 64];
        build_data_frame(7, SRC, DST, payload, &mut out).expect("frame fits");
        out
    }

    /// The invariant whose absence cost a night on the board: byte 0 describes the bytes after it, and the
    /// radio transmits exactly `frame[0]` bytes of MAC frame behind it.
    #[test]
    fn the_first_byte_is_the_length() {
        let mut buf = [0u8; MAX_PHY_PAYLOAD + 1];
        let payload = [0xABu8; MAX_PHY_PAYLOAD - 9];
        for payload_len in 0..=MAX_PHY_PAYLOAD - 9 {
            let n = build_data_frame(1, SRC, DST, &payload[..payload_len], &mut buf).unwrap();
            assert_eq!(
                usize::from(buf[0]),
                n - 1,
                "byte 0 must count everything after it, for payload {payload_len}"
            );
            assert_eq!(n, 10 + payload_len);
        }
    }

    /// A frame control read as a length is exactly the confusion that produced TX_UNDERFLOW: 0x41 is 65.
    #[test]
    fn a_frame_control_is_not_mistaken_for_a_length() {
        let frame = frame_of(b"dosim");
        assert_eq!(frame[0], 14, "14 bytes of MAC frame follow the PHY header");
        assert_eq!(
            frame[1], 0x41,
            "the frame control sits at byte 1, where the radio expects the MAC frame to start"
        );
        assert_ne!(
            frame[0], frame[1],
            "the byte the old code wrote first (0x41, a frame control) must no longer be byte 0"
        );
        assert_eq!(usize::from(frame[0]) + 1, 15);
    }

    #[test]
    fn the_header_layout_is_the_one_the_standard_describes() {
        let frame = frame_of(b"dosim");
        assert_eq!(&frame[1..3], &FCF_DATA_SHORT_SHORT.to_le_bytes());
        assert_eq!(frame[3], 7, "sequence number");
        assert_eq!(&frame[4..6], &[0xFF, 0xFF], "destination PAN: broadcast");
        assert_eq!(
            &frame[6..8],
            &[0xFF, 0xFF],
            "destination address: broadcast"
        );
        assert_eq!(&frame[8..10], &[0x34, 0x12], "source address, not its PAN");
        assert_eq!(&frame[10..15], b"dosim");
    }

    #[test]
    fn pan_compression_keeps_the_source_pan_off_the_wire() {
        let mut buf = [0u8; 32];
        let n = build_data_frame(
            3,
            Address::Short {
                pan: 0xABCD,
                address: 0x0102,
            },
            Address::Short {
                pan: 0x1234,
                address: 0x0304,
            },
            b"x",
            &mut buf,
        )
        .unwrap();
        assert_eq!(n, 11);
        assert_eq!(&buf[4..6], &[0x34, 0x12], "destination PAN");
        assert_eq!(&buf[6..8], &[0x04, 0x03], "destination address");
        assert_eq!(&buf[8..10], &[0x02, 0x01], "source address only");
        assert!(
            !buf[..n].windows(2).any(|w| w == [0xCD, 0xAB]),
            "the source PAN 0xABCD must not appear anywhere"
        );
    }

    #[test]
    fn an_oversized_payload_is_refused_rather_than_truncated() {
        let mut buf = [0u8; 256];
        let payload = [0u8; MAX_PHY_PAYLOAD - 8];
        assert_eq!(
            build_data_frame(0, SRC, DST, &payload, &mut buf),
            Err(BuildError::PayloadTooLong)
        );
    }

    #[test]
    fn a_short_buffer_is_refused_rather_than_half_written() {
        let mut buf = [0u8; 14];
        let before = buf;
        assert_eq!(
            build_data_frame(0, SRC, DST, b"dosim", &mut buf),
            Err(BuildError::BufferTooSmall)
        );
        assert_eq!(
            buf, before,
            "nothing is written when the frame does not fit"
        );
    }

    #[test]
    fn an_ack_with_the_wrong_sequence_is_not_an_acknowledgement() {
        // Length 3, ACK frame control, sequence 42. Both shapes the radio can hand over: with the two-byte
        // frame check sequence it appends, and without it.
        let with_fcs = [0x03u8, 0x02, 0x00, 42, 0xAA, 0xBB];
        let without_fcs = [0x03u8, 0x02, 0x00, 42];
        for ack in [&with_fcs[..], &without_fcs[..]] {
            assert!(is_ack_for(ack, 42), "sequence 42 is ours in {ack:?}");
            assert!(!is_ack_for(ack, 41), "another frame's ACK is not ours");
        }
    }

    #[test]
    fn a_data_frame_is_never_read_as_an_acknowledgement() {
        let frame = frame_of(b"dosim");
        assert!(!is_ack_for(&frame[..15], 7));
    }

    #[test]
    fn a_truncated_ack_is_rejected() {
        assert!(!is_ack_for(&[], 0));
        assert!(!is_ack_for(&[0x03, 0x02], 0));
        // A length byte that disagrees with how many bytes arrived is a frame we misread.
        assert!(!is_ack_for(&[0x09, 0x02, 0x00, 7], 7));
    }

    #[test]
    fn a_sequence_wraps_at_255() {
        let mut seq = Sequence::new(254);
        assert_eq!(seq.current(), 254);
        assert_eq!(seq.advance(), 255);
        assert_eq!(seq.advance(), 0);
        assert_eq!(seq.current(), 0);
    }

    #[test]
    fn an_acknowledged_frame_is_done_whatever_the_attempt() {
        let policy = RetryPolicy::DEFAULT;
        assert_eq!(policy.after(0, true, 0), Next::Done);
        assert_eq!(policy.after(3, true, 12345), Next::Done);
    }

    #[test]
    fn attempts_are_bounded_and_exhaustion_is_reported() {
        let policy = RetryPolicy {
            max_attempts: 2,
            unit_backoff_symbols: 20,
        };
        // `max_attempts` counts retransmissions, so two of them means three transmissions in total: attempts
        // 0, 1 and 2 each get a retry after them, and the failure on attempt 2 spends the last one.
        let mut transmissions = 1;
        let mut attempt = 0u8;
        loop {
            match policy.after(attempt, false, 0) {
                Next::Retry { attempt: next, .. } => {
                    transmissions += 1;
                    attempt = next;
                }
                Next::Exhausted => break,
                Next::Done => panic!("no ACK was passed in"),
            }
        }
        assert_eq!(transmissions, 3, "one original plus two retransmissions");
        assert_eq!(attempt, 2, "the last attempt made was attempt 2");
    }

    #[test]
    fn backoff_grows_with_each_failure_and_stays_inside_its_window() {
        let policy = RetryPolicy {
            max_attempts: 4,
            unit_backoff_symbols: 20,
        };
        for attempt in 0..4u8 {
            let window = 20u32 << u32::from(attempt);
            for random in [0u32, 1, 19, 20, 21, 1_000, u32::MAX] {
                match policy.after(attempt, false, random) {
                    Next::Retry {
                        backoff_symbols, ..
                    } => {
                        assert!(
                            u32::from(backoff_symbols) < window,
                            "attempt {attempt}: {backoff_symbols} must be under {window}"
                        );
                    }
                    other => panic!("attempt {attempt} should retry, got {other:?}"),
                }
            }
        }
    }

    #[test]
    fn the_same_random_value_gives_the_same_backoff() {
        let policy = RetryPolicy::DEFAULT;
        assert_eq!(policy.after(1, false, 9_999), policy.after(1, false, 9_999));
    }

    #[test]
    fn the_retry_reports_which_attempt_comes_next() {
        let policy = RetryPolicy::DEFAULT;
        assert_eq!(
            policy.after(0, false, 0),
            Next::Retry {
                backoff_symbols: 0,
                attempt: 1
            }
        );
    }
}
