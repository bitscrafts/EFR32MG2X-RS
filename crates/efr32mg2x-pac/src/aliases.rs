//! ALIASES. Register block; neither the vendor datasheet nor the reference manual's chapter list describes this block.
//!
//! On this family several peripheral registers can be written three ways: the plain register, a **SET** alias where a
//! written `1` sets bits and a written `0` does nothing, and a **CLR** alias where a `1` clears. The reference manual and
//! the vendor headers place the aliases in two blocks *inside each peripheral's register frame*: SET at offset
//! `0x1000`, CLR at `0x2000`, with the same register offsets inside each block as in the plain frame.
//!
//! The SVD this crate is generated from carries only the plain block -- `efr32mg24_pcnt.h:62` shows the gap it leaves,
//! `uint32_t RESERVED0[1008U]` right after the last plain register, and then `EN_SET`, `CMD_SET`, ... and further down
//! `EN_CLR`, `CMD_CLR`. So `svd2rust` produced `pcnt0_ns::RegisterBlock` with fifteen registers and no way to name the
//! aliases. Writing the plain register instead is not a harmless substitute: measured on the PCNT block, 2026-11-28, it
//! raised a precise bus error with `BFAR = PCNT0 + 0x14` and `CFSR = 0x8200`, twice, and the vendor's own driver only
//! ever writes `CMD_SET` and `EN_SET`/`EN_CLR`.
//!
//! This module is **hand-written**, not generated. If the SVD is ever replaced by one that describes the aliases, delete
//! this file and use the generated accessors instead -- the addresses below are then redundant, not wrong.
//!
//! ```ignore
//! // One command word into PCNT0's CMD SET alias.
//! unsafe { pac::aliases::set(pac::Pcnt0Ns::ptr() as usize, 0x14, 0x002) };
//! ```

/// Offset of the SET alias block inside a peripheral's register frame.
pub const SET_BLOCK: usize = 0x1000;

/// Offset of the CLR alias block inside a peripheral's register frame.
pub const CLR_BLOCK: usize = 0x2000;

/// Write bits into the SET alias of `register` inside the block at `base`.
///
/// Every `1` in `value` sets that bit in the underlying register; every `0` leaves it alone. This is the form the vendor
/// drivers use for command and enable registers.
///
/// # Safety
///
/// `base` must be the address of a peripheral whose register map has the alias blocks -- that is: a peripheral whose
/// vendor header shows the `RESERVED0` gap this module exists for. `register` must be the offset of a real register
/// inside that frame, and the bits in `value` must be ones the register accepts. Writing an address that is not a
/// peripheral raises a precise bus error, which is how the first attempt at this failed.
#[inline(always)]
pub unsafe fn set(base: usize, register: usize, value: u32) {
    // SAFETY: the caller guarantees the address and the register offset; a volatile write is the only access these
    // aliases support (reading them is meaningless).
    unsafe { core::ptr::write_volatile((base + SET_BLOCK + register) as *mut u32, value) };
}

/// Write bits into the CLR alias of `register` inside the block at `base`.
///
/// Every `1` in `value` clears that bit; every `0` leaves it alone.
///
/// # Safety
///
/// Same contract as [`set`].
#[inline(always)]
pub unsafe fn clr(base: usize, register: usize, value: u32) {
    // SAFETY: see `set`.
    unsafe { core::ptr::write_volatile((base + CLR_BLOCK + register) as *mut u32, value) };
}
