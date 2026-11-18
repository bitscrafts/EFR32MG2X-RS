# Spec: dma moves into the HAL, one copy only (H-pub 4/6)
#: depends=C1 status=- block=-

## Overview

Mesmo movimento do C1, aplicado a `dma`. A fonte passa a viver em
`crates/efr32mg2x-hal/src/core/dma.rs` e a crate em `libraries/dma-core` fica como harness
de teste, a compilar esse mesmo ficheiro por `#[path]`. Uma cópia, dois compiladores: o host corre
os testes, o alvo produz o firmware.

## Requirements

**R1 -- o check cobre 4 cores e continua a apanhar uma cópia.**
`scripts/check-core-split.py` passa a ler os shims de todos os cores movidos.
Exemplo: com `4` cores movidos imprime `OK: 4 cores, one copy each`.

**R2 -- os testes de dma continuam a correr no host, sem hardware.**
Exemplo: `cargo test -p efr32mg2x-dma-core --target x86_64-unknown-linux-gnu` passa.

**R3 -- o HAL continua a compilar no host e no alvo.**
Exemplo: `cargo check -p efr32mg2x-hal --target x86_64-unknown-linux-gnu --no-default-features --features mg24` e
`bash scripts/gate.sh` verdes.

## Files

- `libraries/dma-core/src/lib.rs`
- `crates/efr32mg2x-hal/src/core/dma.rs`
- `crates/efr32mg2x-hal/src/core/mod.rs`
- `crates/efr32mg2x-hal/src/dma.rs`
- `crates/efr32mg2x-hal/Cargo.toml`
- `scripts/check-core-split.py`

## TDD Contract

| Test Name | Input | Expected Output |
|---|---|---|
| the_second_core_is_covered_too | 4 shims, all pointing at existing HAL files | `OK`, count is 4 |
| a_copy_of_the_second_core_is_caught | `dma-core/src/lib.rs` with bodies pasted back | `FAIL`, naming the file |
| no_shim_points_outside_the_tree | a `#[path]` pointing at `/projects/...` | `FAIL` |

## Exit Criteria

- [ ] `python3 scripts/check-core-split.py`
- [ ] `cargo test -q -p efr32mg2x-dma-core --target x86_64-unknown-linux-gnu`
- [ ] `bash scripts/gate.sh`

## Guardrails

- Não copiar o ficheiro: duas cópias é o item falhado, mesmo com os testes verdes.
- Não publicar e não flashar.
- Um core por item.
