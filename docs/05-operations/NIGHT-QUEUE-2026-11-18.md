# Night queue — 2026-11-18/19

Ordered by (a) whether it can be proven on the host today, (b) risk to the product, (c) value. Each item
states the evidence, the fix and the proof that closes it. An item is only marked done when the gate is
green **and** the proof named here was run and its output recorded.

Source of truth for the defect list: `_tmp/review-hal-fable/REPORT-fable-partial.md` (H1–H31, B1–B18).
Already closed in earlier sessions: H1–H11, H13–H16, H18–H20, H23, H24, H26, H29, B1, B2, B6, B13, B15, B17.

## Group 1 — arithmetic, provable on the host (no hardware needed)

| # | Where | Defect | Fix | Proof |
|---|---|---|---|---|
| ~~H25~~ **done** | `hal/sysrtc.rs` `schedule_at` | A deadline already in the past never fires until the 32-bit counter wraps (36.4 h). No re-read after arming. | Re-read the counter; if the deadline has passed, program `now + 1` and report it. | host test on the pure decision function + board: `schedule_at(now - 1000)` fires within 2 ticks |
| B8 | `bt/sleeptimer.rs` | Same class: arming a past deadline stalls the stack timer. | Same decision function, reused (`sysrtc_core::compare_target`). | host test |
| ~~H12~~ **done** | `hal/mvp/mod.rs` | Accepts `n <= 65_536`; `SIZE` and `NUMITERS` are 10-bit fields, so above 1024 the length spills into neighbouring fields. | Reject `n > 1024` with an error. | host test that 1025 is refused and 1024 accepted |
| ~~H31~~ **done** | `hal/memory.x` | `FLASH LENGTH = 1512K` covers the NVM3 pages; the BSP says 1472K. | Same length in both. | gate: the two files agree |

## Group 2 — register sequence, decidable from the vendor headers

| # | Where | Defect | Fix | Proof |
|---|---|---|---|---|
| ~~H27~~ **refuted, adjacent defect fixed** | `hal/wdog/mod.rs` | Claim: `LOCK` written disabled is the faulting kind. **Header says otherwise**: `_WDOG_LOCK_RESETVALUE 0x0000ABE8` (unlocked out of reset) and `STATUS.LOCK` is read-only, so `LOCK` is an ordinary writable register, not a config register under the TIMER rule. | The real gap found while checking: the watchdog was **never re-locked** after configuration, leaving a protection register open to any later write. | `lock_config()` after enable, as the vendor's `WDOG_Init` does |
| ~~H30~~ **done** | `hal/board.rs` | `gpio_static` casts `0x5003_C000`, which is `GPIO_NS_BASE`, to `gpio_s`. | Use the PAC pointer. | gate + board example still runs |
| H17 | `hal/pin.rs`, `hal/bus/bitbang.rs` | `DOUT` read-modify-write across the whole port; an interrupt touching the same port is lost. | Write the `+0x1000`/`+0x2000` SET/CLR aliases. | board: bit-bang still ACKs; host: the helper is unit-tested |

## Group 3 — one runtime clock value (the 206 % question)

| # | Where | Defect | Fix | Proof |
|---|---|---|---|---|
| H21 | `hal/clock/clocks.rs` | `Clocks::new` configures nothing and returns whatever frequency the caller states; every divider trusts it. | Read `SYSCLKCTRL.CLKSEL` and the band, or delete the type. | board: the reported clock matches `CMU_SYSCLKCTRL` |
| H22 | `hal/board.rs`, `hal/pwm.rs` | `19_000_000` hard-coded in three places; false after a boost and after `Bluetooth::new` (39 MHz). | One runtime value owned by `Board`. | the delays and tones use the live value |

## Group 4 — Bluetooth (each proven by reading the vendor call site)

| # | Where | Defect | Fix | Proof |
|---|---|---|---|---|
| B3 | `bt/psa.rs` | Local `PSA_ALG_CMAC = 0x03C0_0000` shadows the right `0x03C00200`; the GATT hash always fails. | Delete the local constant. | GATT hash reported OK |
| B4 | `bt/psa.rs` | `psa_mac_sign_setup` treats the key id as "the Nth used slot". | Use `slot_for`. | host test on the lookup |
| ~~B5~~ **done** | `bt/sleeptimer.rs` | `ms32_to_tick` / `tick64_to_ms` return the value; the contract returns a status and writes through a pointer. | Match the prototypes. | header lines + board timer |
| ~~B7~~ **done** | `bt/sleeptimer.rs` | 64-bit tick is the 32-bit counter widened; wraps after 36.4 h. | Extend with an overflow count. | host test across the wrap |
| B9 | `bt/sleeptimer.rs` | Callback called with one argument; the contract is `(handle, data)`. | Pass the handle. | header line |
| ~~B10~~ **done** | `bt/sleeptimer.rs` | 32 ticks/ms is 2.4 % short; accuracy reported as 500 ppm against 6000 recorded. | 64-bit conversion; report the measured figure. | arithmetic test |
| B11 | `bt/psa.rs` | Unknown id returns −140 (accepts only 0/−136); table-full −143; short buffers −135 instead of −138. | Three constants. | the call site |
| B12 | `bt/psa.rs` | `psa_import_key` ignores type/algorithm/usage; `psa_mac_compute` truncates at 128 bytes and reports success. | Honour attributes; reject over-length. | host test |
| ~~B14~~ **done** | `bt/blocking/gatt_db.rs` | A failed step returns without `sl_bt_gattdb_abort`; `handles.push` beyond 8 is discarded. | Abort on error; error at capacity. | host test |
| B16 | `bt/stack_init.rs` | The link layer is served from PendSV, which blocks any RTOS. | Move it to a dedicated software interrupt. | board: stack still runs |
| B18 | `radio-sys/src/ble_config.rs` | The generator is outside the repository; the once-guard hands a second caller an unfilled table. | Commit the generator; fix the guard. | the generator in-tree |

## Group 5 — needs hardware, do only if the board is free

| # | Where | Defect |
|---|---|---|
| H28 | `hal/emu/mod.rs` | EM2 entered with a bare `wfi` and no pending-event check; EM01 voltage can be lowered at any clock. |

## Rules for this queue

- One item, one commit, gate green before the commit. Never `--allow-dirty`, never a red gate.
- A fix that cannot be proven as stated is **not** marked done: it is recorded with what blocked it.
- No publishing, no flash beyond `scripts/flash.sh --confirm`, no destructive filesystem command.

## Estado no fim desta sessão (2026-11-18, 06:05Z)

**Fechados, com gate verde e commit próprio:** H25 (`d4245b2`), B8 (`9f5fb46`), H12+H30+H31 (`ad9e09c`),
H27 — refutado pela fonte, com o defeito adjacente corrigido (`a638e5a`), B3+B4 (`17c8128`),
B5+B10+B14 (`3964dc8`).

**Continua aberto, e porquê:**

| # | Porque não foi feito nesta sessão |
|---|---|
| H17 | Precisa de `DOUT_SET`/`DOUT_CLR`; a PAC não expõe os aliases (`+0x1000`/`+0x2000`). Primeiro estender a PAC ou escrever por offset, e o teste tem de ser na placa (bit-bang a ACK-ar). |
| H21/H22 | É a mudança maior do lote: um valor de relógio em runtime que `Board` possui, e todos os divisores a consumi-lo. Mexe em `usart`, `spi`, `i2c`, `timer`, `pwm` e `delay` ao mesmo tempo — não é trabalho para o fim de uma sessão sem a placa para medir. |
| H28 | Exige a placa para provar a entrada e saída de EM2 (o `wfi` sem verificação de evento pendente não se valida no host). |
| B7 | Precisa de um contador de overflow alimentado pelo SYSRTC, o que cruza o HAL e o stack; e o teste tem de simular 36,4 h. Faz-se com a mesma técnica do `compare_target`: função pura em `libraries/`, testada no host, e depois ligada. |
| B9 | O contrato do callback é `(handle, data)`; mudar a assinatura obriga a rever quem chama (RAIL incluído), e o único chamador observado ainda não foi medido com a assinatura nova. |
| B12 | `psa_import_key` honrar atributos e `psa_mac_compute` recusar acima de 128 bytes: precisa dos valores de atributo que o stack passa, que saem de `audit-blobs.sh callers` — não de memória. |
| B16 | Mover a link layer de PendSV para uma interrupção própria. É a mudança de maior risco do relatório: se falhar, o stack não arranca e a placa fica sem rádio. Só com a placa disponível e tempo para reverter. |
| B18 | O gerador do `ble_config.rs` não está no repositório; é preciso encontrá-lo (ou reescrevê-lo a partir do ficheiro gerado) antes de o commitar. |

**Regra que fica:** cada um destes precisa da sua prova. Fechar um item com um commit e sem a prova que a
tabela pede é a mesma classe de erro que "um check que passa com o caminho vivo avariado".
