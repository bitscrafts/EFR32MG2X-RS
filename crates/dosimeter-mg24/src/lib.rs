//! Dosimetria de ruído com incerteza — lógica pura, sem hardware, testável no host.
//!
//! ## O que este crate faz
//!
//! Duas coisas que a NR-15 exige e que se calculam em aritmética simples:
//!
//! 1. **Nível equivalente** (`Leq`) a partir de amostras de pressão numa janela longa.
//! 2. **Dose acumulada**, com taxa de troca de 5 dB: cada +5 dB permite metade do tempo.
//!
//! ## A regra, como está na norma — e não como a fórmula a aproxima
//!
//! A NR-15 (Portaria 3.214/78, Anexo 1) dá o **limite de exposição diária por nível numa
//! tabela**, e a regra entre duas entradas é arredondar para o **nível imediatamente
//! superior** — não interpolar, e muito menos extrapolar uma curva.
//!
//! ```text
//! nível (dB(A))   máxima exposição diária
//! 85               8 h           90               4 h
//! 86               7 h           95               2 h 15 min
//! 87               6 h          100               1 h
//! 88               5 h          105              20 min
//! 89               4 h 30 min   110               3 min
//! ```
//!
//! A tabela completa, com as 31 entradas (85 a 115 dB(A)), está em [`TABELA_NR15`].
//!
//! **Porque isto importa, medido:** a curva contínua `T(L) = 28800 · 2^(-(L-85)/5)` que este
//! crate usava antes *subestima* a exposição até 17,5 % — a 97 dB a norma permite 1 h 45 min
//! (6 300 s) e a curva dava 5 457 s. Num dosímetro, subestimar a exposição é a única falha
//! que o aparelho existe para evitar.
//!
//! Abaixo de 85 dB(A) a norma não fixa limite: o tempo admissível é infinito e a dose
//! regulamentada não cresce. Acima de 115 dB(A) usa-se a última entrada (15 s) — conservador,
//! e declarado em vez de extrapolado.
//!
//! ## O nível equivalente (`Leq`)
//!
//! `Leq = 10·log10( (1/T) · Σ ti · 10^(Li/10) )` sobre pares (nível, duração), em dB. É a
//! grandeza informativa: diz qual o nível constante que daria a mesma energia. Não substitui
//! a dose — completa-a, porque a dose só conta o que a norma regula.
//!
//! ## Porque isto vive num crate separado
//!
//! Não depende do hardware, do BLE nem do IADC: é aritmética que se prova numa máquina de
//! desenvolvimento com `cargo test`. A aplicação embarcada só tem de lhe passar níveis.

#![no_std]
#![deny(unsafe_code)]

#[cfg(test)]
extern crate alloc;

pub mod features;
pub mod gmm;
pub mod regress;

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

/// Tabela do Anexo 1 da NR-15: (nível em dB(A), máxima exposição diária em segundos).
///
/// **Texto da norma:** «Limites de tolerância para ruído contínuo ou intermitente» — a
/// coluna de máxima exposição diária permissível, para 85 a 115 dB(A).
///
/// Entre duas entradas aplica-se a **regra do nível imediatamente superior**: um nível de
/// 89,5 dB(A) usa a linha dos 90 dB(A), e não uma interpolação entre 89 e 90.
pub const TABELA_NR15: [(f32, f32); 31] = [
    (85.0, 28_800.0), // 8 h
    (86.0, 25_200.0), // 7 h
    (87.0, 21_600.0), // 6 h
    (88.0, 18_000.0), // 5 h
    (89.0, 16_200.0), // 4 h 30 min
    (90.0, 14_400.0), // 4 h
    (91.0, 12_600.0), // 3 h 30 min
    (92.0, 10_800.0), // 3 h
    (93.0, 9_600.0),  // 2 h 40 min
    (94.0, 9_000.0),  // 2 h 30 min
    (95.0, 8_100.0),  // 2 h 15 min
    (96.0, 7_200.0),  // 2 h
    (97.0, 6_300.0),  // 1 h 45 min
    (98.0, 5_400.0),  // 1 h 30 min
    (99.0, 4_500.0),  // 1 h 15 min
    (100.0, 3_600.0), // 1 h
    (101.0, 2_700.0), // 45 min
    (102.0, 2_100.0), // 35 min
    (103.0, 1_800.0), // 30 min
    (104.0, 1_500.0), // 25 min
    (105.0, 1_200.0), // 20 min
    (106.0, 900.0),   // 15 min
    (107.0, 600.0),   // 10 min
    (108.0, 420.0),   // 7 min
    (109.0, 300.0),   // 5 min
    (110.0, 180.0),   // 3 min
    (111.0, 120.0),   // 2 min
    (112.0, 60.0),    // 1 min
    (113.0, 30.0),    // 30 s
    (114.0, 20.0),    // 20 s
    (115.0, 15.0),    // 15 s
];

/// Tempo admissível a um nível `nivel_db`, em segundos, pela **tabela** da NR-15.
///
/// Regra do nível imediatamente superior: um nível fraccionário sobe para a entrada
/// seguinte. Um nível exactamente numa entrada usa essa entrada, sem arredondar.
///
/// Devolve `f32::INFINITY` abaixo de 85 dB(A) — a norma não fixa limite nessa faixa — e o
/// valor da última entrada acima de 115 dB(A).
pub fn tempo_admissivel_s(nivel_db: f32) -> f32 {
    if !nivel_db.is_finite() {
        return 0.0;
    }
    let primeira = TABELA_NR15[0].0;
    let ultima = TABELA_NR15[TABELA_NR15.len() - 1].0;
    if nivel_db < primeira {
        return f32::INFINITY;
    }
    if nivel_db >= ultima {
        return TABELA_NR15[TABELA_NR15.len() - 1].1;
    }
    // Regra do nível imediatamente superior: fracção -> entrada seguinte.
    let arredondado = if nivel_db == libm_floor(nivel_db) {
        nivel_db
    } else {
        libm_floor(nivel_db) + 1.0
    };
    for (nivel, admissivel) in TABELA_NR15 {
        if (nivel - arredondado).abs() < 0.5 {
            return admissivel;
        }
    }
    TABELA_NR15[TABELA_NR15.len() - 1].1
}

/// Exposição a um nível durante uma duração — a entrada do cálculo de `Leq`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Exposicao {
    /// Nível em dB(A).
    pub nivel_db: f32,
    /// Duração, em segundos.
    pub duracao_s: f32,
}

/// Nível equivalente (`Leq`) de um conjunto de exposições, em dB(A).
///
/// `Leq = 10·log10( (1/T) · Σ ti · 10^(Li/10) )`, com `T = Σ ti`.
///
/// **Devolve `None` quando não há medição** — lista vazia, durações não positivas ou nível
/// não finito. Zero dB é indistinguível de um silêncio realmente medido, e devolver 0 para
/// «sem dados» é exactamente o número que este projecto passou a semana a remover.
pub fn leq(exposicoes: &[Exposicao]) -> Option<f32> {
    let mut energia = 0.0f32; // Σ ti · 10^(Li/10)
    let mut total = 0.0f32;
    for e in exposicoes {
        if !e.nivel_db.is_finite() || (!e.duracao_s.is_finite() || e.duracao_s <= 0.0) {
            continue;
        }
        // 10^(L/10) = exp((L/10)·ln 10)
        let potencia = gmm::exp_aprox(e.nivel_db / 10.0 * LN_10);
        energia += e.duracao_s * potencia;
        total += e.duracao_s;
    }
    if (!total.is_finite() || total <= 0.0) || (!energia.is_finite() || energia <= 0.0) {
        return None;
    }
    // log10(x) = ln(x)/ln 10
    Some(10.0 * (gmm::ln_aprox(energia / total) / LN_10))
}

/// Percentagem da dose diária que uma exposição `tempo_medido_s` a `leq_db` representa.
///
/// `dose % = 100 · T_medido / T_admissível(Leq)`, com `T_admissível` vindo da tabela.
/// `None` se o tempo não for positivo ou se a norma não fixar limite (abaixo de 85 dB(A)).
pub fn dose_percentual(leq_db: f32, tempo_medido_s: f32) -> Option<f32> {
    if !leq_db.is_finite() || (!tempo_medido_s.is_finite() || tempo_medido_s <= 0.0) {
        return None;
    }
    let admissivel = tempo_admissivel_s(leq_db);
    if !admissivel.is_finite() || admissivel <= 0.0 {
        return None;
    }
    Some(100.0 * tempo_medido_s / admissivel)
}

/// `ln 10`, para as conversões de `Leq`.
const LN_10: f32 = core::f32::consts::LN_10;

/// `floor` sem `std`: `f32::floor` não existe em `core`.
fn libm_floor(x: f32) -> f32 {
    let t = x as i32 as f32;
    if x < 0.0 && t != x {
        t - 1.0
    } else {
        t
    }
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
        if !dt_s.is_finite() || dt_s <= 0.0 {
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
#[cfg(test)]
fn libm_exp2(x: f32) -> f32 {
    if !x.is_finite() {
        return f32::NAN;
    }
    const LN2: f32 = core::f32::consts::LN_2;
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

    /// A tabela dá 95 dB -> 2 h 15 min. Duas horas são 88,9 % da dose, não 100 %.
    #[test]
    fn noventa_e_cinco_por_duas_horas_e_quinze_da_cem_por_cento() {
        let mut d = Dose::nova();
        d.acumula(95.0, 2.25 * 3600.0);
        let p = d.percentagem();
        assert!((p - 100.0).abs() < 0.5, "deu {p} %, esperado ~100 %");

        let mut parcial = Dose::nova();
        parcial.acumula(95.0, 2.0 * 3600.0);
        let pp = parcial.percentagem();
        assert!(
            (pp - 88.888).abs() < 0.5,
            "deu {pp} %, esperado ~88,9 % (2 h de 2 h 15)"
        );
    }

    /// Metade do tempo admissível = metade da dose.
    #[test]
    fn metade_do_tempo_e_metade_da_dose() {
        let mut d = Dose::nova();
        d.acumula(85.0, 4.0 * 3600.0);
        let p = d.percentagem();
        assert!((p - 50.0).abs() < 0.5, "deu {p} %, esperado ~50 %");
    }

    /// Abaixo de 85 dB(A) a norma não fixa limite: o tempo admissível é infinito e a dose
    /// regulamentada não cresce. Não é o mesmo que "silêncio" — é "fora da tabela".
    #[test]
    fn abaixo_de_oitenta_e_cinco_nao_consome_dose_regulamentada() {
        assert!(tempo_admissivel_s(80.0).is_infinite());
        let mut d = Dose::nova();
        d.acumula(80.0, 8.0 * 3600.0);
        assert_eq!(
            d.percentagem(),
            0.0,
            "80 dB não pode consumir dose regulamentada"
        );
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

    /// FRONTEIRA, lado 1: um nível exactamente numa entrada usa essa entrada, sem subir.
    #[test]
    fn nivel_exactamente_numa_entrada_nao_arredonda() {
        assert_eq!(tempo_admissivel_s(90.0), 14_400.0); // 4 h, e não 3 h 30 min
        assert_eq!(tempo_admissivel_s(95.0), 8_100.0); // 2 h 15 min
        assert_eq!(tempo_admissivel_s(100.0), 3_600.0); // 1 h
    }

    /// FRONTEIRA, lado 2: um nível entre duas entradas sobe para a **imediatamente
    /// superior**. 89,5 dB usa a linha dos 90 dB (14 400 s) — se alguém interpolar, obtém
    /// 16 200 s no ponto de 89 e este teste falha.
    #[test]
    fn nivel_entre_entradas_usa_o_superior() {
        assert_eq!(
            tempo_admissivel_s(89.5),
            14_400.0,
            "89,5 dB tem de usar a linha dos 90"
        );
        assert_eq!(
            tempo_admissivel_s(85.1),
            25_200.0,
            "85,1 dB tem de usar a linha dos 86"
        );
        // O valor interpolado seria este, e é o que NÃO pode acontecer:
        let interpolado = (16_200.0 + 14_400.0) / 2.0;
        assert_ne!(tempo_admissivel_s(89.5), interpolado);
    }

    /// Os dois casos que toda a gente sabe de cor, com a tolerância que o brief pede: 0,1 %.
    #[test]
    fn oitenta_e_cinco_por_oito_e_noventa_por_quatro_dao_cem_por_cento() {
        let a = dose_percentual(85.0, 8.0 * 3600.0).expect("85 dB/8 h é regulado");
        let b = dose_percentual(90.0, 4.0 * 3600.0).expect("90 dB/4 h é regulado");
        assert!((a - 100.0).abs() < 0.1, "85 dB/8 h deu {a} %");
        assert!((b - 100.0).abs() < 0.1, "90 dB/4 h deu {b} %");
    }

    /// A tabela é a norma, não uma conveniência: a 97 dB permite 1 h 45 min (6 300 s), e a
    /// curva contínua antiga dava 5 457 s — subestimava a exposição em 13 %.
    #[test]
    fn a_tabela_e_mais_restritiva_que_a_curva_antiga() {
        assert_eq!(tempo_admissivel_s(97.0), 6_300.0);
        let curva = 28_800.0 * libm_exp2(-(97.0 - 85.0) / 5.0);
        assert!(
            curva < 6_300.0,
            "a curva antiga dava {curva} s, abaixo dos 6 300 s da norma"
        );
    }

    /// Leq de níveis iguais é o próprio nível — controlo de identidade.
    #[test]
    fn leq_de_niveis_iguais_devolve_o_proprio_nivel() {
        let leq = leq(&[
            Exposicao {
                nivel_db: 85.0,
                duracao_s: 3600.0,
            },
            Exposicao {
                nivel_db: 85.0,
                duracao_s: 3600.0,
            },
        ])
        .expect("há medição");
        assert!((leq - 85.0).abs() < 0.01, "Leq de 85 dB iguais deu {leq}");
    }

    /// 90 dB durante metade do tempo de 85 dB dá +3 dB de Leq — a definição de energia.
    #[test]
    fn leq_de_metade_do_tempo_com_o_dobro_da_energia_sobe_tres_db() {
        let leq = leq(&[
            Exposicao {
                nivel_db: 85.0,
                duracao_s: 3600.0,
            },
            Exposicao {
                nivel_db: 88.0,
                duracao_s: 3600.0,
            },
        ])
        .expect("há medição");
        assert!(
            (leq - 86.76).abs() < 0.1,
            "Leq de 85+88 dB iguais durações deu {leq}"
        );
    }

    /// Sem medição não há Leq — `None`, nunca 0 dB (que é indistinguível de silêncio medido).
    #[test]
    fn leq_sem_medicao_devolve_none() {
        assert_eq!(leq(&[]), None);
        assert_eq!(
            leq(&[Exposicao {
                nivel_db: 85.0,
                duracao_s: 0.0
            }]),
            None
        );
        assert_eq!(
            leq(&[Exposicao {
                nivel_db: f32::NAN,
                duracao_s: 10.0
            }]),
            None
        );
    }
}
