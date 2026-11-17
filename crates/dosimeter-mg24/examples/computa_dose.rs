//! Exemplo de uso da API do dosímetro — compilado, não executado.
//!
//! ## Porque este exemplo é uma biblioteca e não um binário
//!
//! O alvo deste repositório é `thumbv8m.main-none-eabihf`, com `-Tlink.x` do `cortex-m-rt`.
//! Um binário nesse alvo precisa de um `#[entry]` e de uma tabela de vectores fornecidos pelo
//! `cortex-m-rt` — e este crate é lógica pura, deliberadamente sem hardware, sem HAL e sem
//! PAC: não pode depender do `cortex-m-rt` só para ter um `main`.
//!
//! A consequência prática de o exemplo ser `crate-type = ["lib"]` é que ele **compila para o
//! alvo real** e prova que a API é usável fora dos testes, sem inventar um runtime que o
//! crate não tem. Quem quiser ver números corre
//! `cargo test -p dosimeter-mg24 --target x86_64-unknown-linux-gnu`.
//!
//! ## O que este ficheiro demonstra
//!
//! As três operações que a aplicação embarcada vai chamar: dose a partir de um nível, `Leq` a
//! partir de exposições, e a percentagem que resulta de comparar o `Leq` com a tabela.

#![no_std]

use dosimeter_mg24::{dose_percentual, leq, tempo_admissivel_s, Dose, Exposicao};

/// Uma medição de oito horas a 85 dB(A) seguida de uma hora a 95 dB(A).
///
/// Devolve `(dose_acumulada_pct, leq_db, tempo_admissivel_a_95_s)`.
pub fn dia_de_trabalho() -> (f32, Option<f32>, f32) {
    let mut dose = Dose::nova();
    dose.acumula(85.0, 8.0 * 3600.0);
    dose.acumula(95.0, 3600.0);

    let exposicoes = [
        Exposicao {
            nivel_db: 85.0,
            duracao_s: 8.0 * 3600.0,
        },
        Exposicao {
            nivel_db: 95.0,
            duracao_s: 3600.0,
        },
    ];

    (
        dose.percentagem(),
        leq(&exposicoes),
        tempo_admissivel_s(95.0),
    )
}

/// A mesma exposição, mas avaliada pelo `Leq` em vez de acumulada por nível.
///
/// É a diferença que o relatório discute: acumular por nível é mais restritivo do que
/// calcular um nível equivalente e compará-lo com a tabela, porque a tabela não é linear.
pub fn dia_de_trabalho_por_leq() -> Option<f32> {
    let exposicoes = [
        Exposicao {
            nivel_db: 85.0,
            duracao_s: 8.0 * 3600.0,
        },
        Exposicao {
            nivel_db: 95.0,
            duracao_s: 3600.0,
        },
    ];
    let equivalente = leq(&exposicoes)?;
    dose_percentual(equivalente, 9.0 * 3600.0)
}
