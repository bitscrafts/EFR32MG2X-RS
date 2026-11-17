# dosimeter-mg24 — spec-02: captura contínua e features no dispositivo
#: depends=01 status=pending block=-

- **Project**: `/projects/efr32mg24-rs/crates/dosimeter-mg24`
- **Priority**: sem captura contínua não há features, e sem features o modelo não recebe
  nada. É a primeira peça que precisa do dispositivo.
- **Status**: SPECIFIED — not yet dispatched · *(lifecycle: SPECIFIED → IN PROGRESS
  on dispatch → IMPLEMENTED — awaiting review → DONE after the orchestrator
  commits and updates HANDOFF.md)*
- **Source**: manual (2026-11-17) · **Depends-On**: spec-01 (coeficientes e ida-e-volta)

---

## Verified Premises

Confirmado por leitura do repositório `efr32mg24-rs`, antes de escrever esta spec.

- `crates/efr32mg2x-hal/src/microphone.rs:225` — `pub fn sample(&self) -> Option<u16>`:
  **uma** conversão IADC por chamada. É polled, não é streaming.
- `crates/efr32mg2x-hal/src/microphone.rs:238` — `pub fn level(&self, window: usize) -> Level`:
  calcula RMS/pico sobre `window` amostras colhidas por polling.
- `crates/efr32mg2x-hal/src/microphone.rs:28` — o próprio ficheiro declara
  «The DMA path belongs later, next to the streaming work» — **a captura contínua não existe**.
- `crates/xiao-mg24-bsp/examples/module_mic_level.rs:69` — `.level(128)` com 128 amostras por
  janela; o exemplo move um LED com o nível. Prova que o caminho polled funciona.
- `crates/xiao-mg24-bsp/examples/module_ble_beacon.rs`, `module_ble_motion.rs`,
  `module_ble_fall.rs` — existem três exemplos BLE no BSP, o que confirma que o rádio e o
  GATT estão a funcionar em hardware.
- O crate `dosimeter-mg24` tem `features::extrai(&[i16], f32) -> Features` com 12 bandas,
  testado no host (spec-01, 6 testes).

**Premissa que NÃO foi verificada e por isso é a primeira tarefa:** a taxa sustentada real
do IADC em modo polled. Não há medição no repositório; o `STATUS.md` afirma «16 kHz audio
streaming», mas o código do driver não a sustenta (só `sample()` e `level()`).

---

## Overview

O dispositivo tem de passar de «lê uma amostra quando o código pede» para «produz amostras a
uma taxa conhecida, em blocos, sem perder nenhuma». Sem isso, as features de `features.rs`
não têm entrada e o modelo não corre.

O defeito concreto, e é o que esta spec ataca: **não se sabe a que taxa o microfone entrega**.
O `STATUS.md` do repositório afirma 16 kHz; o driver só tem conversão simples. Uma afirmação
de taxa não medida é o tipo de premissa que faz um relatório inteiro ruir quando alguém a
testa — por isso a primeira tarefa é medir, e o resultado dessa medição decide as features.

Existem dois caminhos e a medição escolhe:

- **≥ 8 kHz** → features espectrais completas (12 bandas de `features.rs`), como planeado.
- **2–8 kHz** → bandas largas (4 a 6) e envelope; o modelo de densidade continua viável e o
  dosímetro (que só precisa de Leq) não perde nada.

---

## Requirements

### Functional

- **R1 (medir a taxa)**: um binário `mede-taxa` conta amostras durante um intervalo conhecido
  (o `SYSRTC` ou o `RTCC` dão o tempo) e imprime `amostras/s` e o número de falhas. **Esta
  tarefa decide a forma de R2 e R3.**
- **R2 (captura em blocos)**: captura contínua com buffer de pelo menos 512 amostras por bloco,
  sem perder amostras no intervalo entre blocos; a perda é contada, não ignorada.
- **R3 (features por bloco)**: cada bloco produz um vector de 12 bandas usando
  `features::extrai`, com a taxa real medida em R1 como `fs`.
- **R4 (relatório de taxa no arranque)**: o firmware imprime a taxa configurada e a taxa
  medida nos primeiros 2 s, para que a divergência seja visível sem instrumentos.
- **R5 (fila de blocos)**: os blocos entram numa fila de capacidade mínima 4; quando a fila
  enche, o bloco mais antigo é descartado **e a perda é contada** (um contador, não um log).

### Non-Functional

- **R6**: sem `std`, sem `alloc` — buffers estáticos, tamanho declarado em `const`.
- **R7**: o ciclo captura→features cabe no orçamento entre blocos, com margem medida
  (percentagem do tempo de bloco usada), não estimada.

---

## Architecture

```
IADC (polled)  ->  buffer estático [i16; 512]  ->  features::extrai  ->  vector 12 bandas
                                |                                            |
                                +--> contador de perdas                      +--> fila (4) -> BLE
```

### Decisões e o que foi rejeitado

- **Polled primeiro, LDMA depois.** Se o polled der ≥ 2 kHz, entrega o dosímetro e o envelope
  sem escrever um driver DMA. Rejeitado começar pelo LDMA: é a peça mais difícil do HAL e não
  se sabe ainda se é necessária.
- **Medir antes de escolher a taxa.** Rejeitado fixar 16 kHz por o `STATUS.md` o afirmar — a
  afirmação não está sustentada pelo código.
- **Perdas contadas, não ignoradas.** Rejeitado «assumir que não se perde»: uma feature
  calculada sobre um bloco com amostras saltadas é um número plausível e errado.
- **Buffer estático dimensionado em `const`.** Rejeitado `heapless` ou `Vec`: mais uma
  dependência para um tamanho que se conhece à partida.

---

## TDD Contract

| id | teste | given | expects |
|---|---|---|---|
| T1 | `taxa_medida_e_reportada` | binário `mede-taxa` a correr 5 s | imprime `amostras/s` > 0 e o valor difere da nominal em < 20 % (ou reporta a diferença) |
| T2 | `blocos_nao_perdem_amostras` | 100 blocos em captura contínua | contador de perdas = 0 com o consumo a acompanhar a produção |
| T3 | `perda_e_contada` | consumo deliberadamente atrasado 50 ms | contador de perdas > 0 — **o contador não é decorativo** |
| T4 | `features_do_dispositivo_batem_com_o_host` | bloco de 512 amostras gravado no dispositivo | vector de 12 bandas igual ao calculado por `features::extrai` no host, |Δ| ≤ 0,01 dB |
| T5 | **CONTROLE** `fs_errada_muda_as_bandas` | o mesmo bloco com `fs` a metade | as bandas têm de diferir — prova que `fs` entra no cálculo |

**T3 é o que impede o teste de perdas de ser inútil.** Um contador que nunca incrementa
passa T2 trivialmente; T3 força-o a incrementar, e sem T3 não se sabe se o caminho de perda
existe.

---

## Exit Criteria

Verificado contra a árvore pré-implementação: **falham agora** (os ficheiros e binários não
existem). Re-checks carregados passam por desenho.

### Novos nesta parte

- [ ] `test -f src/bin/mede_taxa.rs || test -f examples/mede_taxa.rs`
- [ ] `grep -rq "amostras/s\|amostras_por_segundo" src/bin examples 2>/dev/null`
- [ ] `grep -rq "perdas" src/*.rs && grep -rq "PERDAS\|perdas" src/bin/*.rs 2>/dev/null`
- [ ] `grep -rq "extrai" src/bin/*.rs 2>/dev/null && ! grep -rq "Vec<" src/bin/*.rs 2>/dev/null`
- [ ] `cargo build --release --target thumbv8m.main-none-eabi 2>&1 | grep -q "Finished"`

### Carried re-check — re-checks carregados (passam antes; não são evidência desta parte)

- [ ] `cargo test --test ida_e_volta --target x86_64-unknown-linux-gnu 2>&1 | grep -qE "^test result: ok\. [1-9]"`
- [ ] `test -f src/modelo.rs && grep -q "pub const GMM_MEDIAS" src/modelo.rs`

---

## Guardrails

- **Não tocar no hardware sem autorização explícita do operador.** A placa está a ser usada
  por outro projecto; gravar ou abrir a série colide. *(Razão: dois processos a gravar a mesma
  placa produzem resultados indistinguíveis de defeito de firmware.)*
- **Não afirmar taxa sem medir.** Se R1 não correr, o relatório diz «não medido», nunca «16 kHz».
- **Não commitar** — o commit é do orchestrator.
- **Não editar esta spec** nem afrouxar critérios.
- **Não adicionar dependências** ao crate; buffers estáticos.
- **Se R1 der menos de 2 kHz, PARE e reporte** — a arquitectura de features muda e essa
  decisão é do orchestrator, não do implementador.
