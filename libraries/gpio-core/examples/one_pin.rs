//! Exhaustive demonstration: one pin at a time, across every pin and every model value.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-gpio-core --example one_pin`
//!
//! Two things are printed. First the register split that the exhaustive test discovered -- pins 0..7 live in `MODEL`,
//! pins 8..15 in `MODELH`, four bits each -- and then the property that H17 broke: setting one pin must leave the other
//! fifteen exactly as they were, and the program checks all 16 × 16 combinations rather than one call site.

use efr32mg2x_gpio_core::{
    dout_of, field_of, model, route, with_dout, Models, Port, PINS_PER_PORT,
};

fn main() {
    let mut failures = 0;

    println!("pin  register  shift   route (port A)");
    for pin in 0..PINS_PER_PORT {
        let (register, shift) = field_of(pin).expect("a pin exists");
        println!(
            "{:<4} {:<9?} {:<7} 0x{:02x}",
            pin,
            register,
            shift,
            route(Port::A, pin).unwrap()
        );
    }

    println!("\nproperty: setting one pin leaves the other fifteen alone");
    let background = Models::new(0xAAAA_AAAA, 0xAAAA_AAAA);
    let mut checked = 0;
    for pin in 0..PINS_PER_PORT {
        for value in 0..0x10u8 {
            let after = background.with(pin, value).expect("a valid pin and model");
            for other in 0..PINS_PER_PORT {
                let expected = if other == pin {
                    value
                } else {
                    background.model_of(other).unwrap()
                };
                if after.model_of(other).unwrap() != expected {
                    println!("FAIL: pin {pin} = {value} changed pin {other}");
                    failures += 1;
                }
                checked += 1;
            }
        }
    }
    println!("  {checked} field comparisons, no pin moved");

    println!("\nDOUT is one bit per pin");
    let dout = 0xFFFF_0000u32;
    for pin in 0..PINS_PER_PORT {
        let high = with_dout(dout, pin, true).unwrap();
        let low = with_dout(dout, pin, false).unwrap();
        for other in 0..PINS_PER_PORT {
            let before = dout_of(dout, other).unwrap();
            let unchanged = if other == pin {
                dout_of(high, other).unwrap() && !dout_of(low, other).unwrap()
            } else {
                dout_of(high, other).unwrap() == before && dout_of(low, other).unwrap() == before
            };
            if !unchanged {
                println!("FAIL: pin {pin} disturbed pin {other}");
                failures += 1;
            }
        }
    }
    println!("  every pin drives only its own bit");

    println!(
        "\nmodel values in use: disabled {}, push-pull {}, wired-and {}",
        model::DISABLED,
        model::PUSHPULL,
        model::WIREDAND
    );

    if failures == 0 {
        println!("\nPASS: 256 model writes and 32 level changes, none of them touched a pin they did not name");
    } else {
        std::process::exit(1);
    }
}
