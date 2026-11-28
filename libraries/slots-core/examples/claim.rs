//! Show the claim table refusing a second hand-out.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-slots-core --example claim`
//!
//! This is the half of review finding F10 that can be shown without a board. The other half — that `Board` actually
//! routes every `steal()` through this table — is measured on the part by `hal_one_of_each`.

use efr32mg2x_slots_core::{Slot, Slots, Taken};

fn main() {
    let mut falhas = 0;

    let mut slots = Slots::new();

    // The first ask is the normal path.
    match slots.claim(Slot::Sysrtc) {
        Ok(()) => println!("sysrtc: primeiro pedido aceite"),
        Err(e) => {
            println!("sysrtc: o primeiro pedido foi recusado com {e:?} -- nada funcionaria");
            falhas += 1;
        }
    }

    // The second is the one this exists for.
    match slots.claim(Slot::Sysrtc) {
        Ok(()) => {
            println!("sysrtc: o SEGUNDO pedido foi aceite -- dois donos do mesmo contador");
            falhas += 1;
        }
        Err(Taken(slot)) => println!("sysrtc: segundo pedido recusado, como deve ser ({slot:?})"),
    }

    // And a different peripheral still gets its own slot: the refusal is per slot, not a global stop.
    match slots.claim(Slot::I2c1) {
        Ok(()) => println!("adc: pedido aceite depois da recusa do sysrtc"),
        Err(e) => {
            println!("adc: recusado com {e:?} -- a recusa e' global, o que nao serve");
            falhas += 1;
        }
    }

    // The near-miss that keeps the checks above honest: the slot refused twice is still marked taken.
    if slots.is_taken(Slot::Sysrtc) && !slots.is_taken(Slot::Wdog) {
        println!("estado: sysrtc tomado, wdog livre");
    } else {
        println!("estado: a tabela nao reflecte o que foi pedido");
        falhas += 1;
    }

    if falhas == 0 {
        println!("PASS claim: uma por periferia, e a segunda e' recusada");
    } else {
        println!("FAIL claim: {falhas} desvios");
        std::process::exit(1);
    }
}
