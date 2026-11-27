//! Whether what a caller declared about a key agrees with what the backend would store.
//!
//! ## Why this is a crate and not a function inside the binding
//!
//! It lived next to the C entry point, where it could not be tested: the binding links ARM-only vendor blobs,
//! so `cargo test` for the host fails at the linker with `could not find native static library emlib_min`. A
//! test that cannot run is not a test -- it is a certificate nobody ever validated. The rule is pure arithmetic
//! over three numbers, so it belongs where the gate can run it (`libraries/`, which the gate builds and tests on
//! the host and forbids from depending on any hardware crate).
//!
//! ## The defect this closes
//!
//! The backend decided a key's type from `data_length` alone: 16 bytes became AES, 65 became an ECC public key,
//! and the type field in the caller's own attributes blob was never read. A caller declaring 256-bit AES and
//! handing over 16 bytes was stored as AES-128 without a word; a caller declaring a raw-data key and handing
//! over 16 bytes was stored as AES. Both are silent, and neither shows up until a MAC comes out wrong or a
//! pairing fails with a reason that points somewhere else.
//!
//! ## Why a zero declaration is accepted
//!
//! A caller that passes a freshly reset attributes blob declares nothing, and the stack does that in places. Zero
//! means "unspecified", not "wrong": the length still decides, exactly as before. Only a declaration that is
//! present **and** contradicts what would be stored is refused.
//!
//! ## What this deliberately does not check
//!
//! The size in bits is checked for the AES path only. `128` was measured in all three of the stack's callers; no
//! measurement exists for what the stack declares for an ECC public key. A check written without that measurement
//! would refuse real pairings to protect a hypothesis.
//!
//! # What a declaration is checked against
//!
//! ```
//! use efr32mg2x_psa_shape_core::{shape_agrees, Declared, ShapeError};
//!
//! // What the stack's own callers declare for a MAC key, measured in all three of them: type 0x2400, 128 bits, 16 bytes.
//! let aes = Declared { key_type: 0x2400, bits: 128 };
//! assert_eq!(shape_agrees(16, aes), Ok(()));
//!
//! // A caller declaring 256-bit AES and handing over 16 bytes was stored as AES-128 without a word until this check
//! // existed; a raw-data key handed 16 bytes was stored as AES, because the type field was never read.
//! assert_eq!(
//!     shape_agrees(16, Declared { key_type: 0x2400, bits: 256 }),
//!     Err(ShapeError::WrongSize)
//! );
//! assert_eq!(
//!     shape_agrees(16, Declared { key_type: 0x1000, bits: 0 }),
//!     Err(ShapeError::WrongType)
//! );
//!
//! // A freshly reset attributes blob declares nothing, and the stack does that in places: zero means unspecified, so
//! // the length still decides and a pairing is not refused for a field the caller never wrote.
//! assert_eq!(shape_agrees(16, Declared::unspecified()), Ok(()));
//! assert_eq!(shape_agrees(65, Declared::unspecified()), Ok(()));
//!
//! // The peer's P-256 public key, 0x04 || X || Y: 65 bytes is the type the caller declared, so it agrees. Its bit size
//! // is not checked, because no measurement of what the stack declares there exists.
//! assert_eq!(
//!     shape_agrees(65, Declared { key_type: 0x4112, bits: 256 }),
//!     Ok(())
//! );
//! ```

#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]

/// `PSA_KEY_TYPE_AES`, as the stack declares it in its own attributes blob.
pub const KEY_TYPE_AES: u32 = 0x2400;

/// `PSA_KEY_TYPE_ECC_PUBLIC_KEY(PSA_ECC_FAMILY_SECP_R1)`, the peer's public key during pairing.
pub const KEY_TYPE_ECC_PUBLIC_KEY_P256: u32 = 0x4112;

/// The length of a P-256 public key as the peer sends it: `0x04 || X || Y`.
pub const SEC1_P256_LEN: usize = 65;

/// The only AES key length this backend's MAC can use, in bytes.
pub const AES_KEY_LEN: usize = 16;

/// The declaration the caller wrote into its attributes blob.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Declared {
    /// The key type as written, or 0 when the caller specified nothing.
    pub key_type: u16,
    /// The size in bits as written, or 0 when the caller specified nothing.
    pub bits: u32,
}

/// Why a declaration was refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShapeError {
    /// The caller named a type that is not what this backend stores for that length.
    WrongType,
    /// The caller named a size the MAC cannot use with the data it supplied.
    WrongSize,
}

impl Declared {
    /// A caller that specified nothing.
    #[must_use]
    pub const fn unspecified() -> Self {
        Self {
            key_type: 0,
            bits: 0,
        }
    }
}

/// Whether `declared` agrees with what would be stored for a payload of `data_length` bytes.
///
/// # Errors
///
/// [`ShapeError::WrongType`] when a type was declared and it is not the one implied by the length;
/// [`ShapeError::WrongSize`] when bits were declared for an AES key and they are not 128.
pub fn shape_agrees(data_length: usize, declared: Declared) -> Result<(), ShapeError> {
    let is_public_key = data_length == SEC1_P256_LEN;
    let expected: u32 = if is_public_key {
        KEY_TYPE_ECC_PUBLIC_KEY_P256
    } else {
        KEY_TYPE_AES
    };

    if declared.key_type != 0 && u32::from(declared.key_type) != expected {
        return Err(ShapeError::WrongType);
    }

    if !is_public_key && declared.bits != 0 && declared.bits != 128 {
        return Err(ShapeError::WrongSize);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{shape_agrees, Declared, ShapeError};

    fn d(key_type: u16, bits: u32) -> Declared {
        Declared { key_type, bits }
    }

    #[test]
    fn the_stacks_own_declaration_is_accepted() {
        // Measured in all three of the stack's callers: type 0x2400, 128 bits, 16 bytes.
        assert_eq!(shape_agrees(16, d(0x2400, 128)), Ok(()));
        // The peer's public key: the type matches the length.
        assert_eq!(shape_agrees(65, d(0x4112, 0)), Ok(()));
    }

    #[test]
    fn an_unspecified_declaration_is_not_a_contradiction() {
        // A freshly reset attributes blob declares nothing; the length still decides.
        assert_eq!(shape_agrees(16, Declared::unspecified()), Ok(()));
        assert_eq!(shape_agrees(65, Declared::unspecified()), Ok(()));
    }

    #[test]
    fn a_contradicting_type_is_refused() {
        // A raw-data key handed 16 bytes used to be stored as AES.
        assert_eq!(shape_agrees(16, d(0x1000, 0)), Err(ShapeError::WrongType));
        // An AES declaration handed the peer's public key.
        assert_eq!(shape_agrees(65, d(0x2400, 128)), Err(ShapeError::WrongType));
    }

    #[test]
    fn a_contradicting_size_is_refused_for_aes() {
        // Declared 256 bits, given 16 bytes: the MAC would never use it.
        assert_eq!(shape_agrees(16, d(0x2400, 256)), Err(ShapeError::WrongSize));
        // 192 is a legal AES size and still not what 16 bytes is.
        assert_eq!(shape_agrees(16, d(0x2400, 192)), Err(ShapeError::WrongSize));
    }

    #[test]
    fn a_public_key_size_is_not_checked_because_it_was_never_measured() {
        // Deliberate: no measurement of what the stack declares here, so a check would be a guess.
        assert_eq!(shape_agrees(65, d(0x4112, 256)), Ok(()));
        assert_eq!(shape_agrees(65, d(0x4112, 0)), Ok(()));
    }
}
