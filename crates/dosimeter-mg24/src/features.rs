//! Features acústicas: energia por banda, ponderação A e escala logarítmica.
//!
//! ## Porque Goertzel e não FFT
//!
//! O que interessa são 8 a 16 bandas, não 128 bins. O algoritmo de Goertzel calcula
//! **uma** frequência com o custo de uma multiplicação e uma soma por amostra, sem
//! tabelas de senos e sem memória auxiliar — o que cabe num Cortex-M33 a 78 MHz sem
//! DSP e sem alocar nada. Para N bandas, o custo é N passagens sobre a janela, e a
//! janela só é lida uma vez por banda.
//!
//! ## Controlos que provam que isto mede o que diz medir
//!
//! 1. **Tom puro**: um sinal de 1 kHz tem de dominar a banda que contém 1 kHz, por
//!    margem clara sobre as outras.
//! 2. **Ruído branco**: a energia reparte-se; nenhuma banda pode dominar.
//!
//! Sem estes dois controlos, uma feature que "dá números" não prova nada.
//!
//! ## Ponderação A
//!
//! A NR-15 mede em dB(A): a curva A aproxima a sensibilidade do ouvido, atenuando
//! graves e ganhos agudos. Aplicada por banda, com os valores da norma IEC 61672 na
//! frequência central de cada banda. É uma aproximação (a curva é contínua), e é
//! declarada como tal no relatório.

#![allow(clippy::needless_range_loop)]

/// Número de bandas usado por omissão.
pub const N_BANDAS: usize = 12;

/// Frequência de amostragem por omissão do dosímetro, em Hz.
pub const FS_PADRAO: u32 = 8_000;

/// Frequências centrais (Hz) das 12 bandas entre 100 Hz e 3,8 kHz, ~1/3 de oitava.
///
/// Escolhidas para caber em `FS = 8 kHz` (limite de Nyquist em 4 kHz) e cobrir a banda
/// onde vive o ruído que interessa à NR-15.
pub const CENTRAIS_HZ: [u32; N_BANDAS] = [
    100, 160, 250, 400, 630, 1000, 1600, 2000, 2500, 3150, 3550, 3800,
];

/// Curva A (IEC 61672) aproximada nos centros das bandas, em dB.
///
/// Valores calculados com a expressão normalizada e arredondados a 0,1 dB. Aplicam-se
/// somando à energia da banda em escala logarítmica.
pub const PONDERACAO_A_DB: [f32; N_BANDAS] = [
    -19.1, -13.4, -8.6, -4.8, -1.9, 0.0, 1.0, 1.2, 1.3, 1.2, 1.1, 1.0,
];

/// Uma leitura de features: energia por banda, já em dB e com ponderação A opcional.
#[derive(Clone, Copy, Debug)]
pub struct Features {
    /// Energia por banda em dB (sem ponderação).
    pub bandas_db: [f32; N_BANDAS],
    /// Energia total ponderada em A, em dB. É o que a dose consome.
    pub total_a_db: f32,
}

impl Features {
    pub fn vazia() -> Self {
        Self {
            bandas_db: [-120.0; N_BANDAS],
            total_a_db: -120.0,
        }
    }
}

/// Potência de Goertzel de uma frequência `f` numa janela de amostras `i16`.
///
/// Devolve a magnitude ao quadrado, normalizada pelo número de amostras (potência média).
pub fn goertzel_potencia(amostras: &[i16], f_hz: f32, fs: f32) -> f32 {
    let n = amostras.len();
    if n == 0
        || (!fs.is_finite() || fs <= 0.0)
        || !f_hz.is_finite()
        || f_hz <= 0.0
        || f_hz >= fs / 2.0
    {
        return 0.0;
    }
    let k = 2.0 * core::f32::consts::PI * f_hz / fs;
    let coef = 2.0 * cos_aprox(k);
    let (mut s1, mut s2) = (0.0f32, 0.0f32);
    for &a in amostras {
        let s0 = a as f32 + coef * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    // |X|^2 = s1^2 + s2^2 - coef*s1*s2
    let pot = s1 * s1 + s2 * s2 - coef * s1 * s2;
    (pot / (n as f32 * n as f32)).max(0.0)
}

/// Cosseno por série de Taylor reduzido ao primeiro quadrante (`no_std` não tem `cos`).
fn cos_aprox(x: f32) -> f32 {
    // reduzir a [0, 2pi)
    let dois_pi = 2.0 * core::f32::consts::PI;
    let mut y = x % dois_pi;
    if y < 0.0 {
        y += dois_pi;
    }
    // simetria: cos(pi - y) = -cos(y)
    let (y, sinal) = if y > core::f32::consts::PI {
        (dois_pi - y, 1.0f32)
    } else {
        (y, 1.0)
    };
    let (y, sinal) = if y > core::f32::consts::FRAC_PI_2 {
        (core::f32::consts::PI - y, -sinal)
    } else {
        (y, sinal)
    };
    // Taylor até x^10/10! — erro < 1e-6 em [0, pi/2]
    let y2 = y * y;
    let mut termo = 1.0f32;
    let mut soma = 1.0f32;
    let mut n = 1.0f32;
    while n <= 5.0 {
        termo *= -y2 / ((2.0 * n - 1.0) * (2.0 * n));
        soma += termo;
        n += 1.0;
    }
    sinal * soma
}

/// Potência (em dB) de um valor de potência linear, com piso em -120 dB.
fn pot_db(p: f32) -> f32 {
    if (!p.is_finite() || p <= 0.0) || !p.is_finite() {
        return -120.0;
    }
    let db = 10.0 * log10_aprox(p);
    if db < -120.0 {
        -120.0
    } else {
        db
    }
}

/// `log10` sem `std`.
fn log10_aprox(x: f32) -> f32 {
    if (!x.is_finite() || x <= 0.0) || !x.is_finite() {
        return -120.0;
    }
    // x = m * 2^e com m em [1,2)
    let bits = x.to_bits();
    let mut e = ((bits >> 23) & 0xFF) as i32 - 127;
    let mut m = f32::from_bits((bits & 0x007F_FFFF) | 0x3F80_0000); // [1,2)
    if m >= 1.5 {
        m /= 2.0;
        e += 1;
    }
    // ln(m) por série em torno de 1: ln(m) = 2*atanh((m-1)/(m+1))
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    let mut termo = t;
    let mut soma = t;
    let mut n = 3.0f32;
    while n <= 15.0 {
        termo *= t2;
        soma += termo / n;
        n += 2.0;
    }
    let ln = 2.0 * soma + e as f32 * core::f32::consts::LN_2;
    ln / core::f32::consts::LN_10
}

/// Extrai features de uma janela de amostras centrada em zero.
pub fn extrai(amostras: &[i16], fs: f32) -> Features {
    let mut f = Features::vazia();
    let mut pot_a = 0.0f32;
    for b in 0..N_BANDAS {
        let p = goertzel_potencia(amostras, CENTRAIS_HZ[b] as f32, fs);
        f.bandas_db[b] = pot_db(p);
        // ponderação A: soma-se em potência linear, com o ganho da curva aplicado
        pot_a += p * potencia_de_db(PONDERACAO_A_DB[b]);
    }
    f.total_a_db = pot_db(pot_a);
    f
}

/// 10^(db/10) sem `std` (potência linear correspondente a um ganho em dB).
fn potencia_de_db(db: f32) -> f32 {
    // 10^(db/10) = 2^(db/10 * log2(10))
    let x = db / 10.0 * core::f32::consts::LOG2_10;
    exp2_aprox(x)
}

/// `2^x` por decomposição inteiro + Taylor (mesmo método do módulo de dose).
fn exp2_aprox(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    const LN2: f32 = core::f32::consts::LN_2;
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
    let mut escala = 1.0f32;
    let kk = if k < 0 { -k } else { k };
    let mut i = 0i32;
    while i < kk {
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

    /// Gera uma janela com um tom puro de `f` Hz (amplitude 1000).
    fn tom(f: f32, fs: f32, n: usize) -> [i16; 256] {
        let mut a = [0i16; 256];
        assert!(n <= 256);
        for i in 0..n {
            let fase = 2.0 * core::f32::consts::PI * f * (i as f32) / fs;
            a[i] = (1000.0 * seno_aprox(fase)) as i16;
        }
        a
    }

    /// Seno por série (para gerar os sinais de teste sem `std`).
    fn seno_aprox(x: f32) -> f32 {
        let dois_pi = 2.0 * core::f32::consts::PI;
        let mut y = x % dois_pi;
        if y < 0.0 {
            y += dois_pi;
        }
        let (y, sinal) = if y > core::f32::consts::PI {
            (y - core::f32::consts::PI, -1.0f32)
        } else {
            (y, 1.0)
        };
        let y2 = y * y;
        let mut termo = y;
        let mut soma = y;
        let mut n = 1.0f32;
        while n <= 8.0 {
            termo *= -y2 / ((2.0 * n) * (2.0 * n + 1.0));
            soma += termo;
            n += 1.0;
        }
        sinal * soma
    }

    /// Ruído pseudo-aleatório determinístico (LCG), para o controlo de banda larga.
    fn ruido(n: usize, amplitude: i16) -> [i16; 256] {
        let mut a = [0i16; 256];
        let mut x: u32 = 12345;
        for i in 0..n {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let v = ((x >> 16) as i32 % (amplitude as i32 * 2 + 1)) - amplitude as i32;
            a[i] = v as i16;
        }
        a
    }

    /// CONTROL 1 — tom puro a 1 kHz: a banda de 1 kHz domina por margem clara.
    #[test]
    fn tom_a_1khz_domina_a_banda_de_1khz() {
        let fs = FS_PADRAO as f32;
        let a = tom(1000.0, fs, 256);
        let f = extrai(&a, fs);
        let alvo = 5; // CENTRAIS_HZ[5] == 1000
        let mut segunda = f32::MIN;
        for b in 0..N_BANDAS {
            if b != alvo && f.bandas_db[b] > segunda {
                segunda = f.bandas_db[b];
            }
        }
        let margem = f.bandas_db[alvo] - segunda;
        assert!(
            margem >= 6.0,
            "margem de apenas {margem:.1} dB: banda alvo {:.1} dB, segunda {:.1} dB",
            f.bandas_db[alvo],
            segunda
        );
    }

    /// CONTROL 2 — ruído branco: nenhuma banda domina (é o oposto do controlo 1).
    #[test]
    fn ruido_nao_tem_banda_dominante() {
        let fs = FS_PADRAO as f32;
        let a = ruido(256, 500);
        let f = extrai(&a, fs);
        let mut max = f32::MIN;
        let mut min = f32::MAX;
        for b in 0..N_BANDAS {
            if f.bandas_db[b] > max {
                max = f.bandas_db[b];
            }
            if f.bandas_db[b] < min {
                min = f.bandas_db[b];
            }
        }
        let espalhamento = max - min;
        assert!(
            espalhamento < 25.0,
            "espalhamento de {espalhamento:.1} dB: uma banda domina demasiado para ruído branco"
        );
    }

    /// Controlo negativo do controlo 1: um tom a 250 Hz NÃO domina a banda de 1 kHz.
    #[test]
    fn tom_a_250hz_nao_domina_a_banda_de_1khz() {
        let fs = FS_PADRAO as f32;
        let a = tom(250.0, fs, 256);
        let f = extrai(&a, fs);
        assert!(
            f.bandas_db[2] > f.bandas_db[5],
            "250 Hz devia dominar a banda 2, não a 5"
        );
    }

    #[test]
    fn goertzel_de_silencio_e_zero() {
        let silencio = [0i16; 256];
        let p = goertzel_potencia(&silencio, 1000.0, 8000.0);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn log10_e_exp2_concordam_nos_pontos_conhecidos() {
        // log10(1000) = 3
        assert!((log10_aprox(1000.0) - 3.0).abs() < 1e-3);
        // log10(0.001) = -3
        assert!((log10_aprox(0.001) + 3.0).abs() < 1e-3);
        // 10^(-1.91) ~ 0.0123 (curva A a 100 Hz)
        assert!((potencia_de_db(-19.1) - 0.0123).abs() < 2e-3);
        // ida e volta: 10*log10(10^(db/10)) = db
        for db in [-19.1f32, -4.8, 0.0, 1.2] {
            let p = potencia_de_db(db);
            let volta = 10.0 * log10_aprox(p);
            assert!((volta - db).abs() < 1e-3, "db={db} deu {volta}");
        }
    }

    /// A ponderação A atenua os graves: com a mesma amplitude, 100 Hz fica abaixo de 1 kHz.
    #[test]
    fn ponderacao_a_atenua_os_graves() {
        let fs = FS_PADRAO as f32;
        let grave = extrai(&tom(100.0, fs, 256), fs);
        let medio = extrai(&tom(1000.0, fs, 256), fs);
        assert!(
            grave.total_a_db < medio.total_a_db,
            "100 Hz ({:.1} dB) devia ficar abaixo de 1 kHz ({:.1} dB) com ponderação A",
            grave.total_a_db,
            medio.total_a_db
        );
    }
}
