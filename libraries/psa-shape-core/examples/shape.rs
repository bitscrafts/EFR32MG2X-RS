//! The declaration check, asked the questions the stack actually asks.
//!
//! Run with `cargo run -p efr32mg2x-psa-shape-core --example shape`. It builds on the host, which is the whole
//! reason this rule lives in `libraries/` and not next to the C entry point that consumes it.
//!
//! The three lines are the audit's: the stack's own declaration, a caller that declared nothing, and the two
//! contradictions that used to be stored without a word.

use efr32mg2x_psa_shape_core::{shape_agrees, Declared};

fn main() {
    let casos = [
        // (bytes, type, bits, what it means)
        (16, 0x2400, 128, "the stack's own AES key"),
        (16, 0, 0, "a caller that specified nothing"),
        (65, 0x4112, 0, "the peer's public key"),
        (16, 0x1000, 0, "a raw-data key handed 16 bytes"),
        (16, 0x2400, 256, "256 declared, 16 bytes supplied"),
    ];

    for (bytes, key_type, bits, descricao) in casos {
        let declaracao = Declared { key_type, bits };
        let veredicto = match shape_agrees(bytes, declaracao) {
            Ok(()) => "aceite",
            Err(erro) => match erro {
                efr32mg2x_psa_shape_core::ShapeError::WrongType => "RECUSADO (tipo)",
                efr32mg2x_psa_shape_core::ShapeError::WrongSize => "RECUSADO (tamanho)",
            },
        };
        println!("{bytes:>2} bytes  0x{key_type:04x} {bits:>3} bits  {veredicto:<18} {descricao}");
    }
}
