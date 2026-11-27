# record

A record in RAM, written the same way by every instrument.

## What it is

A record in RAM, written the same way by every instrument.

## Why this is a module and not twenty copies

Most examples in this crate end by writing a line into a mailbox that a host reads back over the debugger, and every
one of them used to carry its own formatter: the same twenty lines of `s`/`u`/`h`/`buf`, copied, with small
differences — one truncated silently, one printed `u64` as `u32`, one had no hex at all. A record is evidence, and
evidence formatted differently in each instrument is evidence nobody can compare across instruments.

## What it guarantees

- **It never panics and never writes out of bounds.** An over-long record is truncated, and the record says so: the
buffer is never left with a half-written number that reads as a different number.
- **Numbers are exact or absent.** `u` prints every digit of a `u64`; `i` handles `i64::MIN` without overflow; `h`
prints two digits, zero-padded, because a byte compares better as `0a` than as `a`.
- **No allocation, no formatting machinery.** Fixed buffer, `no_std`, and no `core::fmt` — the point is to be usable
in a panic path and to cost the same everywhere.

The arithmetic is host-tested through `libraries/record-core`, which compiles this file by path.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Record` | struct | A line of evidence in a fixed buffer. |
| `bit` | fn | Append `1` or `0`, the shape a record uses for a verdict field. |
| `bytes` | fn | — |
| `clear` | fn | Empty the record and forget the truncation. |
| `fn` | const | — |
| `h` | fn | Append a byte as two hexadecimal digits. |
| `h32` | fn | Append a 32-bit number as eight hexadecimal digits, most significant first. |
| `i` | fn | Append a signed number. |
| `is_empty` | fn | — |
| `len` | fn | — |
| `s` | fn | Append text. |
| `truncated` | fn | — |
| `u` | fn | Append an unsigned number, every digit of it. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

- record. (STATUS.md (`../../../../docs/03-development/STATUS`))
- `libraries/` | `no_std` logic with no hardware: `bt-heap`, `aes-modes`, `aes-ctr-radio`, `motion-detection`, `record-core` | host-tested, never published |. (STATUS.md (`../../../../docs/03-development/STATUS`))
- documentation | **B** | `deny(missing_docs)` now holds on all 27 modules (the five exemptions are gone, and the module that needed them -- `cores::i2c::sim` -- is test-only). (HAL-MATURITY-2026-11-25.md (`../../../../docs/04-reference/HAL-MATURITY-2026-11-25`))

## Using it

See `examples/hal_acmp_ladder.rs`, `examples/hal_adc_prs_gpio.rs`, `examples/hal_adc_prs_timer.rs`, `examples/hal_adc_prs_trigger.rs`, `examples/hal_adc_read.rs`, `examples/hal_aes_kat.rs`, `examples/hal_burtc_backup.rs`, `examples/hal_cmu_sources.rs`, `examples/hal_delay_clock.rs`, `examples/hal_eusart_loopback.rs`, `examples/hal_gpcrc.rs`, `examples/hal_gpio_interrupt.rs`, `examples/hal_hfxo.rs`, `examples/hal_letimer_compare.rs`, `examples/hal_mvp_bench.rs`, `examples/hal_one_of_each.rs`, `examples/hal_pcnt_gpio.rs`, `examples/hal_spi_loopback.rs`, `examples/hal_timer0_repacing.rs`, `examples/hal_timer_compare.rs`, `examples/hal_timer_instances.rs`, `examples/hal_timer_ticks.rs`, `examples/hal_usart_link.rs`, `examples/hal_usart_rx_recovery.rs`
