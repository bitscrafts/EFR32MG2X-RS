//! Print every input's `SINGLE` word, and prove the encoding is one-to-one.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-adc-core --example single_word`
//!
//! The columns are the four `SINGLE` fields the hardware reads (`efr32mg24_iadc.h:935-963`): `PORTPOS` at bit 12,
//! `PINPOS` at 8, `PORTNEG` at 4, `PINNEG` at 0. The measured input of this project, PC9, is marked where it appears.

use efr32mg2x_adc_core::{
    bus_for, parse, Bus, Input, Port, PINNEG_SHIFT, PINPOS_SHIFT, PORTNEG_SHIFT, PORTPOS_SHIFT,
};

fn main() {
    let mut failures = 0;

    println!("input   PORTPOS  PINPOS  PORTNEG  PINNEG  word     bus");
    for port in Port::ALL {
        for pin in 0..16u8 {
            let input = Input::new(port, pin).expect("a pin exists");
            let word = input.single_word();
            let bus = match bus_for(input) {
                Bus::MeasuredWithoutAllocation => "measured",
                Bus::Unknown => "unknown",
            };
            // Only the measured input and the ports' first and last pins are printed, to keep this readable.
            if bus == "measured" || pin == 0 || pin == 15 {
                println!(
                    "{port:?}{pin:<5} {:<8} {:<7} {:<8} {:<7} 0x{word:04x} {bus}",
                    (word >> PORTPOS_SHIFT) & 0xF,
                    (word >> PINPOS_SHIFT) & 0xF,
                    (word >> PORTNEG_SHIFT) & 0xF,
                    (word >> PINNEG_SHIFT) & 0xF
                );
            }
        }
    }

    println!("\nround trip and collision check over all 64 inputs:");
    let mut seen = std::vec::Vec::new();
    for port in Port::ALL {
        for pin in 0..16u8 {
            let input = Input::new(port, pin).unwrap();
            let word = input.single_word();
            if parse(word) != Ok(input) {
                println!("FAIL: {port:?}{pin} does not round-trip");
                failures += 1;
            }
            if seen.contains(&word) {
                println!("FAIL: {port:?}{pin} shares a word with an earlier input");
                failures += 1;
            }
            seen.push(word);
        }
    }
    println!("  {} distinct words for {} inputs", seen.len(), 64);

    if seen.len() != 64 {
        println!("FAIL: the encoding collides");
        failures += 1;
    }

    println!("\nthe bus question, stated as it is: 1 of 64 inputs has been measured, and its driver writes no ABUSALLOC");
    if bus_for(Input::new(Port::C, 9).unwrap()) != Bus::MeasuredWithoutAllocation {
        println!("FAIL: PC9 is the input this project has measured");
        failures += 1;
    }

    if failures == 0 {
        println!("\nPASS: 64 inputs, 64 distinct words, every one of them parseable back");
    } else {
        std::process::exit(1);
    }
}
