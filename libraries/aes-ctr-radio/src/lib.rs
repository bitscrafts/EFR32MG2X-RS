//! AES-128 in counter mode, in pure Rust, as the Bluetooth controller needs it.
//!
//! The vendor ships this as `sli_aes_crypt_ctr_radio` (Gecko SDK, Zlib), 35 KB of C driving the RADIOAES
//! peripheral. Two reasons this is Rust instead: the radio's AES block is also used by the link layer, so a
//! software implementation cannot contend with it for a peripheral, and code that produces a connection's
//! access address should be testable in this repository rather than trusted as a blob.
//!
//! Only encryption is implemented, because counter mode only ever encrypts: the ciphertext is XORed with
//! the data on the way out and again on the way in. AES-128 only, which is what the controller sizes its
//! session key for -- a longer key is refused rather than silently truncated.
//!
//! Verified against the FIPS-197 example vector, which is the only way a hand-written AES earns belief.

#![no_std]

/// One AES-128 block, encrypted.
///
/// The state layout is the FIPS-197 one (column-major, byte 0 is the top-left of the state matrix), which
/// is also the byte order of a BLE counter block.
pub fn encrypt_block(key: &[u8; 16], input: &[u8; 16]) -> [u8; 16] {
    let w = expand_key(key);
    let mut state = *input;

    add_round_key(&mut state, &w, 0);
    for round in 1..10 {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, &w, round);
    }
    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &w, 10);
    state
}

/// One AES-CTR step: encrypt `input` with the counter block `counter`, and return the next counter.
///
/// This is the whole "stream" in stream cipher: the caller keeps the counter, so the same function
/// decrypts by being applied to the ciphertext with the same starting counter.
pub fn ctr_step(key: &[u8; 16], counter: &[u8; 16], input: &[u8; 16]) -> ([u8; 16], [u8; 16]) {
    let ks = encrypt_block(key, counter);
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = input[i] ^ ks[i];
    }
    (out, increment_be(*counter))
}

/// The CTR counter advances as a 128-bit big-endian integer.
pub fn increment_be(mut counter: [u8; 16]) -> [u8; 16] {
    for i in (0..16).rev() {
        let (v, carry) = counter[i].overflowing_add(1);
        counter[i] = v;
        if !carry {
            break;
        }
    }
    counter
}

const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

const RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

fn expand_key(key: &[u8; 16]) -> [[u8; 4]; 44] {
    let mut w = [[0u8; 4]; 44];
    for i in 0..4 {
        w[i] = [key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]];
    }
    for i in 4..44 {
        let mut temp = w[i - 1];
        if i % 4 == 0 {
            temp = [
                SBOX[temp[1] as usize] ^ RCON[i / 4 - 1],
                SBOX[temp[2] as usize],
                SBOX[temp[3] as usize],
                SBOX[temp[0] as usize],
            ];
        }
        for j in 0..4 {
            w[i][j] = w[i - 4][j] ^ temp[j];
        }
    }
    w
}

fn add_round_key(state: &mut [u8; 16], w: &[[u8; 4]; 44], round: usize) {
    for c in 0..4 {
        for r in 0..4 {
            state[4 * c + r] ^= w[4 * round + c][r];
        }
    }
}

fn sub_bytes(state: &mut [u8; 16]) {
    for b in state.iter_mut() {
        *b = SBOX[*b as usize];
    }
}

fn shift_rows(state: &mut [u8; 16]) {
    let t = *state;
    for r in 1..4 {
        for c in 0..4 {
            state[4 * c + r] = t[4 * ((c + r) % 4) + r];
        }
    }
}

fn xtime(x: u8) -> u8 {
    (x << 1) ^ (if x & 0x80 != 0 { 0x1b } else { 0 })
}

fn mix_columns(state: &mut [u8; 16]) {
    for c in 0..4 {
        let a = [
            state[4 * c],
            state[4 * c + 1],
            state[4 * c + 2],
            state[4 * c + 3],
        ];
        state[4 * c] = xtime(a[0]) ^ (xtime(a[1]) ^ a[1]) ^ a[2] ^ a[3];
        state[4 * c + 1] = a[0] ^ xtime(a[1]) ^ (xtime(a[2]) ^ a[2]) ^ a[3];
        state[4 * c + 2] = a[0] ^ a[1] ^ xtime(a[2]) ^ (xtime(a[3]) ^ a[3]);
        state[4 * c + 3] = (xtime(a[0]) ^ a[0]) ^ a[1] ^ a[2] ^ xtime(a[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIPS-197, Appendix C.1: the worked example every AES implementation is checked against.
    #[test]
    fn fips197_block() {
        let key: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let plain: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        assert_eq!(
            encrypt_block(&key, &plain),
            [
                0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4,
                0xc5, 0x5a
            ]
        );
    }

    /// A second vector, key and plaintext both zero, which catches key-schedule mistakes that cancel out
    /// in the first one.
    #[test]
    fn zero_key_zero_block() {
        assert_eq!(
            encrypt_block(&[0u8; 16], &[0u8; 16]),
            [
                0x66, 0xe9, 0x4b, 0xd4, 0xef, 0x8a, 0x2c, 0x3b, 0x88, 0x4c, 0xfa, 0x59, 0xca, 0x34,
                0x2b, 0x2e
            ]
        );
    }

    #[test]
    fn ctr_is_reversible_and_advances_by_one() {
        let key = [0x2bu8; 16];
        let iv = [0u8; 16];
        let plain: [u8; 16] = *b"0123456789abcdef";

        let (cipher, next) = ctr_step(&key, &iv, &plain);
        assert_ne!(cipher, plain, "the keystream must change the block");
        assert_eq!(next[15], 1, "the counter advances as a big-endian integer");

        let (back, _) = ctr_step(&key, &iv, &cipher);
        assert_eq!(back, plain, "applying the same step decrypts");
    }

    #[test]
    fn counter_carries_across_bytes() {
        let mut c = [0u8; 16];
        c[15] = 0xff;
        assert_eq!(increment_be(c)[14], 1);
        assert_eq!(increment_be(c)[15], 0);

        // 0xfe ff .. ff + 1 wraps the low bytes to zero and carries into the top byte.
        let mut all = [0xffu8; 16];
        all[0] = 0xfe;
        let mut expected = [0u8; 16];
        expected[0] = 0xff;
        assert_eq!(increment_be(all), expected);
    }
}
