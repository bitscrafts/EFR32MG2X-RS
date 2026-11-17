//! GMM diagonal — densidade da paisagem acústica (P1).
//!
//! ## O que é
//!
//! Uma mistura de `K` gaussianas com covariância **diagonal** sobre um vector de `D`
//! features (as energias por banda, em dB). Diagonal e não completa por duas razões
//! concretas: com poucos dados uma covariância completa não é estimável (D(D+1)/2
//! parâmetros por componente), e a inferência fica em O(K·D) multiplicações por janela —
//! que cabe folgado num Cortex-M33 a 78 MHz.
//!
//! ## O que este módulo NÃO faz
//!
//! Não treina. O treino faz-se no host (Python ou Rust com dados carregados) e os
//! coeficientes entram como `const` no firmware. Treinar no dispositivo com 12 dimensões e
//! poucas centenas de janelas seria pior: o MCU não tem os dados todos em memória.
//!
//! ## Controlo que o método exige
//!
//! O GMM tem de **ganhar ao baseline** de uma única gaussiana (covariância diagonal sobre
//! todos os dados). Se não ganhar em log-verosimilhança, não se usa: um modelo mais
//! complexo que não explica melhor é só mais código.

#![allow(clippy::needless_range_loop)]

use core::f32::consts::PI;

/// Limite do expoente no log-sum-exp, para `exp` não estourar.
const LOGSUMEXP_MIN: f32 = -80.0;

/// Mistura de gaussianas com covariância diagonal.
#[derive(Clone, Copy, Debug)]
pub struct Gmm<const K: usize, const D: usize> {
    /// Pesos das componentes (devem somar 1).
    pub pesos: [f32; K],
    /// Médias por componente e por dimensão.
    pub medias: [[f32; D]; K],
    /// Variâncias por componente e por dimensão (positivas).
    pub vars: [[f32; D]; K],
}

impl<const K: usize, const D: usize> Gmm<K, D> {
    /// Log-verosimilhança de uma janela, em escala logarítmica.
    ///
    /// `log p(x) = logsumexp_k[ log w_k + log N(x | mu_k, diag(var_k)) ]`
    pub fn log_verosimilhanca(&self, x: &[f32; D]) -> f32 {
        let mut termos = [0.0f32; K];
        for k in 0..K {
            let mut log_n = 0.0f32;
            for d in 0..D {
                let v = self.vars[k][d];
                if (!v.is_finite() || v <= 0.0) || !v.is_finite() {
                    log_n = LOGSUMEXP_MIN;
                    break;
                }
                let r = x[d] - self.medias[k][d];
                log_n += -0.5 * (ln_2pi() + ln_aprox(v) + r * r / v);
            }
            let w = self.pesos[k];
            termos[k] = if w > 0.0 {
                ln_aprox(w) + log_n
            } else {
                LOGSUMEXP_MIN
            };
        }
        logsumexp(&termos)
    }

    /// Componente mais provável para uma janela.
    pub fn componente_mais_provavel(&self, x: &[f32; D]) -> usize {
        let mut melhor = 0usize;
        let mut melhor_valor = f32::MIN;
        for k in 0..K {
            let mut log_n = 0.0f32;
            for d in 0..D {
                let v = self.vars[k][d];
                if v <= 0.0 {
                    log_n = LOGSUMEXP_MIN;
                    break;
                }
                let r = x[d] - self.medias[k][d];
                log_n += -0.5 * (ln_2pi() + ln_aprox(v) + r * r / v);
            }
            let valor = if self.pesos[k] > 0.0 {
                ln_aprox(self.pesos[k]) + log_n
            } else {
                LOGSUMEXP_MIN
            };
            if valor > melhor_valor {
                melhor_valor = valor;
                melhor = k;
            }
        }
        melhor
    }
}

/// Baseline: uma gaussiana diagonal única sobre os mesmos dados.
#[derive(Clone, Copy, Debug)]
pub struct GaussianaDiagonal<const D: usize> {
    pub media: [f32; D],
    pub var: [f32; D],
}

impl<const D: usize> GaussianaDiagonal<D> {
    pub fn log_verosimilhanca(&self, x: &[f32; D]) -> f32 {
        let mut log_n = 0.0f32;
        for d in 0..D {
            let v = self.var[d];
            if !v.is_finite() || v <= 0.0 {
                return LOGSUMEXP_MIN;
            }
            let r = x[d] - self.media[d];
            log_n += -0.5 * (ln_2pi() + ln_aprox(v) + r * r / v);
        }
        log_n
    }
}

fn logsumexp(termos: &[f32]) -> f32 {
    let mut max = f32::MIN;
    for &t in termos {
        if t > max {
            max = t;
        }
    }
    if !max.is_finite() {
        return LOGSUMEXP_MIN;
    }
    let mut soma = 0.0f32;
    for &t in termos {
        soma += exp_aprox(t - max);
    }
    max + ln_aprox(soma)
}

const fn ln_2pi() -> f32 {
    // ln(2*pi) = 1.8378770664093453
    1.837_877_1
}

/// `ln` sem `std`.
pub fn ln_aprox(x: f32) -> f32 {
    if (!x.is_finite() || x <= 0.0) || !x.is_finite() {
        return LOGSUMEXP_MIN;
    }
    let bits = x.to_bits();
    let mut e = ((bits >> 23) & 0xFF) as i32 - 127;
    let mut m = f32::from_bits((bits & 0x007F_FFFF) | 0x3F80_0000);
    if m >= 1.5 {
        m /= 2.0;
        e += 1;
    }
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
    2.0 * soma + e as f32 * core::f32::consts::LN_2
}

/// `exp` sem `std`, por decomposição em potência de 2 + Taylor.
pub fn exp_aprox(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    if x < -80.0 {
        return 0.0;
    }
    let z = x / core::f32::consts::LN_2;
    let k = if z >= 0.0 {
        (z + 0.5) as i32
    } else {
        (z - 0.5) as i32
    };
    let r = (z - k as f32) * core::f32::consts::LN_2;
    let mut termo = 1.0f32;
    let mut soma = 1.0f32;
    let mut n = 1.0f32;
    while n <= 9.0 {
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

// `PI` só é usado para documentar o termo 2π no cabeçalho; o valor entra por `ln_2pi`.
const _: f32 = PI;

#[cfg(test)]
mod tests {
    use super::*;

    /// Duas componentes bem separadas em 1 dimensão, para os testes de densidade.
    fn gmm_duas() -> Gmm<2, 1> {
        Gmm {
            pesos: [0.7, 0.3],
            medias: [[0.0], [10.0]],
            vars: [[1.0], [1.0]],
        }
    }

    #[test]
    fn a_janela_proxima_da_media_tem_maior_verosimilhanca() {
        let g = gmm_duas();
        let ll_perto = g.log_verosimilhanca(&[0.0]);
        let ll_longe = g.log_verosimilhanca(&[5.0]);
        assert!(ll_perto > ll_longe, "perto={ll_perto} longe={ll_longe}");
    }

    #[test]
    fn a_componente_escolhida_e_a_certa() {
        let g = gmm_duas();
        assert_eq!(g.componente_mais_provavel(&[0.2]), 0);
        assert_eq!(g.componente_mais_provavel(&[9.8]), 1);
    }

    /// Controlo negativo do método: o GMM tem de bater o baseline de gaussiana única.
    #[test]
    fn gmm_bate_o_baseline_de_gaussiana_unica() {
        let g = gmm_duas();
        // baseline ajustado aos mesmos dois grupos: media ~3, variancia grande
        let base = GaussianaDiagonal::<1> {
            media: [3.0],
            var: [30.0],
        };
        let pontos = [0.0f32, 0.2, -0.3, 9.8, 10.1, 10.0];
        let mut soma_gmm = 0.0;
        let mut soma_base = 0.0;
        for &p in &pontos {
            soma_gmm += g.log_verosimilhanca(&[p]);
            soma_base += base.log_verosimilhanca(&[p]);
        }
        assert!(
            soma_gmm > soma_base,
            "GMM {soma_gmm:.2} devia bater o baseline {soma_base:.2}"
        );
    }

    #[test]
    fn logsumexp_estavel_com_termos_muito_negativos() {
        let v = logsumexp(&[-1000.0, -1001.0]);
        assert!(v.is_finite() && v < -999.0, "deu {v}");
    }

    #[test]
    fn variancia_invalida_nao_produz_nan() {
        let g = Gmm::<1, 1> {
            pesos: [1.0],
            medias: [[0.0]],
            vars: [[0.0]],
        };
        let ll = g.log_verosimilhanca(&[0.0]);
        assert!(ll.is_finite(), "deu {ll}");
    }

    #[test]
    fn ln_e_exp_ida_e_volta() {
        for x in [0.1f32, 1.0, 2.5, 10.0, 100.0] {
            let volta = exp_aprox(ln_aprox(x));
            assert!((volta - x).abs() / x < 1e-4, "x={x} volta={volta}");
        }
    }
}
