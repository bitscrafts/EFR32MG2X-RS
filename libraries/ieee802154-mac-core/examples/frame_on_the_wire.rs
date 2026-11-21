//! What a 802.15.4 transmission looks like as bytes, and what the retry policy decides.
//!
//! Builds the same frame the board sends (`crates/efr32mg24-rail-sys/examples/rail_ieee802154_tx.rs`), prints it
//! byte by byte with the meaning of each field, and then walks the retry policy through a lost acknowledgement.
//!
//! Run on the host, no radio needed:
//!
//! ```text
//! cargo run -p efr32mg2x-ieee802154-mac-core --target x86_64-unknown-linux-gnu --example frame_on_the_wire
//! ```

use efr32mg2x_ieee802154_mac_core::{
    build_data_frame, is_ack_for, Address, Next, RetryPolicy, Sequence,
};

fn main() {
    let source = Address::Short {
        pan: 0x1234,
        address: 0x1234,
    };
    let destination = Address::Broadcast;

    let mut seq = Sequence::new(7);
    let mut frame = [0u8; 64];
    let written = build_data_frame(seq.current(), source, destination, b"dosim", &mut frame)
        .expect("the frame fits in 64 bytes");

    println!("frame, {} bytes on the wire:", written);
    for (index, byte) in frame[..written].iter().enumerate() {
        println!("  [{index:2}] 0x{byte:02X}  {}", what_is(index, *byte));
    }

    println!();
    println!(
        "the first byte says {} MAC bytes follow it, and {} do",
        frame[0],
        written - 1
    );
    assert_eq!(
        usize::from(frame[0]),
        written - 1,
        "a mismatch here is the TX_UNDERFLOW bug: the radio would ask for bytes that are not there"
    );

    println!();
    println!("sequence number: {}", seq.current());
    println!("next frame carries: {}", seq.advance());

    let ack_with_checksum = [0x03u8, 0x02, 0x00, seq.current(), 0xAA, 0xBB];
    println!();
    println!(
        "does [{}] acknowledge sequence {}? {}",
        hex(&ack_with_checksum),
        seq.current(),
        is_ack_for(&ack_with_checksum, seq.current())
    );
    println!(
        "does the same acknowledgement belong to sequence {}? {}",
        seq.current().wrapping_add(1),
        is_ack_for(&ack_with_checksum, seq.current().wrapping_add(1))
    );

    println!();
    println!("no acknowledgement arriving, with the default policy:");
    let policy = RetryPolicy::DEFAULT;
    let mut attempt = 0u8;
    loop {
        match policy.after(attempt, false, 12_345) {
            Next::Retry {
                backoff_symbols,
                attempt: next_attempt,
            } => {
                let micros = u32::from(backoff_symbols) * 16;
                println!(
                    "  attempt {attempt} unanswered -> wait {backoff_symbols} symbols ({micros} us), \
                     transmit attempt {next_attempt}"
                );
                attempt = next_attempt;
            }
            Next::Exhausted => {
                println!(
                    "  attempt {attempt} unanswered -> attempts spent, report the frame as lost"
                );
                break;
            }
            Next::Done => unreachable!("no acknowledgement was passed in"),
        }
    }
}

fn what_is(index: usize, byte: u8) -> String {
    match index {
        0 => format!("PHY header: {byte} MAC bytes follow"),
        1 => format!("frame control, low byte (0x{byte:02X})"),
        2 => format!("frame control, high byte (0x{byte:02X})"),
        3 => format!("sequence number ({byte})"),
        4 | 5 => "destination PAN".to_string(),
        6 | 7 => "destination address".to_string(),
        8 | 9 => "source address".to_string(),
        _ => format!("payload ({})", byte as char),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
