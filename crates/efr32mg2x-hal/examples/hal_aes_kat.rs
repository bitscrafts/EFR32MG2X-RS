//! `RADIOAES` against the two published AES-128 known answers, plus the controls that make them mean something.
//!
//! ## What is being proved
//!
//! A crypto engine that returns its input, a zero block, or the same block for every key produces a number that looks
//! like a ciphertext. The defect this example exists to catch is a descriptor chain that is written and never fetched --
//! the register sequence completes, `STATUS` goes idle, and the buffer holds whatever the caller left there.
//!
//! 1. **FIPS-197 C.1.** Key `000102...0f`, plaintext `001122...ff`, and the published ciphertext
//!    `69c4e0d86a7b0430d8cdb78070b4c55a` must come back byte for byte.
//! 2. **The same call repeated.** A second encryption of the same block must give the same bytes: an engine whose output
//!    is the previous result, or noise, fails here and not silently.
//! 3. **A second key and a second plaintext** (NIST SP 800-38A F.1.1 block 1, `3ad77bb4...ef97`). One vector can be
//!    right for the wrong reason -- a key that is ignored still passes if the vector happens to be the identity of what
//!    the engine does -- so the pass has to survive a key it has never seen.
//! 4. **The decrypt bit returns the plaintext.** The same ciphertext under the same key with the mode word's bit 0 set
//!    must produce FIPS-197's plaintext. This is what says the mode word is being fetched rather than ignored.
//! 5. **Control: the output buffer is prefilled and must change.** Every output buffer starts as `0xA5`; a run that
//!    never moved a byte fails on the first comparison rather than passing on a stale buffer.
//! 6. **Control: `IF` is clear before the first start.** The record carries the flags before and after, so a run whose
//!    `IF` was already set cannot be mistaken for one the engine set.
//!
//! What this does **not** claim: any mode other than ECB, any key other than 128 bits, throughput, or the interrupt path.
//!
//! VERIFIED ON HARDWARE (2026-11-27): the record and what each run rules out are in
//! `docs/04-reference/AES-KAT-2026-11-27.md`.

#![no_std]
#![no_main]

use efr32mg2x_hal::aes::{self, Aes};
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::entry;
use efr32mg2x_hal::record::Record;
use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// The byte every output buffer starts as. A distinct, non-zero, non-key value: a run that writes nothing is visible.
const FILL: u8 = 0xA5;

/// Progress, written at every step: "no record" then says something died and where.
#[used]
#[no_mangle]
static ETAPAS: [core::sync::atomic::AtomicU32; 8] =
    [const { core::sync::atomic::AtomicU32::new(0) }; 8];

fn etapa(i: usize, v: u32) {
    ETAPAS[i].store(v, core::sync::atomic::Ordering::Relaxed);
}

/// Append sixteen bytes as thirty-two hexadecimal digits.
fn block_hex(record: &mut Record<200>, bytes: &[u8; 16]) {
    for byte in bytes {
        record.h(*byte);
    }
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = match Board::take() {
        Some(board) => {
            etapa(0, 1);
            board
        }
        None => {
            etapa(0, 9);
            mailbox::fail(b"aes: peripherals taken twice");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut aes: Aes = match board.aes() {
        Ok(aes) => {
            etapa(1, 1);
            aes
        }
        Err(_) => {
            etapa(1, 9);
            mailbox::fail(b"aes: RADIOAES refused its reset");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let core_config = aes.core_config();

    // Control: nothing has been started yet, so `IF` must be empty. Clearing first makes the control independent of
    // whatever the reset left behind.
    aes.clear_flags();
    let flags_before = aes.flags();

    // ---- 1: FIPS-197 C.1 -------------------------------------------------------------------------------------------
    let mut ciphertext = [FILL; 16];
    if aes
        .encrypt_block(&aes::FIPS197.key, &aes::FIPS197.plaintext, &mut ciphertext)
        .is_err()
    {
        etapa(2, 9);
        mailbox::fail(b"aes: the first block never finished");
        loop {
            cortex_m::asm::wfi();
        }
    }
    etapa(2, 1);

    // ---- 2: the same call repeated ---------------------------------------------------------------------------------
    let mut again = [FILL; 16];
    let repeated_ok = aes
        .encrypt_block(&aes::FIPS197.key, &aes::FIPS197.plaintext, &mut again)
        .is_ok();
    etapa(3, 1 + u32::from(repeated_ok));

    // ---- 3: a second key and a second plaintext --------------------------------------------------------------------
    let mut second = [FILL; 16];
    let second_ok = aes
        .encrypt_block(
            &aes::SP800_38A_ECB_BLOCK1.key,
            &aes::SP800_38A_ECB_BLOCK1.plaintext,
            &mut second,
        )
        .is_ok();
    etapa(4, 1 + u32::from(second_ok));

    // ---- 4: the decrypt bit ----------------------------------------------------------------------------------------
    let mut plaintext = [FILL; 16];
    let decrypt_ok = aes
        .decrypt_block(&aes::FIPS197.key, &aes::FIPS197.ciphertext, &mut plaintext)
        .is_ok();
    etapa(5, 1 + u32::from(decrypt_ok));

    let flags_after = aes.flags();
    let status = aes.status();
    etapa(6, 1);

    // ---- verdict ---------------------------------------------------------------------------------------------------
    let blocks: u32 = 4;
    let fips_answered = aes::answered(&aes::FIPS197, &ciphertext);
    // Control 5: the buffer was prefilled, so a byte-for-byte match is also a statement that the engine wrote it.
    let buffer_written = ciphertext != [FILL; 16];
    let repeated = repeated_ok && again == ciphertext;
    let second_answered = second_ok && aes::answered(&aes::SP800_38A_ECB_BLOCK1, &second);
    // A different key and a different plaintext cannot produce this vector's ciphertext; if they did, the comparison in
    // point 1 would be measuring a constant rather than the engine.
    let second_is_distinct = aes::differences(&aes::FIPS197, &second) == 16;
    let decrypted = decrypt_ok && plaintext == aes::FIPS197.plaintext;

    let ok = fips_answered
        && buffer_written
        && repeated
        && second_answered
        && second_is_distinct
        && decrypted
        && flags_before == 0
        && !aes::is_busy(status);

    etapa(7, 1);

    let mut record: Record<200> = Record::new();
    record
        .s(b"aes: ok=")
        .u(ok as u32)
        .s(b" n=")
        .u(blocks)
        .s(b" ct=");
    block_hex(&mut record, &ciphertext);
    record.s(b" c2=");
    block_hex(&mut record, &second);
    record.s(b" pt=");
    block_hex(&mut record, &plaintext);
    record
        .s(b" f0=0x")
        .h32(flags_before)
        .s(b" f1=0x")
        .h32(flags_after)
        .s(b" st=0x")
        .h32(status)
        .s(b" hw=0x")
        .h32(core_config);

    if ok {
        mailbox::pass(record.bytes());
    } else {
        mailbox::fail(record.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
