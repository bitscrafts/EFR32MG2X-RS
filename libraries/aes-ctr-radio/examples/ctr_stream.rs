//! How the Bluetooth controller's AES-CTR is used, and that it agrees with the specification.
//!
//! Runs on the host: `cargo run --target x86_64-unknown-linux-gnu -p aes-ctr-radio --example ctr_stream`.
//!
//! The two things a caller has to get right are shown here: the counter is the caller's to keep, and the
//! same call both encrypts and decrypts because counter mode XORs a keystream.

use aes_ctr_radio::{ctr_step, encrypt_block};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn main() {
    // 1. One block, against the FIPS-197 worked example (Appendix C.1).
    let key: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f,
    ];
    let plain: [u8; 16] = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ];
    let block = encrypt_block(&key, &plain);
    println!("block  {}", hex(&block));
    assert_eq!(hex(&block), "69c4e0d86a7b0430d8cdb78070b4c55a");

    // 2. Three consecutive counter blocks, which is the shape the controller uses.
    let mut counter = [0u8; 16];
    let message: [u8; 16] = *b"the quick brown ";
    println!("\ncounter            ciphertext");
    for _ in 0..3 {
        let (out, next) = ctr_step(&key, &counter, &message);
        println!("{}  {}", hex(&counter), hex(&out));
        counter = next;
    }

    // 3. Applying the same step to the ciphertext returns the plaintext.
    let (again, _) = ctr_step(&key, &[0u8; 16], &ctr_step(&key, &[0u8; 16], &message).0);
    println!(
        "\nround trip {}",
        if again == message { "ok" } else { "BROKEN" }
    );
    assert_eq!(again, message);
}
