//! Comparing a stored bond address with the peer of a connection.
//!
//! Kept apart from `bonding_ext` because it is pure: it can be unit-tested on the host with `rustc --test`, which the
//! rest of that module cannot (it links the vendor's NVM3).

/// Whether a stored bond address belongs to a peer that connected from `(address, address_type)`.
///
/// Only an address that is stable can be compared byte for byte: a public address (type 0), or a static random one
/// (type 1 with the two top bits `0b11`, `address[5] >> 6`). A resolvable private address changes every ~15 minutes and
/// needs the IRK (`resolve_rpa`); a non-resolvable one never identifies a bond. `stored_len` is the length of piece 0.
pub(crate) fn address_matches(
    stored: &[u8],
    stored_len: usize,
    address: [u8; 6],
    address_type: u8,
) -> bool {
    if stored_len < 6 || stored.len() < 6 {
        return false;
    }
    let stable = address_type == 0 || (address_type == 1 && address[5] >> 6 == 0b11);
    stable && stored[..6] == address
}

#[cfg(test)]
mod tests {
    use super::address_matches;

    const PUBLIC: [u8; 6] = [0x56, 0x44, 0x81, 0x80, 0xDE, 0x90];
    /// Static random: the two top bits of the most significant byte (`address[5]`) are set.
    const STATIC_RANDOM: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0xC6];

    #[test]
    fn equal_public_address_matches() {
        assert!(address_matches(&PUBLIC, 6, PUBLIC, 0));
    }

    #[test]
    fn one_byte_different_does_not_match() {
        // Two addresses that share bytes 0-2 were the collision of the old compare.
        for i in 0..6 {
            let mut other = PUBLIC;
            other[i] ^= 0x01;
            assert!(!address_matches(&PUBLIC, 6, other, 0), "byte {i}");
        }
    }

    #[test]
    fn static_random_address_matches_with_type_one() {
        // The old compare required the connection type to equal the stored one (0), so this never matched.
        assert!(address_matches(&STATIC_RANDOM, 6, STATIC_RANDOM, 1));
    }

    #[test]
    fn resolvable_private_address_is_not_compared_bytewise() {
        let mut rpa = STATIC_RANDOM;
        rpa[5] = 0x46; // top bits 0b01
        assert!(!address_matches(&rpa, 6, rpa, 1));
    }

    #[test]
    fn short_stored_piece_never_matches() {
        assert!(!address_matches(&PUBLIC, 5, PUBLIC, 0));
    }
}
