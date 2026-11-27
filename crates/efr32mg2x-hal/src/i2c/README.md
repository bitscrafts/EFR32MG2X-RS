# i2c

`embedded-hal-async` I2C: a mesma máquina de estados por byte, à espera da interrupção em vez de girar.

## What it is

`embedded-hal-async` I2C: a mesma máquina de estados por byte, à espera da interrupção em vez de girar.

## Porque é que isto não é só `async` à volta do código que já existia

O `I2c0::write` bloqueante faz `while status().pstart() { polls += 1 }` até um limite. Embrulhar isso num
`async fn` daria uma função que se chama `async`, devolve um futuro, e **bloqueia o executor inteiro** quando
corre — todas as outras tarefas ficavam paradas durante a transferência. É a mentira que este ficheiro existe
para não contar.

O que aqui se faz é esperar de verdade: quando a condição ainda não está satisfeita, a tarefa **regista o seu
waker e devolve `Pending`**, e a interrupção do I2C acorda-a quando o periférico muda de estado. Entre uma
espera e a seguinte o executor corre o que quiser.

## Quem serve a interrupção

O vector é da aplicação, como todos os outros nesta HAL: ela define `I2C0_IRQHandler` e chama
[`on_interrupt`]. A HAL não instala handlers em crates de biblioteca — um handler instalado aqui correria mesmo
num firmware que nunca usasse I2C assíncrono.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Config` | struct | — |
| `Error` | enum | — |
| `I2c0` | struct | I2C0 peripheral instance. |
| `I2c1` | struct | I2C1 peripheral instance. |
| `Speed` | enum | — |
| `fn` | const | Create new I2C configuration with specified speed. |
| `last_state` | fn | O `(STATUS, IF)` da última espera falhada -- a leitura que diz em que estado o barramento ficou. |
| `new` | fn | Creates a new I2C0 instance # Arguments * `i2c` - I2C0_S peripheral instance * `config` - I2C configuration * `hfclk` - the HFCLK frequency the divider is computed from # Example ```no_run let i2c = I2c0::new(dp.i2c0_ns, Config::new(Speed::Standard100kHz), clocks.hfclk()); ```. |
| `on_interrupt` | fn | — |
| `read` | fn | Read data from I2C device # Arguments * `addr` - 7-bit device address * `buffer` - Buffer to store read data # Returns `Ok(())` on success, `Err(Error)` on failure. |
| `waits` | fn | Os contadores, para quem os quiser ler sem `nm`. |
| `write` | fn | Write data to I2C device # Arguments * `addr` - 7-bit device address * `bytes` - Data to write # Returns `Ok(())` on success, `Err(Error)` on failure. |
| `write_read` | fn | Write register then read from I2C device # Arguments * `addr` - 7-bit device address * `register` - Register address to read from * `buffer` - Buffer to store read data # Returns `Ok(())` on success, `Err(Error)` on failure. |

## Registers it touches

`I2c0Ns`, `I2c0S`, `I2c1S`, `Interrupt`, `Peripherals`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

- `i2c0`, `i2c1` | **I2C slave** (address match) | absent as a driver. (COVERAGE-BOUNDARY-2026-11-25.md (`../../../../docs/04-reference/COVERAGE-BOUNDARY-2026-11-25`))
- documentation | **B** | `deny(missing_docs)` now holds on all 27 modules (the five exemptions are gone, and the module that needed them -- `cores::i2c::sim` -- is test-only). (HAL-MATURITY-2026-11-25.md (`../../../../docs/04-reference/HAL-MATURITY-2026-11-25`))
- Wake latency:** if the MCU timer paces each sample, every sample costs an EM2→EM0 wake and an I2C transfer. (HAL-OPUS-ANALYSIS-2026-11-22.md (`../../../../docs/04-reference/HAL-OPUS-ANALYSIS-2026-11-22`))

## Using it

See `examples/hal_i2c_async.rs`, `examples/hal_i2c_hardware_imu.rs`, `examples/hal_i2c_recovery.rs`
