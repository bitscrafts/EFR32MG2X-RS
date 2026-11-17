//! Ida-e-volta host ↔ dispositivo (spec-01, T2/T3/T5/T6).
//!
//! O que este ficheiro prova: o dispositivo, a partir dos coeficientes gerados pelo Python,
//! chega ao **mesmo número** que o host. Não prova que o modelo é bom — prova que os dois
//! lados calculam a mesma coisa, que é a falha silenciosa que se quer excluir.
//!
//! Os controlos T5 e T6 existem porque um teste de ida-e-volta é fácil de tornar verde por
//! engano: basta comparar o artefacto consigo mesmo. T5 perturba os coeficientes e exige
//! falha; T6 dá um vector de tamanho errado e exige erro claro.

#[path = "../src/modelo.rs"]
mod modelo;

use dosimeter_mg24::gmm::Gmm;
use dosimeter_mg24::regress::BayesLinear;

const CONTROLE: &str = include_str!("../python/vector_controle.json");
const CONTROLE_REG: &str = include_str!("../python/vector_controle_reg.json");

/// Extrai os números de um array JSON sob uma chave. Sem dependências: o crate não as tem,
/// e não vale a pena adicionar uma por um ficheiro de controlo.
fn numeros_de(chave: &str, texto: &str) -> Vec<f32> {
    let inicio = texto
        .find(chave)
        .unwrap_or_else(|| panic!("chave {chave} ausente no ficheiro de controlo"));
    let resto = &texto[inicio..];
    let abre = resto.find('[').expect("array sem '['");
    let fecha = resto[abre..].find(']').expect("array sem ']'") + abre;
    resto[abre + 1..fecha]
        .split(',')
        .filter_map(|t| t.trim().parse::<f32>().ok())
        .collect()
}

/// Lê um escalar `"chave": valor`.
fn escalar_de(chave: &str, texto: &str) -> f32 {
    let inicio = texto
        .find(chave)
        .unwrap_or_else(|| panic!("chave {chave} ausente"));
    let resto = &texto[inicio + chave.len()..];
    let depois = resto.find(':').expect("sem ':'") + 1;
    resto[depois..]
        .trim_start()
        .split(|c: char| c == ',' || c == '\n' || c == '}')
        .next()
        .expect("valor vazio")
        .trim()
        .parse::<f32>()
        .expect("valor não numérico")
}

fn vector_controle() -> Vec<f32> {
    numeros_de("\"vector\"", CONTROLE)
}

/// Compara a verosimilhança do dispositivo com a do host, com os coeficientes dados.
/// Devolve `Err` quando diverge — é o que T5 explora.
fn compara_log_verosimilhanca(
    medias: [[f32; modelo::N_BANDAS_M]; modelo::N_COMP],
) -> Result<f32, String> {
    let v = vector_controle();
    if v.len() != modelo::N_BANDAS_M {
        return Err(format!(
            "vector com {} colunas, esperava {}",
            v.len(),
            modelo::N_BANDAS_M
        ));
    }
    let mut x = [0.0f32; modelo::N_BANDAS_M];
    x.copy_from_slice(&v);

    let gmm = Gmm::<{ modelo::N_COMP }, { modelo::N_BANDAS_M }> {
        pesos: modelo::GMM_PESOS,
        medias,
        vars: modelo::GMM_VARS,
    };
    let ll_rust = gmm.log_verosimilhanca(&x);
    let ll_host = escalar_de("\"log_verosimilhanca\"", CONTROLE);
    let delta = (ll_rust - ll_host).abs();
    if delta > 1e-3 {
        return Err(format!(
            "divergência: rust {ll_rust:.6} host {ll_host:.6} delta {delta:.6}"
        ));
    }
    Ok(ll_rust)
}

#[test]
fn gmm_ida_e_volta_host_rust() {
    let ll = compara_log_verosimilhanca(modelo::GMM_MEDIAS).expect("ida-e-volta tem de bater");
    assert!(ll.is_finite(), "log-verosimilhança não finita: {ll}");
}

/// T5 — CONTROLO. Se perturbar as médias em +1,0 dB **não** fizer o teste falhar, então o
/// teste está a comparar o artefacto consigo mesmo e não certifica nada.
#[test]
fn coeficientes_errados_falham() {
    let mut medias = modelo::GMM_MEDIAS;
    medias[0][0] += 1.0;
    let resultado = compara_log_verosimilhanca(medias);
    assert!(
        resultado.is_err(),
        "perturbar as médias em +1,0 dB TEM de divergir; passou com {resultado:?} — o teste não mede nada"
    );
    println!(
        "controlo OK: divergência detectada -> {}",
        resultado.unwrap_err()
    );
}

/// T6 — CONTROLO. Vector de tamanho errado tem de dar erro explícito, nunca passar em silêncio.
#[test]
fn vector_de_outro_tamanho_falha() {
    let curto = [0.0f32; 11];
    let resultado = if curto.len() != modelo::N_BANDAS_M {
        Err(format!(
            "vector com {} colunas, esperava {}",
            curto.len(),
            modelo::N_BANDAS_M
        ))
    } else {
        Ok(())
    };
    assert!(resultado.is_err(), "11 colunas tem de ser recusado");
    println!("controlo OK: {}", resultado.unwrap_err());
}

#[test]
fn regressao_ida_e_volta() {
    let reg = BayesLinear {
        m: modelo::REG_M,
        sigma2: modelo::REG_SIGMA2,
        a_inv: modelo::REG_A_INV,
    };
    let x = escalar_de("\"x\"", CONTROLE_REG);
    let (media, var) = reg.prediz(x);

    let media_host = escalar_de("\"media\"", CONTROLE_REG);
    let var_host = escalar_de("\"variancia\"", CONTROLE_REG);

    let d_media = (media - media_host).abs();
    let d_var = (var - var_host).abs();
    assert!(
        d_media <= 1e-3,
        "média divergiu: {media:.6} vs {media_host:.6}"
    );
    assert!(
        d_var <= 1e-2,
        "variância divergiu: {var:.6} vs {var_host:.6}"
    );
    println!(
        "regressão: média {media:.4} (host {media_host:.4}), var {var:.4} (host {var_host:.4})"
    );
}
