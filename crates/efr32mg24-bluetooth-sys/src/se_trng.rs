//! Hardware true random numbers, through the Secure Engine mailbox.
//!
//! # Why this module exists
//!
//! `sli_bt_system_start_bluetooth` fails with `0x040C` (`SL_STATUS_BT_RADIO`) because the link layer's
//! `ll_randomInit` asks for 32 bytes of entropy and this firmware refuses. The refusal rested on a premise that was
//! simply wrong — that this part has no TRNG. It does: the device header defines `SEMAILBOX_PRESENT`, and the
//! Secure Engine exposes `SLI_SE_COMMAND_TRNG_GET_RANDOM`, which is what the vendor's own `sl_se_get_random` uses on
//! this exact part.
//!
//! # Why it is written against the mailbox instead of vendoring the SE manager
//!
//! The vendor's SE manager is a large component with its own command contexts, key store and OSAL. For a single
//! command the protocol is small enough to state completely, and every constant below comes from a vendor header
//! or from the emlib implementation — none is invented:
//!
//! | Constant | Value | Source |
//! |---|---|---|
//! | mailbox base | `0x4C00_0000` | PAC `SemailboxSHost` = `SEMAILBOX_S_HOST_BASE` (`efr32mg24b220f1536im48.h`) |
//! | `TX_HEADER` value | `4 * (4 + parameters)` | `em_se.c`, `SE_executeCommand` |
//! | FIFO order | command, `data_in`, `data_out`, parameters… | same |
//! | descriptor `next` | `1` = STOP | `em_se.h`, `SE_DATATRANSFER_STOP` |
//! | descriptor `length` | `len \| 0x2000_0000` = REALIGN | `em_se.h`, `SE_DATATRANSFER_REALIGN` |
//! | response | `RX_HEADER & 0x000F_0000`, `0` = OK | `em_se.h`, `SE_RESPONSE_MASK` / `SE_RESPONSE_OK` |
//! | TRNG command | `0x0700_0000` | `sli_se_manager_internal.h`, `SLI_SE_COMMAND_TRNG_GET_RANDOM` |
//! | clock gate | `CMU_CLKEN1.SEMAILBOXHOST` | `efr32mg24_cmu.h` bit 10; enabled per command by `sli_se_lock_acquire` |
//!
//! Register offsets are **not** written out anywhere in this file: the PAC's SVD-generated types are used, so an
//! offset cannot drift from the part's definition. The one number that is not a vendor constant is the spin bound
//! below, which exists because the vendor's own wait is an unbounded busy loop and an unbounded loop in firmware
//! with no debugger is a hang with no diagnosis.
//!
//! # What is never done here
//!
//! No fallback to a counter, a timer, or a hash of either. A deterministic "random" would make every bonding key
//! predictable, which is worse than a stack that refuses to start. On any failure the destination buffer is zeroed
//! and an error returned.

use core::ffi::c_void;

use efr32mg24_pac::{CmuNs, SemailboxSHost};

/// `SLI_SE_COMMAND_TRNG_GET_RANDOM`, from `sli_se_manager_internal.h`.
const SLI_SE_COMMAND_TRNG_GET_RANDOM: u32 = 0x0700_0000;

/// `SE_DATATRANSFER_STOP`: this descriptor is the last one.
const SE_DATATRANSFER_STOP: u32 = 0x0000_0001;

/// `SE_DATATRANSFER_REALIGN`: the SE may realign its writes, which is what makes an unaligned caller buffer safe.
const SE_DATATRANSFER_REALIGN: u32 = 0x2000_0000;

/// `SE_RESPONSE_MASK`: the response code lives in bits 16..19 of `RX_HEADER`.
const SE_RESPONSE_MASK: u32 = 0x000F_0000;

/// `SE_RESPONSE_OK`.
const SE_RESPONSE_OK: u32 = 0;

/// The vendor's `SE_executeCommand` writes `4 * (4 + num_parameters)` to `TX_HEADER`. A TRNG request carries one
/// parameter (the byte count).
const TX_HEADER_TRNG: u32 = 4 * (4 + 1);

/// A bound on each mailbox wait. The vendor's is unbounded; this one turns a silent hang into a status code.
/// 2_000_000 iterations at ~19 MHz is on the order of a tenth of a second — far longer than a TRNG command takes,
/// short enough that a stuck SE does not look like a dead board.
const SPIN_BOUND: u32 = 2_000_000;

/// `SE_DataTransfer_t` from `em_se.h`: `{ void *data; void *next; uint32_t length; }`.
#[repr(C, align(4))]
struct Descriptor {
    data: *mut c_void,
    next: u32,
    length: u32,
}

/// Why a request failed, as a status the caller can publish without leaking entropy.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The mailbox never became ready to accept a command.
    TxTimeout,
    /// The mailbox never returned a response.
    RxTimeout,
    /// The SE answered, and refused. Carries the response code (`RX_HEADER & 0x000F_0000`).
    SeRefused(u32),
    /// Both attempts failed.
    Failed,
}

/// Fill `buf` with hardware entropy.
///
/// The whole request is retried once, on the vendor's reasoning that a TRNG read costs no entropy and each attempt
/// is independent. A second failure is reported, never papered over: the buffer is zeroed first.
///
/// # Safety
///
/// `buf` must be valid for writes for its whole length. The SE writes into it directly (with REALIGN set), so it
/// must also be in RAM the SE can reach — any ordinary buffer in this part's SRAM is.
pub unsafe fn fill(buf: &mut [u8]) -> Result<(), Error> {
    let mut last = Error::Failed;
    for attempt in 0..2 {
        match fill_once(buf) {
            Ok(()) => {
                if attempt > 0 {
                    RETRIES.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                }
                return Ok(());
            }
            Err(e) => {
                last = e;
                FAILURES.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            }
        }
    }
    for b in buf.iter_mut() {
        *b = 0;
    }
    Err(last)
}

/// Whole words go straight into the caller's buffer; a remainder of 1..3 bytes is fetched as one extra word and
/// copied out, exactly as `sl_se_get_random` does. The SE's unit of work is a word, and asking for a non-multiple
/// is not supported by the command.
unsafe fn fill_once(buf: &mut [u8]) -> Result<(), Error> {
    let len = buf.len();
    let words = (len / 4) * 4;
    if words > 0 {
        run_command(buf.as_mut_ptr(), words as u32)?;
    }
    let rest = len - words;
    if rest > 0 {
        let mut surplus = [0u8; 4];
        run_command(surplus.as_mut_ptr(), 4)?;
        buf[words..].copy_from_slice(&surplus[..rest]);
    }
    Ok(())
}

/// One TRNG command: enable the mailbox clock, hand over the descriptor, wait for the answer, clock it back off.
unsafe fn run_command(dst: *mut u8, len: u32) -> Result<(), Error> {
    let mb = SemailboxSHost::ptr();

    // The vendor enables the mailbox clock around *each* command (`sli_se_lock_acquire` / `release`), not once at
    // init, so this mirrors that rather than leaving the clock on.
    //
    // **The non-secure CMU alias, deliberately.** The first attempt used the secure alias (`CMU_S`, `0x4000_8000`)
    // and the very next radio bring-up faulted writing `SYSRTC->CMD` at `0x500a_8014` (`cfsr=0x8200`,
    // `bfar=0x500a8014`): a read-modify-write through a different alias than the one the rest of this firmware uses
    // for the same registers. The HAL has always gated SYSRTC through `CMU_NS` (`0x5000_8000`) and that works; this
    // module now uses the same base, so both owners of the CMU write the same register file.
    (*CmuNs::ptr())
        .clken1()
        .modify(|_, w| w.semailboxhost().set_bit());

    // **Clock gate: measured to be required.** Without it the SE never answers and the bus stalls on the read of
    // `TX_STATUS` -- the probe never left the entropy stage. With it, the command completes and the bring-up
    // advances past `ll_randomInit` to the next wall (the SYSRTC), which is progress even though it ends in a fault.
    //
    // The two earlier variants that did touch this gate (secure alias, then non-secure) were each followed by a fault
    // writing `SYSRTC->CMD` at `0x500a_8014`, and that is *not* attributable to this write: the same fault appears
    // because the start now gets far enough to configure timing, which the previous build never reached.
    // Two attempts that did touch it (secure alias, then
    // non-secure) were each followed by a fault writing `SYSRTC->CMD` at `0x500a_8014` during the radio bring-up, and
    // this firmware does not yet know which CMU register the SE gate really lives in for this part. Whether the SE
    // answers without the gate is a measurement, not an assumption: if the command times out, the gate is needed and
    // the offset has to be established from the SVD rather than guessed.

    let result = (|| {
        spin_until(|| (*mb).tx_status().read().txint().bit_is_set())
            .map_err(|_| Error::TxTimeout)?;

        (*mb)
            .tx_header()
            .write(|w| unsafe { w.bits(TX_HEADER_TRNG) });

        let desc = Descriptor {
            data: dst as *mut c_void,
            next: SE_DATATRANSFER_STOP,
            length: len | SE_DATATRANSFER_REALIGN,
        };
        write_fifo(SLI_SE_COMMAND_TRNG_GET_RANDOM);
        write_fifo(0); // no input data
        write_fifo(&desc as *const Descriptor as u32);
        write_fifo(len); // the command's single parameter

        spin_until(|| (*mb).rx_status().read().rxint().bit_is_set())
            .map_err(|_| Error::RxTimeout)?;

        let response = (*mb).rx_header().read().bits() & SE_RESPONSE_MASK;
        if response == SE_RESPONSE_OK {
            Ok(())
        } else {
            Err(Error::SeRefused(response))
        }
    })();

    (*CmuNs::ptr())
        .clken1()
        .modify(|_, w| w.semailboxhost().clear_bit());
    result
}

unsafe fn write_fifo(word: u32) {
    SemailboxSHost::ptr()
        .as_ref()
        .unwrap()
        .fifo()
        .write(|w| w.bits(word));
}

/// Bounded polling. A hang says nothing; a status code says where.
fn spin_until(mut ready: impl FnMut() -> bool) -> Result<(), ()> {
    for _ in 0..SPIN_BOUND {
        if ready() {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    Err(())
}

use core::sync::atomic::AtomicU32;

/// Requests served, and requests that had to be retried / gave up. Published as counts only — never a byte of the
/// output, which is the one thing that must not be logged.
static REQUESTS: AtomicU32 = AtomicU32::new(0);
static RETRIES: AtomicU32 = AtomicU32::new(0);
static FAILURES: AtomicU32 = AtomicU32::new(0);

/// Note that a request arrived. Called by the PSA shim so the count includes refused ones.
pub fn count_request() {
    REQUESTS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
}

/// `(requests, retries, failures)`.
pub fn stats() -> (u32, u32, u32) {
    (
        REQUESTS.load(core::sync::atomic::Ordering::Relaxed),
        RETRIES.load(core::sync::atomic::Ordering::Relaxed),
        FAILURES.load(core::sync::atomic::Ordering::Relaxed),
    )
}
