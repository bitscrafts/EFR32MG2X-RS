//! The `RADIOAES` scatter-gather words, and the two published known answers this crate compares against.
//!
//! `RADIOAES` has no key register, no data register and no mode register. The block is driven by a descriptor chain the
//! engine fetches from RAM: the fetcher walks a list of `{address, next, length, tag}` words, the pusher walks a second
//! list, and every item that is not payload is a *configuration block* whose tag names where in the crypto core's
//! parameter space it lands -- the key at offset 8, the mode word at offset 0, the IV at offset 40. That makes the tag
//! and control-word arithmetic the part of the driver that can be wrong while every register still reads back what was
//! written, which is exactly why it is here, where a host test can run it.
//!
//! Where the numbers come from: the tag bits and the parameter offsets are the ones the vendor's own RADIOAES driver
//! uses (`sli_protocol_crypto_radioaes.c`, vendored in `crates/efr32mg24-bluetooth-sys/vendor/support` for the BLE
//! stack, with the register block in `efr32mg24_aes.h`). No code is copied from it; what is taken is the interface, and
//! the driver in `src/aes/mod.rs` implements it in Rust. The known answers are the standards' own: FIPS-197 appendix C.1
//! and NIST SP 800-38A appendix F.1.1 (ECB-AES128, block 1). Both were re-derived on the host with
//! `openssl enc -aes-128-ecb -nopad` before they were written down, so a typo in a vector cannot pass as a measurement
//! of the part.
//!
//! This file is compiled by `libraries/aes-core` through `#[path]`, which is where the tests below run.

/// Engine selector in a scatter-gather tag: the block goes through the BA411E AES core.
pub const ENGINE_BA411E: u32 = 0x0000_0001;

/// The block carries configuration (mode, key, IV) rather than payload.
pub const TAG_CONFIG: u32 = 0x0000_0010;

/// The block is the last of its chain. The engine stops fetching after it.
pub const TAG_LAST: u32 = 0x0000_0020;

/// `CTRL` in scatter-gather mode: both the fetcher and the pusher follow descriptor lists instead of their registers.
pub const CTRL_SCATTER_GATHER: u32 = 0x0000_0003;

/// `CMD`: start the fetcher and the pusher together.
pub const CMD_START: u32 = 0x0000_0003;

/// Where the mode word lands in the crypto core's parameter space.
pub const CFG_OFFSET_MODE: u8 = 0;
/// Where the key lands. Offset 8 is the core's key word, not the key's second word.
pub const CFG_OFFSET_KEY: u8 = 8;
/// Where an IV lands, for the modes that take one. This driver only runs ECB, which takes none.
pub const CFG_OFFSET_IV: u8 = 40;

/// The mode word's ECB selector.
pub const MODE_ECB: u32 = 0x0000_0100;
/// The mode word's decrypt selector. Absent means encrypt.
pub const MODE_DECRYPT: u32 = 0x0000_0001;
/// The mode word's 128-bit key selector, which is zero -- the field's reset value.
pub const KEY_AES128: u32 = 0;

/// The low bits of `STATUS`: `FETCHERBSY` and `PUSHERBSY`.
pub const STATUS_BUSY: u32 = 0x0000_0003;

/// The length field of a descriptor is the low 28 bits; the top four carry addressing and interrupt flags.
pub const LENGTH_MASK: u32 = 0x0FFF_FFFF;

/// A descriptor's `next` word when the chain ends: bit 0 set means "stop", not "the descriptor at address 1".
pub const NEXT_STOP: u32 = 0x0000_0001;

/// The mode word for a single-block ECB operation.
///
/// `decrypt` is the whole of the difference: the same word with bit 0 set asks the core to run the inverse cipher. The
/// key size is fixed at 128 bits here because the field is zero for AES-128 and this driver only claims AES-128.
#[must_use]
pub fn ecb_control(decrypt: bool) -> u32 {
    MODE_ECB | KEY_AES128 | if decrypt { MODE_DECRYPT } else { 0 }
}

/// The tag of a configuration block that lands at `offset` in the core's parameter space.
///
/// A configuration block is not data: the engine must not encrypt it, and it must not be the last block of the fetcher
/// chain, because the payload block follows it.
#[must_use]
pub fn config_tag(offset: u8) -> u32 {
    ENGINE_BA411E | TAG_CONFIG | ((offset as u32) << 8)
}

/// The tag of a payload block. `last` marks the end of the fetcher chain.
#[must_use]
pub fn data_tag(last: bool) -> u32 {
    ENGINE_BA411E | if last { TAG_LAST } else { 0 }
}

/// The tag of a pusher block: the pusher has no configuration to carry, so its tag is the engine plus the end marker.
#[must_use]
pub fn push_tag(last: bool) -> u32 {
    data_tag(last)
}

/// The `length` word of a descriptor for `bytes` bytes read with an incrementing address.
#[must_use]
pub fn length_word(bytes: usize) -> u32 {
    (bytes as u32) & LENGTH_MASK
}

/// Whether the engine still has work: either half busy.
#[must_use]
pub fn is_busy(status: u32) -> bool {
    status & STATUS_BUSY != 0
}

/// A published known answer: one key, one plaintext block and the ciphertext the standard says it produces.
#[derive(Debug, Clone, Copy)]
pub struct KnownAnswer {
    /// Where the answer is published, so a reader can check it without trusting this file.
    pub source: &'static str,
    /// The 128-bit key.
    pub key: [u8; 16],
    /// The plaintext block.
    pub plaintext: [u8; 16],
    /// The ciphertext block the standard publishes for (`key`, `plaintext`).
    pub ciphertext: [u8; 16],
}

/// FIPS-197 appendix C.1: the AES-128 example every implementation is checked against first.
pub const FIPS197: KnownAnswer = KnownAnswer {
    source: "FIPS-197 C.1",
    key: [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f,
    ],
    plaintext: [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ],
    ciphertext: [
        0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5,
        0x5a,
    ],
};

/// NIST SP 800-38A appendix F.1.1, block 1: a second key and a second plaintext, so a pass cannot come from one vector
/// that happened to be right for the wrong reason.
pub const SP800_38A_ECB_BLOCK1: KnownAnswer = KnownAnswer {
    source: "NIST SP 800-38A F.1.1, block 1",
    key: [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c,
    ],
    plaintext: [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17,
        0x2a,
    ],
    ciphertext: [
        0x3a, 0xd7, 0x7b, 0xb4, 0x0d, 0x7a, 0x36, 0x60, 0xa8, 0x9e, 0xca, 0xf3, 0x24, 0x66, 0xef,
        0x97,
    ],
};

/// Whether `block` is the ciphertext the standard publishes for this vector.
///
/// Written as a comparison against the published bytes rather than as "the engine answered", so that an engine that
/// returns its input, a zero block or a block under the wrong key is a failure instead of a reading.
#[must_use]
pub fn answered(expected: &KnownAnswer, block: &[u8; 16]) -> bool {
    expected.ciphertext == *block
}

/// How many bytes of `block` differ from this vector's published ciphertext, `0..=16`.
///
/// The count, not the bytes: it is what the record carries for a control run, and a count of 16 is what a different key
/// produces while a count of 0 is an engine that ignored the key.
#[must_use]
pub fn differences(expected: &KnownAnswer, block: &[u8; 16]) -> u32 {
    let mut n = 0;
    for (got, want) in block.iter().zip(expected.ciphertext.iter()) {
        if got != want {
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn the_encrypt_mode_word_is_ecb_with_a_128_bit_key() {
        assert_eq!(ecb_control(false), 0x0000_0100);
        assert_eq!(ecb_control(true), 0x0000_0101);
        // The only difference between the two directions is the decrypt bit.
        assert_eq!(ecb_control(true) ^ ecb_control(false), MODE_DECRYPT);
    }

    #[test]
    fn a_key_block_lands_at_offset_eight_and_a_mode_block_at_offset_zero() {
        assert_eq!(config_tag(CFG_OFFSET_KEY), 0x0000_0811);
        assert_eq!(config_tag(CFG_OFFSET_MODE), 0x0000_0011);
    }

    #[test]
    fn only_the_end_of_a_chain_carries_the_last_bit() {
        assert_eq!(data_tag(false), 0x0000_0001);
        assert_eq!(data_tag(true), 0x0000_0021);
        assert_eq!(push_tag(true), 0x0000_0021);
        // A configuration block is never the last block: the payload it configures follows it.
        assert_eq!(config_tag(CFG_OFFSET_KEY) & TAG_LAST, 0);
    }

    #[test]
    fn a_configuration_block_is_never_mistaken_for_payload() {
        assert_ne!(config_tag(CFG_OFFSET_MODE) & TAG_CONFIG, 0);
        assert_eq!(data_tag(true) & TAG_CONFIG, 0);
    }

    #[test]
    fn the_length_word_keeps_the_addressing_flags_clear() {
        assert_eq!(length_word(16), 16);
        assert_eq!(length_word(4), 4);
        assert_eq!(length_word(16) & !LENGTH_MASK, 0);
    }

    #[test]
    fn only_the_two_busy_bits_make_the_engine_busy() {
        assert!(!is_busy(0));
        assert!(is_busy(1));
        assert!(is_busy(2));
        assert!(is_busy(3));
        assert!(!is_busy(0x0001_0000));
    }

    #[test]
    fn each_published_vector_is_answered_by_its_own_ciphertext_and_by_nothing_else() {
        assert!(answered(&FIPS197, &FIPS197.ciphertext));
        assert!(answered(
            &SP800_38A_ECB_BLOCK1,
            &SP800_38A_ECB_BLOCK1.ciphertext
        ));
        // Controls, both directions: the predicate has to reject the plaintext of its own vector and the ciphertext of
        // the other one, or "answered" would be true for anything.
        assert!(!answered(&FIPS197, &FIPS197.plaintext));
        assert!(!answered(&FIPS197, &SP800_38A_ECB_BLOCK1.ciphertext));
        assert!(!answered(&SP800_38A_ECB_BLOCK1, &FIPS197.ciphertext));
        assert_eq!(differences(&FIPS197, &FIPS197.ciphertext), 0);
        assert_eq!(differences(&FIPS197, &SP800_38A_ECB_BLOCK1.ciphertext), 16);
    }
}
