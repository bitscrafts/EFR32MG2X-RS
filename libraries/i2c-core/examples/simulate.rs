//! Run the I2C transaction against a slave model and print what happened on the bus.
//!
//! This is the measurement that does not need a board. `write_read` is the same function the driver calls, over a model
//! that latches its register pointer on a write and loses it when the bus is released -- which is what a real device
//! does, and what the old two-transaction `write_read` in the HAL relied on not happening.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-i2c-core --example simulate`
//!
//! Exits non-zero if the bus sequence or the data is not what the transaction promises, so it can be used as a check
//! rather than only as a demonstration.

use efr32mg2x_i2c_core::sim::simulate_write_read;

fn main() {
    // The IMU on this board answers at 0x6A; register 3 upwards holds 0x13, 0x14, 0x15 in the model.
    let result = simulate_write_read(0x6A, 3, 3);

    println!("trace  : {}", result.trace());
    println!(
        "values : {:02x?}",
        &result.values[..result.count.min(result.values.len())]
    );
    println!("ok     : {}", result.ok);

    let mut failures = 0;

    // One transaction: START, address+write, register, repeated START, address+read, three bytes, STOP.
    if result.trace() != "SWWSWRRRP" {
        println!(
            "FAIL: the bus sequence is `{}`, expected `SWWSWRRRP` (no STOP between the write and the read)",
            result.trace()
        );
        failures += 1;
    }
    if result.values[..3] != [0x13, 0x14, 0x15] {
        println!(
            "FAIL: read {:02x?}, expected [13, 14, 15] -- a wrong pointer reads register 0",
            &result.values[..3]
        );
        failures += 1;
    }
    if !result.ok {
        println!("FAIL: the transaction returned an error");
        failures += 1;
    }

    if failures == 0 {
        println!("PASS: one transaction, repeated START, and the register that was asked for");
    } else {
        std::process::exit(1);
    }
}
