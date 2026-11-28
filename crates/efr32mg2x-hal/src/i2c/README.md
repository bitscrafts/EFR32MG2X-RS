# i2c

`embedded-hal-async` I2C: the same byte-by-byte state machine, waiting on the interrupt instead of spinning.

## What it is

`embedded-hal-async` I2C: the same byte-by-byte state machine, waiting on the interrupt instead of spinning.

## Why this is not just `async` wrapped around the code that already existed

The blocking `I2c0::write` does `while status().pstart() { polls += 1 }` up to a bound. Wrapping that in an `async
fn` gives a function called `async` that returns a future and **blocks the whole executor** while it runs -- every
other task stops for the duration of the transfer. That is the lie this file exists to avoid telling.

What happens here is real waiting: when the condition is not yet satisfied the task **records its waker and
returns `Pending`**, and the I2C interrupt wakes it when the peripheral changes state. Between one wait and the
next the executor runs whatever it likes.

## Who serves the interrupt

The vector belongs to the application, like every other one in this HAL: it defines `I2C0_IRQHandler` and calls
[`on_interrupt`]. The HAL does not install handlers in library crates -- a handler installed here would run in a
firmware that never used asynchronous I2C at all.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Config` | struct | — |
| `Error` | enum | — |
| `I2c0` | struct | I2C0 peripheral instance. |
| `I2c1` | struct | I2C1 peripheral instance. |
| `Speed` | enum | — |
| `fn` | const | Create new I2C configuration with specified speed. |
| `last_state` | fn | The `(STATUS, IF)` of the last failed wait -- the reading that says what state the bus was left in. |
| `new` | fn | Creates a new I2C0 instance # Arguments * `i2c` - I2C0_S peripheral instance * `config` - I2C configuration * `hfclk` - the HFCLK frequency the divider is computed from # Example ```no_run let i2c = I2c0::new(dp.i2c0_ns, Config::new(Speed::Standard100kHz), clocks.hfclk()); ```. |
| `on_interrupt` | fn | — |
| `read` | fn | Read data from I2C device # Arguments * `addr` - 7-bit device address * `buffer` - Buffer to store read data # Returns `Ok(())` on success, `Err(Error)` on failure. |
| `waits` | fn | The counters, for a caller who wants them without `nm`. |
| `write` | fn | Write data to I2C device # Arguments * `addr` - 7-bit device address * `bytes` - Data to write # Returns `Ok(())` on success, `Err(Error)` on failure. |
| `write_read` | fn | Write register then read from I2C device # Arguments * `addr` - 7-bit device address * `register` - Register address to read from * `buffer` - Buffer to store read data # Returns `Ok(())` on success, `Err(Error)` on failure. |

## Registers it touches

`I2c0Ns`, `I2c0S`, `I2c1S`, `Interrupt`, `Peripherals`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

- Wake latency:** if the MCU timer paces each sample, every sample costs an EM2→EM0 wake and an I2C transfer. (HAL-OPUS-ANALYSIS-2026-11-22.md (`../../../../docs/04-reference/HAL-OPUS-ANALYSIS-2026-11-22`))
- The 2026-11-22 review's item 2 was "no hardware I2C path sends STOP or ABORT after an error, so one NACK can leave the. (I2C-RECOVERY-STATE.md (`../../../../docs/04-reference/I2C-RECOVERY-STATE`))

## Using it

See `examples/hal_i2c_async.rs`, `examples/hal_i2c_hardware_imu.rs`, `examples/hal_i2c_recovery.rs`, `examples/hal_one_of_each.rs`
