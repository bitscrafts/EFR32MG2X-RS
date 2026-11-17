# dosimeter-mg24 — spec-01: Treino host e coeficientes para o dispositivo
#: depends=- status=pending block=-

- **Project**: `/projects/efr32mg24-rs/crates/dosimeter-mg24`
- **Priority**: sem isto o modelo no dispositivo não tem coeficientes; é a última peça que
  se fecha **sem hardware**.
- **Status**: SPECIFIED — not yet dispatched · *(lifecycle: SPECIFIED → IN PROGRESS
  on dispatch → IMPLEMENTED — awaiting review → DONE after the orchestrator
  commits and updates HANDOFF.md)*
- **Source**: manual (2026-11-17) · **Depends-On**: none

---

## Verified Premises

Cada afirmação abaixo foi confirmada por leitura do ficheiro, antes de escrever a spec.

- `src/features.rs:29` — `pub const N_BANDAS: usize = 12;` — o vector de features tem 12
  dimensões fixas.
- `src/features.rs:163` — `pub fn extrai(amostras: &[i16], fs: f32) -> Features` — as
  features produzem-se de amostras cruas; é esta a função que o treino tem de imitar.
- `src/gmm.rs:32` — `pub struct Gmm<const K: usize, const D: usize>` com campos
  `pesos: [f32; K]`, `medias: [[f32; D]; K]`, `vars: [[f32; D]; K]` — coeficientes em
  arrays de tamanho fixo, prontos para `const`.
- `src/gmm.rs:45` — `pub fn log_verosimilhanca(&self, x: &[f32; D]) -> f32` — a inferência
  que tem de reproduzir o valor calculado no host.
- `src/regress.rs:35` — `pub struct BayesLinear` com `m: [f32; 2]`, `sigma2: f32`,
  `a_inv: [[f32; 2]; 2]`.
- `src/regress.rs:49` — `pub fn fit(xs: &[f32], ys: &[f32], alpha: f32, sigma2: f32) -> Option<Self>`.
- `src/regress.rs:89` — `pub fn prediz(&self, x: f32) -> (f32, f32)` — devolve (média, variância).
- `src/lib.rs:33` — `#[cfg(test)] extern crate alloc;` — o crate é `no_std`; nos testes
  `Vec` só existe por causa desta linha.
- `Cargo.toml` — **sem dependências** (`[dependencies]` vazio). O crate não pode ganhar
  dependências: o alvo é um Cortex-M33 sem alocador.
- Contagem actual de testes: **28 a passar**, medido com
  `cargo test -p dosimeter-mg24 --target x86_64-unknown-linux-gnu`.

---

## Overview

O crate tem os três modelos implementados e testados com dados **sintéticos**: features
(Goertzel + curva A), densidade (GMM) e regressão com incerteza. O que falta é o caminho
entre os dados reais do sensor e esses modelos: **um treino no host que produz os
coeficientes como `const` arrays, e um teste que prove que o dispositivo calcula o mesmo
que o host**.

Sem este elo, o firmware precisaria de treinar no MCU — o que o crate decidiu não fazer,
porque o dispositivo não tem os dados todos em memória e um modelo treinado com poucas
centenas de janelas no próprio chip seria pior.

O defeito a evitar, e é o motivo desta spec existir: **dois caminhos de cálculo que
divergem em silêncio**. Python treina com uma normalização, Rust infere com outra, e o
número no ecrã é plausível e errado. A defesa é um teste de ida-e-volta com tolerância
declarada e o mesmo vector de entrada dos dois lados.

---

## Requirements

### Functional

- **R1 (treino do GMM)**: `python/treina_gmm.py` lê um CSV de features (linhas = janelas,
  12 colunas = bandas em dB), ajusta um GMM diagonal com `K` componentes e `D=12`, e escreve
  `src/modelo.rs` com `pub const GMM_PESOS`, `GMM_MEDIAS`, `GMM_VARS` — arrays de tamanho
  fixo, sem alocação, prontos para `const`.
- **R2 (treino da regressão)**: `python/treina_regressao.py` lê pares `(nivel_t, nivel_{t+1})`
  do mesmo CSV e escreve `pub const REG_M`, `REG_SIGMA2`, `REG_A_INV` em `src/modelo.rs`.
  Quando o ficheiro já existe, **acrescenta** o bloco da regressão sem destruir o do GMM.
- **R3 (ida-e-volta)**: existe um teste em `src/modelo.rs` (ou `tests/`) que, para uma janela
  fixa gravada em `python/vector_controle.json`, compara `Gmm::log_verosimilhanca` em Rust
  com o valor gravado pelo Python, exigindo `|Δ| ≤ 1e-3`. O vector e o valor esperado são
  gerados pelo script — nunca escritos à mão.
- **R4 (sem dependências)**: o crate continua sem `[dependencies]`; o Python usa apenas
  `numpy` (via `uv run --with numpy`), sem `scikit-learn`, porque o objectivo é ter o
  algoritmo explícito e auditável.
- **R5 (reprodutibilidade)**: o treino é determinístico — `--semente` obrigatório; correr
  duas vezes com a mesma semente produz `src/modelo.rs` byte a byte igual.

### Non-Functional

- **R6**: `src/modelo.rs` gerado tem de compilar em `no_std` (sem `std`, sem `alloc`).
- **R7**: a geração é rápida o suficiente para iterar: < 5 s para 5 000 janelas.

---

## Architecture

```
python/treina_gmm.py  ──┐
python/treina_regressao.py ──┴──> src/modelo.rs (const arrays)  <── src/gmm.rs, src/regress.rs
                    │
                    └──> python/vector_controle.json  <── teste de ida-e-volta (R3)
```

### Decisões e o que foi rejeitado

- **Coeficientes em `const`, não carregados de flash.** Um array `const` entra na imagem e
  o linker dimensiona-o; não há parsing, não há falha em runtime, e o custo em RAM é zero
  (fica em flash). Rejeitado: ficheiro de configuração em flash com parser — mais código,
  mais modos de falha, e o dispositivo não tem como validar o que lê.
- **Formato do `modelo.rs` com vírgulas e arrays literais.** Rejeitado: `serde`, `bincode`,
  `include_bytes!` com pós-processamento — todos acrescentam código no lado do dispositivo
  para resolver um problema que só existe no host.
- **Diagonal e não covariância completa.** Já decidido no crate: com 12 dimensões, a
  covariância completa tem 78 parâmetros por componente, não estimáveis com os dados que
  vamos ter.
- **Treino em Python e não em Rust.** O host tem `numpy` e iteração rápida; o dispositivo não
  treina. Rejeitado: treinar em Rust no host só para "usar uma linguagem" — dobra o código
  sem ganho.

---

## TDD Contract

| id | teste | given | expects |
|---|---|---|---|
| T1 | `modelo_compila_sem_dependencias` | `src/modelo.rs` gerado | `cargo build` do crate passa, e `grep -c '^\[dependencies\]' Cargo.toml` + linhas seguintes = 0 entradas |
| T2 | `gmm_ida_e_volta_host_rust` | vector fixo + valor do Python | `|ll_rust − ll_python| ≤ 1e-3` |
| T3 | `regressao_ida_e_volta` | mesmos `(x,y)` do Python | `|Δmédia| ≤ 1e-3` e `|Δvariância| ≤ 1e-2` |
| T4 | `treino_deterministico` | duas corridas, mesma semente | ficheiros `modelo.rs` idênticos (`cmp`) |
| T5 | **CONTROLE** `coeficientes_errados_falham` | `GMM_MEDIAS` perturbado em +1,0 numa banda | T2 tem de FALHAR — prova que o teste detecta divergência |
| T6 | **CONTROLE** `vector_de_outro_tamanho_falha` | `vector_controle.json` com 11 colunas | o teste falha com erro claro, não passa em silêncio |

**A R3 é a que uma implementação plausível satisfaz em aparência apenas.** O caminho fácil é
gerar o `vector_controle.json` a partir do *próprio* Rust e comparar Rust com Rust — ida-e-volta
verdadeira seria trivialmente verde e não provaria nada. Por isso T5 existe: se perturbar os
coeficientes não fizer o teste falhar, o teste está a comparar o artefacto consigo mesmo.

---

## Exit Criteria

Cada critério corre no crate. **Verificado contra a árvore pré-implementação: todos FALHAM
agora** (excepto os marcados «carregado», que passam por desenho e por isso não contam como
evidência desta parte).

### Novos nesta parte

- [ ] `test -f python/treina_gmm.py && grep -q "def main" python/treina_gmm.py`
- [ ] `test -f python/treina_regressao.py && grep -q "def main" python/treina_regressao.py`
- [ ] `test -f src/modelo.rs && grep -q "pub const GMM_MEDIAS" src/modelo.rs`
- [ ] `test -f python/vector_controle.json && python3 -c "import json,sys; d=json.load(open('python/vector_controle.json')); sys.exit(0 if len(d['vector'])==12 else 1)"`
- [ ] `cargo test -p dosimeter-mg24 --target x86_64-unknown-linux-gnu --test ida_e_volta gmm_ida_e_volta_host_rust 2>&1 | grep -qE "^test result: ok\. [1-9]"`
- [ ] `test -d python && ! grep -rq "sklearn\|scikit" python/`

### Carried re-check — re-checks carregados (passam antes; não são evidência desta parte)

- [ ] `cargo tree -p dosimeter-mg24 2>/dev/null | grep -qE "^dosimeter-mg24 v[0-9]" && ! cargo tree -p dosimeter-mg24 2>/dev/null | grep -q "─"`

- [ ] `cargo test -p dosimeter-mg24 --target x86_64-unknown-linux-gnu --lib 2>&1 | grep -qE "^test result: ok\. (2[8-9]|[3-9][0-9]) passed"`
- [ ] `grep -q "pub mod features;" src/lib.rs && grep -q "pub mod gmm;" src/lib.rs && grep -q "pub mod regress;" src/lib.rs`

---

## Guardrails

- **Não editar esta spec.** É o contrato; alterações fazem-se por nova spec.
- **Não commitar.** O commit é do orchestrator depois da revisão. *(Razão: o histórico do
  repositório é a prova do que foi entregue; um worker que commita destrói essa separação.)*
- **Não enfraquecer nem apagar teste.** Se um teste falha, o defeito está no código. *(Razão:
  o valor destes testes é detectarem divergência entre host e dispositivo; um teste afrouxado
  é a única forma de essa divergência passar despercebida.)*
- **Não adicionar dependências ao crate** (`Cargo.toml` sem `[dependencies]`). *(Razão: o alvo
  é Cortex-M33 sem alocador; uma dependência com `std` quebra a compilação embarcada.)*
- **Não escrever o `vector_controle.json` à mão** — é gerado pelo script. *(Razão: um vector
  escrito à mão testa a compreensão do autor, não o código.)*
- **Se uma premissa desta spec estiver errada, PARE e reporte** em vez de codificar em volta.
- **Não tocar em `src/lib.rs`, `src/features.rs`, `src/gmm.rs`, `src/regress.rs`** — o âmbito
  desta parte é o treino no host e o ficheiro gerado. Alterações nos modelos exigem spec nova.
