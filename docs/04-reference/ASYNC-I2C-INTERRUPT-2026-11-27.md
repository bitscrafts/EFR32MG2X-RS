# The async I2C wait sleeps, and the interrupt costs 6 entries per transaction — 2026-11-27

## The defect, in the file's own words

`crates/efr32mg2x-hal/src/i2c/async_impl.rs` carried this, dated from the first attempt:

> A primeira versão deste ficheiro armava `IEN` para `txc`/`rxdatav`/`nack`/`mstop` e acordava a tarefa na interrupção.
> Medido na placa: **44 647 591** entradas no handler em poucos segundos (`ICSR = 0x82b`, vector 43), porque neste
> periférico o flag de `IF` volta a afirmar-se enquanto a condição for verdadeira — limpar não o desarma.

The conclusion drawn then was to give up on the interrupt and self-wake (`cx.waker().wake_by_ref()`), which meant the
core never slept during a transfer. The note also said what the fix had to be: *armar um bit de cada vez, e não todos
de uma vez*. That is what is now done.

## What changed

- **One `IEN` bit per wait.** `Ate` carries the mask of the single event it is waiting for — `START` for the address
  phase, `TXBL` and `TXC` per byte, `RXDATAV` per received byte, `MSTOP` for the stop — instead of arming four at once.
- **`on_interrupt()` disarms before it clears.** It reads `IF`, writes zero to `IEN`, then writes the pending bits to
  the `_CLR` window. The order is the whole fix: with `IEN` still armed and the condition still true, the vector fires
  again the instant the ISR returns. That is what 44 647 591 entries were.
- **No `wake_by_ref()`.** The future returns `Pending` and the NVIC is unmasked; the interrupt wakes the executor.
- The vector stays where this crate keeps every vector: **the application's**. `hal_i2c_async` defines
  `I2C0_IRQHandler`, calls `i2c::on_interrupt()`, counts the entry and wakes its task.

## The record

Three flashes of the same image, three boots, `status=PASS` in all three, byte-identical:

```
i2c-async: who=0x6a pending=23 irq=12 wfi=12
```

Two transactions in the same image (one after the other), each a `WHO_AM_I` register read through
`embedded_hal_async::i2c::I2c`:

| field | meaning |
|---|---|
| `who=0x6a` | both reads returned the IMU's `WHO_AM_I` — the async path moves real bytes |
| `irq=12` | twelve entries in `I2C0_IRQHandler` for two transactions: **six per transaction**, against 44 647 591 before |
| `wfi=12` | twelve sleeps: one per interrupt, i.e. the core was asleep and the interrupt is what woke it |
| `pending=23` | polls that returned `Pending`; the five that did not sleep are the ones that arrived while the previous wake was still being handled |

Controls, both inside the same run:

- **The count scales with the work.** `irq` and `wfi` are compared *before and after* the second transaction and the
  record is only a pass if both grew. A counter stuck at a constant would satisfy the first transaction and fail this.
- **The blocking path is the reference.** The same read through `I2c0::write_read` is already proven elsewhere
  (`hal_i2c_hardware_imu` returns `0x6a`), so a failure here is not the bus.

## What this does not prove

- **Latency.** Nothing here times the transfer, so the claim is "the core sleeps and one interrupt per event wakes it",
  not "the transfer takes N microseconds".
- **Other tasks.** The executor in the example is fifteen lines with `wfi` and a flag-sized waker. A real executor
  behaves the same way for this future, but this instrument does not run two tasks at once.
- **A failed transfer.** The error paths (`Nack`, `Buserr`) are unchanged and the NACK check still reads the secure
  alias; this run does not exercise one.

## Why it took a second visit

The first attempt failed for a reason that looked like a driver bug and was a register property: on this block `IF`
self-asserts while the condition holds, so "clear the flag" is not "handle the event". The old note recorded the
measurement and the wrong conclusion together, and the wrong conclusion — self-wake — was cheap enough to look like a
design decision. It was a retreat, and it left the core awake for the length of every transfer.
