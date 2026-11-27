//! The `RADIOAES` descriptor arithmetic and the published known answers, compiled on the host so its tests run here.
//!
//! The HAL cannot be compiled for an `x86_64` host -- it contains ARM `asm!` -- so `cores/aes.rs` is the only place the
//! driver's words can be observed without a board. This crate includes that file by path, which keeps one copy and runs
//! the tests that live next to it (`cargo test -p efr32mg2x-aes-core`).
//!
//! What is tested is the mapping from a mode and a parameter offset to the words the engine is handed, and that the
//! known-answer predicate rejects a block it should reject; what is **not** tested, because it cannot be, is whether the
//! part's crypto core answers those vectors. That answer is on the board:
//! `docs/04-reference/AES-KAT-2026-11-27.md`.
//!
//! # One ECB block, as the chain the engine fetches
//!
//! ```
//! use efr32mg2x_aes_core::{
//!     answered, config_tag, data_tag, differences, ecb_control, is_busy, length_word, CFG_OFFSET_KEY,
//!     CFG_OFFSET_MODE, FIPS197,
//! };
//!
//! // The fetcher chain for a single block: the mode word at parameter offset 0, the key at 8, then sixteen bytes of
//! // payload, and only the payload block carries the end-of-chain bit.
//! assert_eq!(config_tag(CFG_OFFSET_MODE), 0x0011);
//! assert_eq!(config_tag(CFG_OFFSET_KEY), 0x0811);
//! assert_eq!(data_tag(false), 0x0001);
//! assert_eq!(data_tag(true), 0x0021);
//! assert_eq!(length_word(16), 16);
//!
//! // Decrypt differs from encrypt in one bit of the mode word and in nothing else.
//! assert_eq!(ecb_control(false), 0x0100);
//! assert_eq!(ecb_control(true), 0x0101);
//! assert!(is_busy(0x0003), "either half still fetching or pushing is work in progress");
//!
//! // The engine's answer is compared against the published ciphertext of its own vector, so a block that comes back
//! // under the wrong key is a failure and not a reading. Both directions are shown, because a predicate that only ever
//! // says yes is not a test.
//! assert!(answered(&FIPS197, &FIPS197.ciphertext));
//! assert!(!answered(&FIPS197, &FIPS197.plaintext));
//! assert_eq!(differences(&FIPS197, &FIPS197.plaintext), 16);
//! ```
#![deny(missing_docs)]
#![no_std]
#![forbid(unsafe_code)]

#[path = "../../../crates/efr32mg2x-hal/src/cores/aes.rs"]
mod driver;

pub use driver::{
    answered, config_tag, data_tag, differences, ecb_control, is_busy, length_word, push_tag,
    KnownAnswer, CFG_OFFSET_IV, CFG_OFFSET_KEY, CFG_OFFSET_MODE, CMD_START, CTRL_SCATTER_GATHER,
    ENGINE_BA411E, FIPS197, KEY_AES128, LENGTH_MASK, MODE_DECRYPT, MODE_ECB, NEXT_STOP,
    SP800_38A_ECB_BLOCK1, STATUS_BUSY, TAG_CONFIG, TAG_LAST,
};
