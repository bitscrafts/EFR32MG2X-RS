//! Print the words the driver hands the AES engine and the two published answers it is judged against.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-aes-core --example vectors`
//!
//! Two kinds of number end up on the board's record, and this prints both so the line is auditable without the board:
//! the tag and mode words (which come from the interface, not from a measurement) and the ciphertexts (which come from
//! the standards and from the part). A failure here exits non-zero, so a vector edited into the source cannot pass as a
//! measurement.
//!
//! The published vectors were re-derived on the host with
//! `openssl enc -aes-128-ecb -K <key> -nopad` before they were written into the source.

use efr32mg2x_aes_core::{
    answered, config_tag, data_tag, differences, ecb_control, is_busy, length_word, push_tag,
    CFG_OFFSET_KEY, CFG_OFFSET_MODE, FIPS197, SP800_38A_ECB_BLOCK1,
};

fn hex(bytes: &[u8]) -> String {
    let mut out = String::new();
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn main() {
    let mut failures = 0;

    println!("the words the fetcher is handed:");
    println!("  mode, encrypt          = 0x{:08x}", ecb_control(false));
    println!("  mode, decrypt          = 0x{:08x}", ecb_control(true));
    println!(
        "  key block at offset 8  = 0x{:08x}",
        config_tag(CFG_OFFSET_KEY)
    );
    println!(
        "  mode block at offset 0 = 0x{:08x}",
        config_tag(CFG_OFFSET_MODE)
    );
    println!("  payload, last          = 0x{:08x}", data_tag(true));
    println!("  pusher, last           = 0x{:08x}", push_tag(true));
    println!("  16-byte length word    = 0x{:08x}", length_word(16));

    // Controls on the arithmetic itself: a decrypt word that equalled the encrypt word would make both directions the
    // same request, and a config tag that carried the "last" bit would end the fetcher before the payload.
    if ecb_control(true) == ecb_control(false) {
        println!("FAIL: the two directions share one mode word");
        failures += 1;
    }
    if config_tag(CFG_OFFSET_KEY) & 0x20 != 0 {
        println!("FAIL: a key block marks itself as the last block");
        failures += 1;
    }
    if !is_busy(3) || is_busy(0) {
        println!("FAIL: the busy predicate does not read the two busy bits");
        failures += 1;
    }

    for vector in [FIPS197, SP800_38A_ECB_BLOCK1] {
        println!("\n{}:", vector.source);
        println!("  key        {}", hex(&vector.key));
        println!("  plaintext  {}", hex(&vector.plaintext));
        println!("  ciphertext {}", hex(&vector.ciphertext));
        if !answered(&vector, &vector.ciphertext) {
            println!("FAIL: the vector does not answer itself");
            failures += 1;
        }
        if answered(&vector, &vector.plaintext) || differences(&vector, &vector.plaintext) != 16 {
            println!("FAIL: the predicate accepts the plaintext as its own answer");
            failures += 1;
        }
    }

    // The two vectors must not share an answer either, or the second one proves nothing.
    if answered(&FIPS197, &SP800_38A_ECB_BLOCK1.ciphertext) {
        println!("FAIL: the two vectors share a ciphertext");
        failures += 1;
    }

    if failures == 0 {
        println!("\nPASS: the words are distinct and both published answers are answered");
    } else {
        println!("\nFAIL: {failures} check(s) failed");
        std::process::exit(1);
    }
}
