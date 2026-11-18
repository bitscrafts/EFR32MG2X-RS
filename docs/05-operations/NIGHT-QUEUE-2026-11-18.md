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
| ~~B9~~ **done** | `bt/sleeptimer.rs` | Callback called with one argument; the contract is `(handle, data)`. | Pass the handle. | header line |
| ~~B10~~ **done** | `bt/sleeptimer.rs` | 32 ticks/ms is 2.4 % short; accuracy reported as 500 ppm against 6000 recorded. | 64-bit conversion; report the measured figure. | arithmetic test |
| B11 | `bt/psa.rs` | Unknown id returns −140 (accepts only 0/−136); table-full −143; short buffers −135 instead of −138. | Three constants. | the call site |
| ~~B12~~ **parcial** | `bt/psa.rs` | `psa_import_key` ignores type/algorithm/usage; `psa_mac_compute` truncates at 128 bytes and reports success. | Honour attributes; reject over-length. | host test |
| ~~B14~~ **done** | `bt/blocking/gatt_db.rs` | A failed step returns without `sl_bt_gattdb_abort`; `handles.push` beyond 8 is discarded. | Abort on error; error at capacity. | host test |
| B16 | `bt/stack_init.rs` | The link layer is served from PendSV, which blocks any RTOS. | Move it to a dedicated software interrupt. | board: stack still runs |
| ~~B18~~ **done** | `radio-sys/src/ble_config.rs` | The generator is outside the repository; the once-guard hands a second caller an unfilled table. | Commit the generator; fix the guard. | the generator in-tree |

## Group 5 — needs hardware, do only if the board is free

| # | Where | Defect |
|---|---|---|
| H28 | `hal/emu/mod.rs` | EM2 entered with a bare `wfi` and no pending-event check; EM01 voltage can be lowered at any clock. |

## Rules for this queue

- One item, one commit, gate green before the commit. Never `--allow-dirty`, never a red gate.
- A fix that cannot be proven as stated is **not** marked done: it is recorded with what blocked it.
- No publishing, no flash beyond `scripts/flash.sh --confirm`, no destructive filesystem command.

## Estado no fim desta sessão (2026-11-18, 12:20Z)

**Fechados, com gate verde e commit próprio — dezasseis itens:**

| Item | Commit | O que mudou |
|---|---|---|
| H25 | `d4245b2` | deadline no passado dispara em vez de esperar 36,4 h; 5 testes no host |
| B8 | `9f5fb46` | a mesma decisão, partilhada com o stack (`sysrtc_core::compare_target`) |
| H12 | `ad9e09c` | MVP recusa acima de 1024 (os campos são de 10 bits) |
| H30 | `ad9e09c` | `gpio_static` usa o ponteiro da PAC em vez de um endereço NS |
| H31 | `ad9e09c` | `memory.x` do HAL e do BSP concordam em 1472K |
| H27 | `a638e5a` | **refutado pela fonte** (`LOCK` reseta em `0xABE8`), e o defeito real ao lado: o watchdog nunca era re-bloqueado |
| B3 | `17c8128` | `PSA_ALG_CMAC` local sombreava o valor da especificação |
| B4 | `17c8128` | id de chave pelo id guardado, não pela "N-ésima usada" |
| B5 | `3964dc8` | protótipos do timer devolvem status e escrevem por ponteiro |
| B10 | `3964dc8` | conversão exacta ms↔ticks (32768/1000), não 32 |
| B14 | `3964dc8` | sessão GATT abortada no erro; handle em excesso é erro |
| B7 | `5d798ba` | tick de 64 bits conta as voltas (`WideTick`, 5 testes) |
| B9 | `d357c68` | callback do timer recebe `(handle, data)` como o contrato |
| B12 | `a7c0ccb` | **parcial**: MAC sobre mensagem truncada é recusado |
| B18 | `7db09c6` + `e215335` | o gerador do `ble_config.rs` passou para `scripts/`, e o gate regenera e compara |

**Continua aberto, e a razão é a mesma de antes:**

| # | Porque não foi feito |
|---|---|
| H17 | Os aliases `+0x1000`/`+0x2000` existem, mas a prova exigida é o bit-bang a receber ACK de um dispositivo real. **Verificado esta noite: não há exemplo de bit-bang no BSP nem dispositivo conhecido identificado na bancada** — um teste assim falharia pela sua própria razão. Precisa de (a) um exemplo que fale com o OLED ou o PCF8563 e (b) a placa. |
| H21/H22 | Um valor de relógio em runtime que `Board` possui: mexe em `usart`, `spi`, `i2c`, `timer`, `pwm` e `delay` ao mesmo tempo. |
| H28 | Entrada e saída de EM2 só se provam na placa. |
| B12 (resto) | `psa_import_key` honrar os atributos exige os valores que o stack passa, de `audit-blobs.sh callers` — não de memória. |
| B16 | Mover a link layer de PendSV: o maior risco do relatório; sem tempo de reverter, não se toca. |
| B18 | O gerador do `ble_config.rs` não está no repositório. |

**O padrão que se confirmou esta noite:** três vezes o defeito estava *ao lado* do que o relatório apontava
(H27: a premissa era falsa, o watchdog sem re-lock era real; B3/B4: duas constantes na mesma função; B5: o
exemplo que compilava provava que o protótipo estava errado). Ler o sítio antes de aceitar a acusação pagou
três vezes em catorze.
