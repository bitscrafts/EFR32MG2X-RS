//! The vectors, printed. This is the example the gate needs to know the crate is built by something, and it is
//! also the fastest way to answer "is the crypto still right?" without a board.
//!
//! `cargo run --target x86_64-unknown-linux-gnu -p aes-modes --example vectors`

use aes_modes::{cmac, decrypt, encrypt, BLE_NONCE_LEN, BLE_TAG_LEN};

fn hex(bytes: &[u8]) -> String {
    let mut s = String::new();
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn main() {
    // RFC 4493, the four CMAC vectors over the 64-byte test message.
    let key: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c,
    ];
    let msg = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17,
        0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf,
        0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb, 0xc1, 0x19, 0x1a,
        0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b,
        0xe6, 0x6c, 0x37, 0x10,
    ];
    println!("CMAC (RFC 4493):");
    for len in [0usize, 16, 40, 64] {
        println!("  {len:>2} bytes -> {}", hex(&cmac(&key, &msg[..len])));
    }

    // Bluetooth parameters: 13-byte nonce, 4-byte tag.
    let ble_key = [0x11u8; 16];
    let nonce = [0x22u8; BLE_NONCE_LEN];
    let mut data = [0u8; 20];
    for (i, b) in data.iter_mut().enumerate() {
        *b = i as u8;
    }
    let plain = data;
    let tag = encrypt(&ble_key, &nonce, &[], &mut data);
    println!("\nCCM* (Bluetooth: M=4, L=2):");
    println!("  plain   {}", hex(&plain));
    println!("  cipher  {}", hex(&data));
    println!("  tag     {}", hex(&tag));
    assert_eq!(tag.len(), BLE_TAG_LEN);

    let mut back = data;
    match decrypt(&ble_key, &nonce, &[], &mut back, &tag) {
        Ok(()) => {
            println!("  round trip ok, plaintext back: {}", back == plain);
            assert_eq!(back, plain);
        }
        Err(_) => panic!("a packet this program just encrypted failed to verify"),
    }

    // A single flipped bit must be refused, and the buffer must be left as it arrived.
    let mut forged = data;
    forged[5] ^= 0x01;
    let before = forged;
    match decrypt(&ble_key, &nonce, &[], &mut forged, &tag) {
        Err(_) => println!(
            "  forged packet refused, buffer untouched: {}",
            forged == before
        ),
        Ok(()) => panic!("a forged packet verified -- the tag check is not doing its job"),
    }
}
