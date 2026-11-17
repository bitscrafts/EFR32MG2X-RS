//! Print the divider for every clock this project has measured, and the rate the hardware will actually produce.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-usart-core --example divider`
//!
//! The second column is what the old HAL code computed with `u32` arithmetic, where `256 * 19 000 000` wraps. The two
//! disagreeing is the measurement; the printed baud rate is what a console would see.

use efr32mg2x_usart_core::{actual_baud, clkdiv};

fn main() {
    let mut failures = 0;
    println!("pclk        baud     clkdiv   actual   error (ppm)   old(u32 wrap)");
    for pclk in [19_000_000u32, 38_000_000, 56_000_000, 80_000_000] {
        for baud in [9_600u32, 115_200, 921_600] {
            match clkdiv(pclk, baud, 16) {
                Ok(div) => {
                    let actual = actual_baud(pclk, div, 16);
                    let ppm =
                        ((actual as i64 - baud as i64).unsigned_abs() * 1_000_000) / baud as u64;
                    // What the previous expression produced, in wrapping u32 arithmetic.
                    let wrapped = (256u32.wrapping_mul(pclk) / (16 * baud)).wrapping_sub(256);
                    println!("{pclk:<11} {baud:<8} {div:<8} {actual:<8} {ppm:<13} {wrapped}");
                    if ppm >= 20_000 {
                        println!("FAIL: {ppm} ppm off at {pclk} Hz / {baud} baud is more than a UART tolerates");
                        failures += 1;
                    }
                }
                Err(e) => println!("{pclk:<11} {baud:<8} refused: {e:?}"),
            }
        }
    }

    // The defect, stated as a number: the old expression and the correct one differ wherever the product wraps.
    let correct = clkdiv(19_000_000, 115_200, 16).unwrap();
    let wrapped = (256u32.wrapping_mul(19_000_000) / (16 * 115_200)).wrapping_sub(256);
    println!("\nat 19 MHz / 115200: correct {correct}, old expression {wrapped}");
    if correct == wrapped {
        println!("FAIL: the defect is not reproduced -- check the arithmetic");
        failures += 1;
    }

    if failures == 0 {
        println!(
            "PASS: every divider fits, no multiplication wraps, and the rates are within tolerance"
        );
    } else {
        std::process::exit(1);
    }
}
