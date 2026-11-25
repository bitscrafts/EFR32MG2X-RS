//! Show a record being written, and refuse to print a verdict if it came out wrong.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-record-core --example format`
//!
//! This is the half of the formatter that can be shown without a board: the arithmetic of decimal, negative and hex
//! digits, and what a buffer too small to hold the number does. The other half — that the instruments on the part print
//! the same bytes they printed before the formatter was shared — was measured by flashing four of them and comparing
//! their records.

use efr32mg2x_record_core::Record;

fn main() {
    let mut failures = 0;

    // What an instrument's line looks like: a name, a few fields, and a verdict.
    let mut line = Record::<96>::new();
    line.s(b"adc: gnd=")
        .u(0u16)
        .s(b" ch0=")
        .u(0u16)
        .s(b" ref=")
        .s(b"VBGR");
    let text = core::str::from_utf8(line.bytes()).unwrap_or("<not ASCII>");
    println!("record: {text}");
    if text != "adc: gnd=0 ch0=0 ref=VBGR" {
        println!("  the line came out differently -- every instrument's evidence would shift");
        failures += 1;
    }
    if line.truncated() {
        println!("  the buffer was too small and said so, but 96 bytes should have held this");
        failures += 1;
    }

    // The near-miss: the same record in a buffer one byte too small.
    let mut tight = Record::<20>::new();
    tight.s(b"adc: gnd=0 ch0=0 ref=VBGR");
    println!(
        "tight: {} bytes, truncated={}",
        tight.len(),
        tight.truncated()
    );
    if !tight.truncated() {
        println!("  a record cut at 20 bytes was not flagged -- a cut field reads as a zero field");
        failures += 1;
    }

    // Both ends of the number formats, and the one value that panics a naive implementation.
    let mut numbers = Record::<48>::new();
    numbers.u(u64::MAX).s(b" ").i(i64::MIN).s(b" ").h(0x0a);
    let text = core::str::from_utf8(numbers.bytes()).unwrap_or("<not ASCII>");
    println!("numbers: {text}");
    if text != "18446744073709551615 -9223372036854775808 0a" {
        println!("  a digit was lost or a sign was dropped");
        failures += 1;
    }

    if failures == 0 {
        println!("PASS record: 4 checks");
    } else {
        println!("FAIL record: {failures} check(s)");
        std::process::exit(1);
    }
}
