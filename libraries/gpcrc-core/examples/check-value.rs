//! Print both CRC-32 conventions and refuse to say PASS if either is wrong.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-gpcrc-core --example check-value`
//!
//! Two jobs. It pins the arithmetic to the published check values, which is the half that runs anywhere; and it prints the
//! raw reflected value (`0x340BC6D9`) that the part returns, so the number in the board's record has a reference that was
//! computed somewhere else.

use efr32mg2x_gpcrc_core::{crc32, crc32_msb, reflected_via_msb, CRCPoly, INIT};

fn main() {
    let mut failures = 0;

    // The reflected convention, which is what every other tool means by CRC-32.
    let zlib = crc32(b"123456789");
    println!("reflected CRC-32 of \"123456789\" = {zlib:08x} (published: cbf43926)");
    if zlib != 0xCBF4_3926 {
        println!("  the reflected convention does not match its published vector");
        failures += 1;
    }

    // What the accelerator returns: the same engine, one step short of the convention.
    let raw = !zlib;
    println!("what GPCRC DATA holds before the caller's xor = {raw:08x} (measured on the part: 340bc6d9)");
    if raw != 0x340B_C6D9 {
        println!("  this does not match the value measured on 2026-11-25; one of the two is wrong");
        failures += 1;
    }

    // The MSB-first convention, kept because the reference manual describes the engine that way and it is the one to reach
    // for if a protocol needs it.
    let msb = crc32_msb(b"123456789", INIT);
    println!("MSB-first CRC-32 of \"123456789\" = {msb:08x} (published MPEG-2: 0376e6e7)");
    if msb != 0x0376_E6E7 {
        println!("  the MSB-first convention does not match its published vector");
        failures += 1;
    }

    // The relation between the two, computed rather than asserted.
    println!(
        "relation: reflected via the MSB-first engine = {:08x} (must equal {zlib:08x})",
        reflected_via_msb(b"123456789")
    );
    if reflected_via_msb(b"123456789") != zlib {
        println!("  the two conventions are not bit reversals of each other as documented");
        failures += 1;
    }

    println!(
        "polynomials: crc32={:08x} crc16={:08x}",
        CRCPoly::Crc32.value(),
        CRCPoly::Crc16.value()
    );
    if CRCPoly::Crc32.value() != 0x04C1_1DB7 || CRCPoly::Crc16.value() != 0x1021 {
        println!("  a polynomial is not the one the manual names");
        failures += 1;
    }

    if failures == 0 {
        println!("PASS gpcrc-core: 5 checks");
    } else {
        println!("FAIL gpcrc-core: {failures} check(s)");
        std::process::exit(1);
    }
}
