//! Regressão linear bayesiana — nível seguinte com incerteza (P2).
//!
//! ## O modelo
//!
//! Para prever o nível do instante seguinte a partir do actual: `y = w0 + w1·x + ruído`,
//! com ruído gaussiano de variância conhecida `sigma2` e prior gaussiano isotrópico de
//! precisão `alpha` sobre `w`.
//!
//! ```text
//! posterior   A = alpha·I + Phi^T Phi / sigma2      m = A^-1 Phi^T y / sigma2
//! preditiva   media = m·phi(x*)                    var = sigma2·(1 + phi^T A^-1 phi)
//! ```
//!
//! A variância preditiva tem **dois** termos e é isso que a torna honesta: `sigma2` é o
//! ruído irredutível da medição, e `sigma2·phi^T A^-1 phi` é a incerteza sobre os próprios
//! parâmetros — que encolhe onde há dados e cresce longe deles.
//!
//! ## Porque não um GP com kernel
//!
//! Um GP com RBF exige resolver um sistema `n×n` no treino e guardar `n` pontos no
//! dispositivo. Com um regressor linear bayesiano guardam-se **dois** coeficientes e uma
//! matriz 2×2: cabe em 20 bytes de RAM, e a inferência é uma multiplicação. O curso pede
//! predições probabilísticas, não o modelo mais caro disponível.
//!
//! ## O controlo que este módulo tem de passar
//!
//! **Cobertura.** Se se diz 90 %, então em 90 % dos casos reais o valor observado tem de
//! cair dentro do intervalo — entre 85 % e 95 % com esta amostra. Um modelo pode ter bom
//! erro médio e intervalos errados; é por isso que as duas coisas se medem em separado.

#![allow(clippy::needless_range_loop)]

/// Regressor linear bayesiano de ordem 1 (`phi = [1, x]`).
#[derive(Clone, Copy, Debug)]
pub struct BayesLinear {
    /// Média posterior dos coeficientes: `[w0, w1]`.
    pub m: [f32; 2],
    /// Variância do ruído de observação.
    pub sigma2: f32,
    /// `(alpha·I + Phi^T Phi / sigma2)^-1`, necessária para a variância preditiva.
    pub a_inv: [[f32; 2]; 2],
}

impl BayesLinear {
    /// Ajusta o posterior a partir de pares `(x, y)`.
    ///
    /// Devolve `None` se não houver dados ou se os parâmetros forem inválidos — nunca um
    /// modelo a fingir que aprendeu.
    pub fn fit(xs: &[f32], ys: &[f32], alpha: f32, sigma2: f32) -> Option<Self> {
        if xs.len() != ys.len() || xs.is_empty() {
            return None;
        }
        if (!alpha.is_finite() || alpha <= 0.0)
            || (!sigma2.is_finite() || sigma2 <= 0.0)
            || !alpha.is_finite()
            || !sigma2.is_finite()
        {
            return None;
        }
        // Phi = [1, x] -> acumular Phi^T Phi e Phi^T y
        let (mut s00, mut s01, mut s11) = (0.0f32, 0.0f32, 0.0f32);
        let (mut t0, mut t1) = (0.0f32, 0.0f32);
        for i in 0..xs.len() {
            let (x, y) = (xs[i], ys[i]);
            if !x.is_finite() || !y.is_finite() {
                return None;
            }
            s00 += 1.0;
            s01 += x;
            s11 += x * x;
            t0 += y;
            t1 += x * y;
        }
        let a00 = alpha + s00 / sigma2;
        let a01 = s01 / sigma2;
        let a11 = alpha + s11 / sigma2;
        let det = a00 * a11 - a01 * a01;
        if !det.is_finite() || det.abs() <= 1e-12 {
            return None;
        }
        let a_inv = [[a11 / det, -a01 / det], [-a01 / det, a00 / det]];
        // m = A^-1 * (Phi^T y / sigma2)
        let b0 = t0 / sigma2;
        let b1 = t1 / sigma2;
        let m = [
            a_inv[0][0] * b0 + a_inv[0][1] * b1,
            a_inv[1][0] * b0 + a_inv[1][1] * b1,
        ];
        Some(Self { m, sigma2, a_inv })
    }

    /// Média e variância preditivas em `x`.
    pub fn prediz(&self, x: f32) -> (f32, f32) {
        if !x.is_finite() {
            return (f32::NAN, f32::INFINITY);
        }
        let media = self.m[0] + self.m[1] * x;
        // phi^T A^-1 phi com phi = [1, x]
        let q = self.a_inv[0][0] + 2.0 * self.a_inv[0][1] * x + self.a_inv[1][1] * x * x;
        let var = self.sigma2 * (1.0 + q);
        (media, if var > 0.0 { var } else { 0.0 })
    }

    /// Intervalo `media ± z·desvio` em `x`.
    pub fn intervalo(&self, x: f32, z: f32) -> (f32, f32) {
        let (m, v) = self.prediz(x);
        let s = raiz(v);
        (m - z * s, m + z * s)
    }
}

/// Raiz quadrada por Newton (sem `std`).
pub fn raiz(v: f32) -> f32 {
    if (!v.is_finite() || v <= 0.0) || !v.is_finite() {
        return 0.0;
    }
    let mut x = v;
    let mut i = 0;
    while i < 24 {
        x = 0.5 * (x + v / x);
        i += 1;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    /// Gerador determinístico (LCG) para os testes não dependerem de `rand`.
    struct Lcg(u32);
    impl Lcg {
        fn novo(semente: u32) -> Self {
            Lcg(semente)
        }
        fn uniforme(&mut self) -> f32 {
            self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((self.0 >> 8) as f32) / ((1u32 << 24) as f32)
        }
        /// Aproximação gaussiana por soma de 12 uniformes (teorema central do limite).
        fn normal(&mut self) -> f32 {
            let mut s = 0.0;
            for _ in 0..12 {
                s += self.uniforme();
            }
            s - 6.0
        }
    }

    /// Dados sintéticos de y = 2 + 0,5x + ruído de desvio 1.
    fn dados(n: usize, semente: u32) -> (Vec<f32>, Vec<f32>) {
        let mut r = Lcg::novo(semente);
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for _ in 0..n {
            let x = r.uniforme() * 40.0 + 40.0; // 40..80 dB
            let y = 2.0 + 0.5 * x + r.normal();
            xs.push(x);
            ys.push(y);
        }
        (xs, ys)
    }

    #[test]
    fn recupera_os_coeficientes_de_dados_limpos() {
        // sem ruído: y = 2 + 0.5x exacto
        let xs: Vec<f32> = (0..50).map(|i| 40.0 + i as f32).collect();
        let ys: Vec<f32> = xs.iter().map(|x| 2.0 + 0.5 * x).collect();
        let m = BayesLinear::fit(&xs, &ys, 1e-3, 1.0).expect("devia ajustar");
        assert!((m.m[0] - 2.0).abs() < 0.1, "w0 = {}", m.m[0]);
        assert!((m.m[1] - 0.5).abs() < 0.01, "w1 = {}", m.m[1]);
    }

    /// O erro a horizonte zero é pequeno: o modelo aprendeu a relação.
    #[test]
    fn erro_pequeno_em_dados_sinteticos() {
        let (xs, ys) = dados(200, 7);
        let m = BayesLinear::fit(&xs, &ys, 1e-3, 1.0).expect("devia ajustar");
        let mut soma = 0.0;
        for i in 0..xs.len() {
            let (media, _) = m.prediz(xs[i]);
            soma += (media - ys[i]).abs();
        }
        let mae = soma / xs.len() as f32;
        assert!(mae < 1.0, "erro absoluto médio {mae:.2} devia ser < 1 dB");
    }

    /// CONTROL: a cobertura a 90 % tem de ficar entre 85 % e 95 %.
    #[test]
    fn cobertura_do_intervalo_a_90_por_cento() {
        let (xs, ys) = dados(300, 11);
        let m = BayesLinear::fit(&xs, &ys, 1e-3, 1.0).expect("devia ajustar");
        let z = 1.644_853_6; // quantil 95 % da normal
        let mut dentro = 0usize;
        let mut total = 0usize;
        let mut r = Lcg::novo(99);
        // pontos de teste independentes do treino, com ruído novo
        for _ in 0..2000 {
            let x = r.uniforme() * 40.0 + 40.0;
            let y = 2.0 + 0.5 * x + r.normal();
            let (lo, hi) = m.intervalo(x, z);
            if y >= lo && y <= hi {
                dentro += 1;
            }
            total += 1;
        }
        let cobertura = dentro as f32 / total as f32;
        assert!(
            cobertura >= 0.85 && cobertura <= 0.95,
            "cobertura {:.3} fora da faixa 0,85-0,95",
            cobertura
        );
    }

    /// CONTROL inverso: uma incerteza declarada pequena tem de dar cobertura baixa.
    /// Se um intervalo de ±0,1 dB "cobrisse" 90 % dos casos, o teste de cobertura não
    /// estaria a medir nada.
    #[test]
    fn intervalo_estreito_nao_cobre() {
        let (xs, ys) = dados(300, 13);
        let m = BayesLinear::fit(&xs, &ys, 1e-3, 1.0).expect("devia ajustar");
        let mut dentro = 0usize;
        let mut total = 0usize;
        let mut r = Lcg::novo(101);
        for _ in 0..1000 {
            let x = r.uniforme() * 40.0 + 40.0;
            let y = 2.0 + 0.5 * x + r.normal();
            let (media, _) = m.prediz(x);
            if (y - media).abs() <= 0.1 {
                dentro += 1;
            }
            total += 1;
        }
        let cobertura = dentro as f32 / total as f32;
        assert!(
            cobertura < 0.20,
            "um intervalo de +-0,1 dB cobriu {:.0} % — o teste nao mede o que diz",
            cobertura * 100.0
        );
    }

    #[test]
    fn sem_dados_nao_ha_modelo() {
        assert!(BayesLinear::fit(&[], &[], 1.0, 1.0).is_none());
        assert!(BayesLinear::fit(&[1.0], &[], 1.0, 1.0).is_none());
    }

    #[test]
    fn parametros_invalidos_nao_ajustam() {
        let (xs, ys) = dados(20, 3);
        assert!(BayesLinear::fit(&xs, &ys, 0.0, 1.0).is_none());
        assert!(BayesLinear::fit(&xs, &ys, 1.0, 0.0).is_none());
    }

    #[test]
    fn raiz_concorda_com_quadrados_conhecidos() {
        assert!((raiz(4.0) - 2.0).abs() < 1e-5);
        assert!((raiz(2.0) - 1.414_213_6).abs() < 1e-4);
        assert_eq!(raiz(-1.0), 0.0);
    }
}
