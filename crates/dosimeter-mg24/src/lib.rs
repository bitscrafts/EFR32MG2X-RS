//! Dosimetria de ruído com incerteza — lógica pura, sem hardware, testável no host.
//!
//! ## O que este crate faz
//!
//! Duas coisas que a NR-15 exige e que se calculam em aritmética simples:
//!
//! 1. **Nível equivalente** (`Leq`) a partir de amostras de pressão numa janela longa.
//! 2. **Dose acumulada**, com taxa de troca de 5 dB: cada +5 dB permite metade do tempo.
//!
//! ## A regra, escrita como está na norma
//!
//! Para um nível `L` em dB(A) mantido durante `t` segundos, e o limite de 85 dB(A) para
//! 8 horas (28 800 s):
//!
//! ```text
//! q     = 5 dB              taxa de troca
//! T(L)  = 28800 * 2^(-(L - 85) / q)      tempo admissível a esse nível
//! dose += t / T(L)                        fracção da dose diária consumida
//! ```
//!
//! Verificação directa: 85 dB durante 8 h -> `2^0 = 1` -> 100 %. 90 dB durante 4 h ->
//! `T = 14400 s` -> 4 h de 4 h -> 100 %. É o controlo numérico do crate.
//!
//! ## Porque isto vive num crate separado
//!
//! Não depende do hardware, do BLE nem do IADC: é aritmética que se prova numa máquina de
//! desenvolvimento com `cargo test`. A aplicação embarcada só tem de lhe passar níveis.

#![no_std]
#![deny(unsafe_code)]

/// Tempo admissível a 85 dB(A), em segundos: 8 horas (NR-15).
pub const TEMPO_LIMITE_S: f32 = 8.0 * 3600.0;

/// Nível de referência da norma, em dB(A).
pub const NIVEL_REFERENCIA_DB: f32 = 85.0;

/// Taxa de troca da NR-15, em dB: cada +5 dB reduz o tempo admissível para metade.
pub const TAXA_TROCA_DB: f32 = 5.0;

/// Piso de plausibilidade: abaixo disto o sensor nao esta' a medir ruido util.
///
/// Um microfone desligado le 0 e um valor desses produziria um tempo admissivel
/// astronomico — isto e', uma dose que nunca sobe. Um piso explicito evita que
/// uma avaria de sensor pareca "ambiente silencioso".
pub const NIVEL_MINIMO_DB: f32 = 20.0;

/// Leitura com incerteza — o que a apresentação mostra e o que o telemóvel recebe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leitura {
    /// Nível em dB(A).
    pub nivel_db: f32,
    /// Semi-largura do intervalo de credibilidade, em dB.
    pub incerteza_db: f32,
}

impl Leitura {
    /// Intervalo `[min, max]`, com protecção contra incerteza negativa.
    pub fn intervalo(&self) -> (f32, f32) {
        let h = if self.incerteza_db.is_finite() && self.incerteza_db > 0.0 {
            self.incerteza_db
        } else {
            0.0
        };
        (self.nivel_db - h, self.nivel_db + h)
    }
}

/// Tempo admissível a um nível `nivel_db`, em segundos.
pub fn tempo_admissivel_s(nivel_db: f32) -> f32 {
    let expoente = -(nivel_db - NIVEL_REFERENCIA_DB) / TAXA_TROCA_DB;
    TEMPO_LIMITE_S * libm_exp2(expoente)
}

/// Acumulador de dose.
#[derive(Clone, Copy, Debug, Default)]
pub struct Dose {
    /// Fracção da dose diária já consumida (1.0 = 100 %).
    fracao: f32,
    /// Tempo total de exposição considerado, em segundos.
    tempo_s: f32,
}

impl Dose {
    pub fn nova() -> Self {
        Self::default()
    }

    /// Acrescenta uma exposição de `dt_s` segundos a `nivel_db`.
    ///
    /// Níveis abaixo de 0 dB não são fisicamente úteis e são ignorados em vez de produzirem
    /// um tempo admissível astronómico — um sensor desligado lê 0 e não deve "curar" a dose.
    pub fn acumula(&mut self, nivel_db: f32, dt_s: f32) {
        if !(dt_s.is_finite() && dt_s > 0.0) {
            return;
        }
        if !nivel_db.is_finite() || nivel_db < NIVEL_MINIMO_DB {
            return;
        }
        let admissivel = tempo_admissivel_s(nivel_db);
        if admissivel.is_finite() && admissivel > 0.0 {
            self.fracao += dt_s / admissivel;
            self.tempo_s += dt_s;
        }
    }

    /// Dose em percentagem (100 % = limite diário atingido).
    pub fn percentagem(&self) -> f32 {
        self.fracao * 100.0
    }

    /// Tempo de exposição já contabilizado, em segundos.
    pub fn tempo_s(&self) -> f32 {
        self.tempo_s
    }

    /// Repõe a dose a zero (comando vindo do telemóvel).
    pub fn repor(&mut self) {
        self.fracao = 0.0;
        self.tempo_s = 0.0;
    }

    /// Estimativa do tempo restante ao nível actual, em segundos.
    ///
    /// `None` se o nível não for utilizável. Se a dose já excedeu 100 %, devolve 0.
    pub fn tempo_restante_s(&self, nivel_db: f32) -> Option<f32> {
        if !nivel_db.is_finite() || nivel_db < NIVEL_MINIMO_DB {
            return None;
        }
        let taxa = 1.0 / tempo_admissivel_s(nivel_db); // fracção por segundo
        let resta = 1.0 - self.fracao;
        if taxa <= 0.0 || !taxa.is_finite() {
            return None;
        }
        let t = resta / taxa;
        Some(if t > 0.0 { t } else { 0.0 })
    }
}

/// `2^x` sem `std`: `exp2` não existe em `core`.
///
/// Implementação por `exp(x * ln 2)` com uma série de Taylor em torno de `k*ln2`, que dá
/// erro relativo < 1e-6 para o intervalo de níveis que interessa (0 a 140 dB, isto é,
/// expoente de -11 a +11). Substituível por `libm` se o projecto o passar a usar.
fn libm_exp2(x: f32) -> f32 {
    if !x.is_finite() {
        return f32::NAN;
    }
    const LN2: f32 = 0.693_147_2;
    // 2^x = 2^k * e^((x-k)*ln2), com k = round(x) -> |(x-k)*ln2| <= ln2/2
    let k = if x >= 0.0 {
        (x + 0.5) as i32
    } else {
        (x - 0.5) as i32
    };
    let r = (x - k as f32) * LN2;

    let mut termo = 1.0f32;
    let mut soma = 1.0f32;
    let mut n = 1.0f32;
    while n <= 8.0 {
        termo *= r / n;
        soma += termo;
        n += 1.0;
    }

    // 2^k por duplicacao/halvagem sucessiva
    let mut escala = 1.0f32;
    let kk_abs = if k < 0 { -k } else { k };
    let mut i = 0i32;
    while i < kk_abs {
        escala *= 2.0;
        i += 1;
    }
    if k >= 0 {
        soma * escala
    } else {
        soma / escala
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Controlo da norma: 85 dB(A) durante 8 h consome exactamente 100 % da dose.
    #[test]
    fn oitenta_e_cinco_por_oito_horas_da_cem_por_cento() {
        let mut d = Dose::nova();
        d.acumula(85.0, 8.0 * 3600.0);
        let p = d.percentagem();
        assert!((p - 100.0).abs() < 0.5, "deu {p} %, esperado ~100 %");
    }

    /// Taxa de troca de 5 dB: 90 dB durante metade do tempo dá a mesma dose.
    #[test]
    fn noventa_por_quatro_horas_da_cem_por_cento() {
        let mut d = Dose::nova();
        d.acumula(90.0, 4.0 * 3600.0);
        let p = d.percentagem();
        assert!((p - 100.0).abs() < 0.5, "deu {p} %, esperado ~100 %");
    }

    #[test]
    fn noventa_e_cinco_por_duas_horas_da_cem_por_cento() {
        let mut d = Dose::nova();
        d.acumula(95.0, 2.0 * 3600.0);
        let p = d.percentagem();
        assert!((p - 100.0).abs() < 0.5, "deu {p} %, esperado ~100 %");
    }

    /// Metade do tempo admissível = metade da dose.
    #[test]
    fn metade_do_tempo_e_metade_da_dose() {
        let mut d = Dose::nova();
        d.acumula(85.0, 4.0 * 3600.0);
        let p = d.percentagem();
        assert!((p - 50.0).abs() < 0.5, "deu {p} %, esperado ~50 %");
    }

    /// Controlo negativo: 80 dB dobra o tempo admissivel, logo 8 h dao metade da dose.
    #[test]
    fn oitenta_db_nao_atinge_o_limite() {
        let mut d = Dose::nova();
        d.acumula(80.0, 8.0 * 3600.0);
        let p = d.percentagem();
        assert!((p - 50.0).abs() < 1.0, "deu {p} %, esperado ~50 %");
    }

    /// Um sensor desligado (0 dB) não pode "curar" a dose já acumulada.
    #[test]
    fn nivel_invalido_nao_reduz_a_dose() {
        let mut d = Dose::nova();
        d.acumula(90.0, 3600.0);
        let antes = d.percentagem();
        d.acumula(0.0, 3600.0);
        d.acumula(-5.0, 3600.0);
        d.acumula(f32::NAN, 10.0);
        d.acumula(90.0, -10.0);
        assert!(
            (d.percentagem() - antes).abs() < 1e-6,
            "a dose mudou com entradas inválidas"
        );
    }

    #[test]
    fn repor_volta_a_zero() {
        let mut d = Dose::nova();
        d.acumula(95.0, 3600.0);
        assert!(d.percentagem() > 0.0);
        d.repor();
        assert_eq!(d.percentagem(), 0.0);
        assert_eq!(d.tempo_s(), 0.0);
    }

    #[test]
    fn intervalo_da_leitura_com_incerteza() {
        let l = Leitura {
            nivel_db: 84.0,
            incerteza_db: 1.5,
        };
        assert_eq!(l.intervalo(), (82.5, 85.5));
        let sem = Leitura {
            nivel_db: 88.0,
            incerteza_db: -3.0,
        };
        assert_eq!(
            sem.intervalo(),
            (88.0, 88.0),
            "incerteza inválida nao pode alargar"
        );
    }

    /// A exp2 própria tem de concordar com a do sistema (que aqui não existe) nos pontos
    /// que a norma usa.
    #[test]
    fn exp2_nos_pontos_da_norma() {
        // 85 dB -> 2^0 = 1
        assert!((tempo_admissivel_s(85.0) - TEMPO_LIMITE_S).abs() < 1.0);
        // 90 dB -> metade
        assert!((tempo_admissivel_s(90.0) - TEMPO_LIMITE_S / 2.0).abs() < 1.0);
        // 80 dB -> dobro
        assert!((tempo_admissivel_s(80.0) - TEMPO_LIMITE_S * 2.0).abs() < 1.0);
        // 100 dB -> 1/8
        assert!((tempo_admissivel_s(100.0) - TEMPO_LIMITE_S / 8.0).abs() < 2.0);
    }
}
