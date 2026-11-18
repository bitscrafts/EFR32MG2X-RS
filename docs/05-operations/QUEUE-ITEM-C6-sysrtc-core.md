# Spec: sysrtc moves into the HAL, one copy only (H-pub 6/6)
#: depends=C1 status=- block=-

## Overview

Mesmo movimento do C1, aplicado a `sysrtc`. A fonte passa a viver em
`crates/efr32mg2x-hal/src/core/sysrtc.rs` e a crate em `libraries/sysrtc-core` fica como harness
de teste, a compilar esse mesmo ficheiro por `#[path]`. Uma cópia, dois compiladores: o host corre
os testes, o alvo produz o firmware.

## Requirements

**R1 -- o check cobre 6 cores e continua a apanhar uma cópia.**
`scripts/check-core-split.py` passa a ler os shims de todos os cores movidos.
Exemplo: com `6` cores movidos imprime `OK: 6 cores, one copy each`.

**R2 -- os testes de sysrtc continuam a correr no host, sem hardware.**
Exemplo: `cargo test -p efr32mg2x-sysrtc-core --target x86_64-unknown-linux-gnu` passa.

**R3 -- o HAL continua a compilar no host e no alvo.**
Exemplo: `cargo check -p efr32mg2x-hal --target x86_64-unknown-linux-gnu --no-default-features --features mg24` e
`bash scripts/gate.sh` verdes.

## Files

- `libraries/sysrtc-core/src/lib.rs`
- `crates/efr32mg2x-hal/src/core/sysrtc.rs`
- `crates/efr32mg2x-hal/src/core/mod.rs`
- `crates/efr32mg2x-hal/src/sysrtc.rs`
- `crates/efr32mg2x-hal/Cargo.toml`
- `scripts/check-core-split.py`

## TDD Contract

| Test Name | Input | Expected Output |
|---|---|---|
| the_second_core_is_covered_too | 6 shims, all pointing at existing HAL files | `OK`, count is 6 |
| a_copy_of_the_second_core_is_caught | `sysrtc-core/src/lib.rs` with bodies pasted back | `FAIL`, naming the file |
| no_shim_points_outside_the_tree | a `#[path]` pointing at `/projects/...` | `FAIL` |

## Exit Criteria

- [ ] `python3 scripts/check-core-split.py`
- [ ] `cargo test -q -p efr32mg2x-sysrtc-core --target x86_64-unknown-linux-gnu`
- [ ] `bash scripts/gate.sh`

## Guardrails

- Não copiar o ficheiro: duas cópias é o item falhado, mesmo com os testes verdes.
- Não publicar e não flashar.
- Um core por item.
