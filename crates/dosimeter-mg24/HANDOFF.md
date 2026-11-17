# HANDOFF — dosimeter-mg24

**Actualizado:** 2026-11-17 · **Entrega:** 30/11 (SIGAA) · **Disciplina:** CK0475

## O que é

Dosímetro de ruído com **incerteza**: mede exposição (NR-15), corre o modelo no MCU e mostra
no telemóvel o nível **com intervalo de credibilidade**, não um número seco.

Hardware: **XIAO MG24 Sense** — microfone analógico na placa (PC9, IADC 12 bits), BLE 5.3,
MVP. Sem fiação externa.

## Estado do código (tudo provado por testes que correm)

```
src/lib.rs       dosimetria NR-15                  9 testes
src/features.rs  Goertzel (12 bandas) + curva A    6 testes
src/gmm.rs       densidade GMM + baseline          6 testes
src/regress.rs   nível com incerteza + cobertura   7 testes
                                             total 28 testes, 0 falhas
```

**Comando:** `cargo test -p dosimeter-mg24 --target x86_64-unknown-linux-gnu`
(o `--target` é necessário enquanto o crate viver dentro do workspace do EFR32, que força
`thumbv8m`; quando o projecto passar a `/projects/dosimeter-mg24/` deixa de ser preciso).

### Os três testes que dão nota

```
cobertura_do_intervalo_a_90_por_cento      cobertura observada 0,85-0,95        PASSA
intervalo_estreito_nao_cobre               +-0,1 dB cobre < 20 % (anti-auto-engano) PASSA
gmm_bate_o_baseline_de_gaussiana_unica     GMM > gaussiana única                PASSA
tom_a_1khz_domina_a_banda_de_1khz          tom puro tem de acender a banda certa PASSA
ruido_nao_tem_banda_dominante              controlo oposto do anterior           PASSA
```

## Documentos

```
spec/E1-APP.DOSIMETER.md          especificação completa: 12 tarefas com critério medido
docs/RELATORIO-E-PAPER.md         relatório + estrutura NeurIPS/ICML
docs/APRESENTACAO.md              guião de 10 slides, com aplicabilidade nos slides 2-3
```

## O que falta, por responsável

### Nesta janela (só Rust puro, sem hardware)

```
1  treino em Python -> const arrays (GMM e regressão) + teste de ida-e-volta Rust<->Python
2  página Web Bluetooth (HTML/JS) para a demo
3  script Python de gráficos para o paper (nível + banda de incerteza ao longo do tempo)
```

### Na janela de infra (hardware)

```
E1-MIC.1   medir taxa sustentada do IADC          <- BLOQUEIA a decisão de features
E1-MIC.2   captura contínua (LDMA) + anel
E1-MIC.3   offset DC e escala
E1-BLE.1   serviço GATT: nivel / intervalo / dose / comando
E1-BLE.2   cliente no telemóvel
E1-MVP.1   MVP (opcional; se não der ganho ≥2x, fecha como "não compensa")
```

## Ferramentas: o que existe nesta máquina

```
rust + cargo (nightly para o workspace EFR32)   SIM
python 3 + numpy                                SIM (uv run --with numpy)
podman no lnx, 31 imagens                       SIM — nenhuma com JDK/Android/Gradle
java, javac, gradle, adb, kotlinc               NÃO
```

**Android:** não há toolchain local. Caminho por container é possível
(`mingc/android-build-box` ou `ghcr.io/cirruslabs/android-sdk`, ~4-5 GB de download), mas
uma app não dá nota e precisa do telemóvel para se depurar. **Recomendação: demo com
`nRF Connect` (grátis, já usada na verificação do BLE do MG24) + gráficos gerados no host.**

## Regras que valem neste projecto

```
0  NAO TOCAR NO HARDWARE ate o operador autorizar (2026-11-17):
   a placa XIAO MG24 esta' a ser gravada e testada pelo projecto de infra.
   Proibido: flash, espflash/openocd, abrir /dev/ttyACM*, usar o USB, mexer no BLE do dispositivo.
   Motivo: colisao — dois processos a gravar a mesma placa dao resultados que nao se distinguem
   de defeito de firmware.
   Permitido sem autorizacao: codigo no host (Rust/Python), testes que nao tocam no hardware,
   documentos, build de containers (podman no lnx, que nao e' a placa).
```

```
1  só escrever dentro de crates/dosimeter-mg24/ · sem commits nesta janela
2  o trabalho de infra (IADC, LDMA, BLE, MVP) é de outra janela
3  nenhum número no relatório sem teste que o produza; o resto fica [A MEDIR]
4  resultados negativos entram no paper (o PPCA espectral não bateu a interpolação linear)
```

## Aplicabilidade (entra na apresentação)

```
dosimetro pessoal   84 +-1,5 dB e 88 +-6 dB sao decisoes diferentes (NR-15: 85 dB(A)/8 h)
maquinas            um sensor por maquina; dois microfones nao se instalam
rede urbana         nos baratos, sem sincronismo entre si
```
