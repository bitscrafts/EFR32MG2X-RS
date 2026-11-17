//! Show why `flush` on `TXFL` returns before the bytes have left, and what `TXIDLE` does instead.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-spi-core --example flush`
//!
//! The two flags are not synonyms: `TXFL` reads 1 when the FIFO has **room** (its reset value is 1), and `TXIDLE` reads 1
//! when there is nothing left to send. A driver that waits on `TXFL` therefore reports completion at the moment its own
//! buffer was filled.

use efr32mg2x_spi_core::{cfg2_with_sdiv, flush_on_txfl, flush_on_txidle, sdiv, TxPath};

fn main() {
    let mut failures = 0;

    println!("=== the divider, encoded where the synchronous path reads it");
    for (pclk, bitrate) in [
        (19_000_000u32, 1_000_000u32),
        (19_000_000, 4_000_000),
        (80_000_000, 8_000_000),
    ] {
        match sdiv(pclk, bitrate) {
            Ok(value) => {
                let cfg2 = cfg2_with_sdiv(0, value);
                println!(
                    "pclk {pclk:<10} bitrate {bitrate:<9} SDIV {value:<4} CFG2 0x{cfg2:08x} (field at bit 24)"
                );
            }
            Err(e) => {
                println!("pclk {pclk} bitrate {bitrate} refused: {e:?}");
                failures += 1;
            }
        }
    }

    println!("\n=== flush: TXFL against TXIDLE");
    let mut path = TxPath::new();
    path.push(0xAA);
    path.push(0xBB);
    println!(
        "two bytes queued: txfl={} txidle={}",
        path.txfl(),
        path.txidle()
    );

    let by_txfl = flush_on_txfl(&mut path, 3);
    println!(
        "flush on TXFL returned after {by_txfl} polls; txidle is now {}",
        path.txidle()
    );
    if path.txidle() {
        println!("FAIL: a flush on TXFL should not have waited for the shifter");
        failures += 1;
    }

    match flush_on_txidle(&mut path, 10) {
        Ok(polls) => {
            println!(
                "flush on TXIDLE returned after {polls} polls; txidle is now {}",
                path.txidle()
            );
            if !path.txidle() {
                println!("FAIL: the shifter should be idle");
                failures += 1;
            }
        }
        Err(e) => {
            println!("FAIL: the shifter never drained: {e:?}");
            failures += 1;
        }
    }

    if failures == 0 {
        println!("\nPASS: SDIV lands at bit 24, and only TXIDLE means the wire is quiet");
    } else {
        std::process::exit(1);
    }
}
