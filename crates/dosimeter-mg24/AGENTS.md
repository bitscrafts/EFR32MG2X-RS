# AGENTS.md — dosimeter-mg24

Dosímetro de ruído com incerteza para o **Seeed XIAO MG24 Sense** (EFR32MG24B220F1536IM48-B).

Vive **dentro** do repositório de suporte (`/projects/efr32mg24-rs/crates/dosimeter-mg24`), como crate do
workspace. Não há cópia fora do repositório: quem precisar da lógica de dosimetria chama **este** repo
(`efr32mg24-rs`), para não existirem duas versões da mesma aritmética.

## HARD RULE — âmbito

**Nada é publicado em crates.io.** `publish = false` no manifesto, e mesmo assim: nunca `cargo publish` nem
`cargo yank` sem uma ordem expressa do operador, na mesma conversa.

**Não se faz commit** sem ordem expressa. O trabalho fica na árvore e o operador decide quando o guardar.

**Esta crate não toca no hardware.** Não importa `efr32mg2x-pac`, nem a HAL, nem a BSP: é aritmética pura
que se prova no host. Se um dia precisar de um tipo do hardware, o valor entra como número (f32), não como
dependência — é o que a mantém testável numa máquina de desenvolvimento.

## O que este crate é

```
src/lib.rs     Leq (nível equivalente), dose acumulada NR-15, intervalo de incerteza
               9 testes no host, no_std, deny(unsafe_code)
```

A regra, como está na norma: `T(L) = 28800 * 2^(-(L-85)/5)` e `dose += t / T(L)`. Os controlos numéricos
que fecham o crate: 85 dB durante 8 h → 100 %; 90 dB durante 4 h → 100 %.

## Como se verifica

```bash
cd /projects/efr32mg24-rs
cargo test -p dosimeter-mg24 --target x86_64-unknown-linux-gnu   # 9 testes, no host
# (sem `--target`, o .cargo/config.toml do repo aponta para o alvo embebido, onde um no_std
#  sem panic_handler nao compila -- nao e' defeito da crate)
cargo build -p dosimeter-mg24     # sem dependências: a crate não tem nenhuma
```

## Estado

Escrito e testado no host. **Não medido em hardware**: a expansão (OLED/BME680/RTC/buzzer) foi devolvida
por avaria, e o caminho do microfone (IADC contínuo) ainda não existe na HAL. Ver
`docs/03-development/SPEC-DOSIMETER-MG24.md`, tarefas `E1-MIC.1` e `E1-MIC.2`.
