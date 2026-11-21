//! What to do with a frame that arrived, and what to send back.
//!
//! ## The two decisions a receiver cannot avoid
//!
//! A node that hears everything must answer two questions before it touches the frame: **is this addressed to
//! me**, and **what do I put on the air in reply**. Getting the first wrong makes a node acknowledge frames meant
//! for its neighbour, which silences the neighbour's acknowledgement and makes two links fail at once. Getting
//! the second wrong makes every sender retransmit a frame that was already delivered.
//!
//! ## What this does not check, and why
//!
//! The frame check sequence. The radio validates it in hardware and reports the verdict with the received
//! packet; recomputing a CRC here would be a second opinion that costs cycles and can disagree with the thing
//! that actually decides. This module assumes the caller has already discarded frames the radio marked bad.
//!
//! ## The layout it understands
//!
//! Exactly what [`crate::build_data_frame`] produces: PAN ID compression on, destination and source addresses
//! short. A frame in another layout is reported as such instead of being half-parsed -- a receiver that guesses
//! at a layout it does not know will address its acknowledgement to the wrong node.

use crate::{Address, BuildError};

/// The frame types of 802.15.4, from the low three bits of the frame control field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameType {
    /// A beacon, sent by a coordinator.
    Beacon,
    /// A data frame.
    Data,
    /// An acknowledgement.
    Ack,
    /// A MAC command.
    Command,
    /// A type this build does not know; the value is kept so a log can say which.
    Reserved(u8),
}

impl FrameType {
    /// Classify the low three bits of a frame control field.
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0b111 {
            0 => Self::Beacon,
            1 => Self::Data,
            2 => Self::Ack,
            3 => Self::Command,
            other => Self::Reserved(other),
        }
    }
}

/// The verdict on a received frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Received {
    /// A data frame addressed to this node, or to everyone. The sequence number is what the acknowledgement must
    /// carry back, and the source is who to reply on behalf of.
    ForMe {
        /// The sequence number to acknowledge.
        sequence: u8,
        /// The sender's short address.
        source: u16,
    },
    /// A frame in the understood layout, but addressed to another node. It must be left alone: acknowledging it
    /// would silence the node that is actually meant to answer.
    NotMine,
    /// A frame whose layout this build does not understand, or one too short to be whole. Not an error to
    /// report loudly -- simply not something to act on.
    Ignored,
}

/// Decide what a received frame is, given this node's own short address.
///
/// `my_address` is used only when the frame is addressed to a specific node; a broadcast is delivered to
/// everyone regardless.
pub fn classify(frame: &[u8], my_address: u16) -> Received {
    // PHY header, frame control, sequence, destination PAN, destination address, source address.
    if frame.len() < 10 {
        return Received::Ignored;
    }
    if 1 + usize::from(frame[0]) > frame.len() {
        return Received::Ignored;
    }

    let fcf = u16::from_le_bytes([frame[1], frame[2]]);
    if FrameType::from_bits(fcf as u8) != FrameType::Data {
        return Received::Ignored;
    }
    // Only the layout this crate writes: PAN ID compression, both addresses short. A different layout would need
    // a different offset for the source address, and guessing it is how an acknowledgement goes to the wrong
    // node.
    let pan_compression = (fcf >> 6) & 1;
    let dst_short = (fcf >> 10) & 0b11 == 0b10;
    let src_short = (fcf >> 14) & 0b11 == 0b10;
    if pan_compression != 1 || !dst_short || !src_short {
        return Received::Ignored;
    }

    let destination = u16::from_le_bytes([frame[6], frame[7]]);
    if destination != 0xFFFF && destination != my_address {
        return Received::NotMine;
    }

    Received::ForMe {
        sequence: frame[3],
        source: u16::from_le_bytes([frame[8], frame[9]]),
    }
}

/// Build the acknowledgement for `sequence`, ready for the transmit FIFO with the length byte first.
///
/// An acknowledgement carries no payload and no addresses: the PHY header says three MAC bytes follow, then the
/// frame control for an acknowledgement and the sequence number it answers. The radio appends the frame check
/// sequence itself, as it does for any frame.
///
/// # Errors
///
/// [`BuildError::BufferTooSmall`] if `out` cannot hold four bytes.
pub fn build_ack_frame(sequence: u8, out: &mut [u8]) -> Result<usize, BuildError> {
    const ACK_MAC_BYTES: usize = 3;
    if out.len() < 1 + ACK_MAC_BYTES {
        return Err(BuildError::BufferTooSmall);
    }
    out[0] = ACK_MAC_BYTES as u8;
    out[1..3].copy_from_slice(&crate::FCF_ACK.to_le_bytes());
    out[3] = sequence;
    Ok(4)
}

/// Whether this node's own address is what a frame should be sent from, for the case where the caller wants to
/// build a data frame to a specific peer.
pub fn my_address(source: Address) -> u16 {
    match source {
        Address::Broadcast => 0xFFFF,
        Address::Short { address, .. } => address,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_data_frame;

    const ME: u16 = 0x1234;

    fn frame_to(destination: Address, sequence: u8) -> [u8; 64] {
        let mut out = [0u8; 64];
        build_data_frame(
            sequence,
            Address::Short {
                pan: 0x1234,
                address: ME,
            },
            destination,
            b"dosim",
            &mut out,
        )
        .expect("fits");
        out
    }

    #[test]
    fn a_frame_addressed_to_me_is_mine() {
        let frame = frame_to(
            Address::Short {
                pan: 0x1234,
                address: ME,
            },
            42,
        );
        assert_eq!(
            classify(&frame, ME),
            Received::ForMe {
                sequence: 42,
                source: ME
            }
        );
    }

    #[test]
    fn a_broadcast_is_everyones() {
        let frame = frame_to(Address::Broadcast, 7);
        assert!(matches!(
            classify(&frame, ME),
            Received::ForMe { sequence: 7, .. }
        ));
        // And it is everybody's for a different node too, which is what makes it a broadcast.
        assert!(matches!(
            classify(&frame, 0xBEEF),
            Received::ForMe { sequence: 7, .. }
        ));
    }

    /// The control that matters: a frame for somebody else must NOT be acknowledged here. Acknowledging it
    /// silences the node that was meant to answer, so two links fail instead of one frame being retried.
    #[test]
    fn a_frame_for_another_node_is_left_alone() {
        let frame = frame_to(
            Address::Short {
                pan: 0x1234,
                address: 0x5678,
            },
            9,
        );
        assert_eq!(classify(&frame, ME), Received::NotMine);
    }

    #[test]
    fn an_acknowledgement_is_not_delivered_as_data() {
        let mut ack = [0u8; 4];
        build_ack_frame(5, &mut ack).expect("fits");
        assert_eq!(classify(&ack, ME), Received::Ignored);
    }

    #[test]
    fn a_short_frame_is_ignored_rather_than_half_parsed() {
        assert_eq!(classify(&[], ME), Received::Ignored);
        assert_eq!(classify(&[0x0E, 0x41, 0x88], ME), Received::Ignored);
        // A length byte claiming more than arrived is a frame we misread.
        let mut frame = frame_to(Address::Broadcast, 1);
        frame[0] = 100;
        assert_eq!(classify(&frame[..20], ME), Received::Ignored);
    }

    #[test]
    fn frame_types_are_read_from_the_low_three_bits() {
        assert_eq!(FrameType::from_bits(0x41), FrameType::Data);
        assert_eq!(FrameType::from_bits(0x02), FrameType::Ack);
        assert_eq!(FrameType::from_bits(0x00), FrameType::Beacon);
        assert_eq!(FrameType::from_bits(0x03), FrameType::Command);
        assert_eq!(FrameType::from_bits(0x07), FrameType::Reserved(7));
    }

    #[test]
    fn the_acknowledgement_carries_the_length_and_the_sequence() {
        let mut ack = [0u8; 8];
        let n = build_ack_frame(42, &mut ack).expect("fits");
        assert_eq!(n, 4);
        assert_eq!(ack[0], 3, "three MAC bytes follow the PHY header");
        assert_eq!(&ack[1..3], &crate::FCF_ACK.to_le_bytes());
        assert_eq!(ack[3], 42);
        // And it is accepted by the matcher that reads acknowledgements, which is the pair that has to agree.
        assert!(crate::is_ack_for(&ack[..n], 42));
        assert!(!crate::is_ack_for(&ack[..n], 41));
    }

    #[test]
    fn an_acknowledgement_that_cannot_fit_is_refused() {
        let mut small = [0u8; 3];
        assert_eq!(
            build_ack_frame(1, &mut small),
            Err(BuildError::BufferTooSmall)
        );
    }

    #[test]
    fn a_frame_we_build_round_trips_through_classify() {
        // The sender's own address, read back from the frame a receiver sees: the two halves of this crate agree.
        for destination in [
            Address::Broadcast,
            Address::Short {
                pan: 0x1234,
                address: ME,
            },
        ] {
            let frame = frame_to(destination, 200);
            match classify(&frame, ME) {
                Received::ForMe { sequence, source } => {
                    assert_eq!(sequence, 200);
                    assert_eq!(source, ME);
                }
                other => panic!("expected it to be mine, got {other:?}"),
            }
        }
    }
}
