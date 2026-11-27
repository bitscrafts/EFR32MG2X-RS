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
