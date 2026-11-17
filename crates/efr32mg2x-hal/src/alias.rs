//! The clear-alias window of a peripheral register block.
//!
//! Every peripheral in this family presents its register block four times: the registers themselves, then the same
//! registers again at `+0x1000` (write 1 **sets** the bit), `+0x2000` (write 1 **clears** it) and `+0x3000` (write 1
//! toggles it). The layout is visible in the vendor's own device headers, e.g. `vendor/device/efr32mg24_i2c.h:79-115`,
//! where `IF`, `IF_SET`, `IF_CLR` and `IF_TGL` are separated by `RESERVED1[1007]` -- one 0x1000-byte window each.
//!
//! Writing 1 to the plain register is the ambiguous case, and the vendor never relies on it: `I2C_IntClear` reaches for
//! `i2c->IF_CLR` whenever the device has the aliases (`vendor445/inc/em_i2c.h:295-300`). Nine sites in this crate wrote
//! 1 to the plain `IF` and called that clearing, so a flag was set instead of cleared -- a NACK that never goes away, an
//! overflow that re-fires, a DMA completion that stays pending. Found by the 2026-11-17 review (H3).
//!
//! The generated PAC exposes only the base window, so the alias is reached by pointer arithmetic on the block type. The
//! block is the same type at a fixed offset, which is why the code below can keep using the typed accessors instead of
//! raw register numbers.

/// The register block of a peripheral, addressed through its clear-alias window (`+0x2000`).
///
/// # Safety
///
/// `base` must be the base pointer of a peripheral register block that provides the alias windows. All of them do in
/// this device family, and the caller is always a driver holding that peripheral.
#[inline]
pub(crate) unsafe fn clear_window<T>(base: *const T) -> &'static T {
    // SAFETY: the caller guarantees `base` is this peripheral's block; the alias windows are part of the same block,
    // mapped at a fixed offset by the hardware, and are always accessible once the peripheral is.
    unsafe { &*((base as usize + CLEAR_OFFSET) as *const T) }
}

/// Offset of the clear window from the base of a register block.
///
/// One 0x1000-byte window for the registers themselves, then one for each alias: `_SET` at 0x1000, `_CLR` at 0x2000.
pub(crate) const CLEAR_OFFSET: usize = 0x2000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clear_window_is_two_windows_above_the_block() {
        // The number is the whole point of this module, so it gets a check that does not depend on the hardware.
        assert_eq!(CLEAR_OFFSET, 0x2000);
        assert_eq!(CLEAR_OFFSET % 0x1000, 0, "windows are 0x1000 apart");
    }
}
