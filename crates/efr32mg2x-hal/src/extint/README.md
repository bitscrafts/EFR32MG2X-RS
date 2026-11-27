# extint

GPIO external interrupts, delivered to the core instead of to the PRS.

## What it is

GPIO external interrupts, delivered to the core instead of to the PRS.

[`crate::pin::EdgeWatch`] configures the edge detector and its sticky flag. Until now the only consumer this crate
could offer was the PRS, which takes the pad's signal without the CPU (measured 2026-11-26). This module adds the
other consumer: the NVIC, with a counter the handler keeps, so a driver can wait for an edge instead of polling.

## The two vectors, and who installs them

The part has one vector per channel parity: `GPIO_ODD_IRQHandler` for channels 1, 3, 5 ... and
`GPIO_EVEN_IRQHandler` for 0, 2, 4 ... This HAL does **not** define them. A library that takes two of the part's
vectors from every application is a library the application cannot configure, and the same crate already keeps the
SYSRTC handler in the application for the same reason. Both handlers are one call:

```ignore
#[no_mangle]
pub extern "C" fn GPIO_ODD_IRQHandler() {
efr32mg2x_hal::extint::service(Parity::Odd);
}

#[no_mangle]
pub extern "C" fn GPIO_EVEN_IRQHandler() {
efr32mg2x_hal::extint::service(Parity::Even);
}
```

A handler that does not call [`service`] leaves the channel's flag set and the vector re-enters for ever: the line
is asserted while any flag of the parity is set. [`service`] acknowledges every flag of its parity and counts only
the channels an [`EdgeCounter`] has armed, so a channel nobody is counting costs one read and one write, not a
storm.

## Why the flag is cleared before the counters move

`IF` is a level that survives the handler, so the acknowledgement comes first and the count after it. An edge that
arrives inside that window is lost from the count but is not lost to the vector — it sets the flag again and the
handler runs a second time. The other order would count the same flag twice.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `EdgeCounter` | struct | One external-interrupt channel, counted in the handler. |
| `Parity` | enum | — |
| `arm` | fn | Start counting: zero the counter, clear the flag, enable the pin's interrupt and unmask its vector. |
| `channel` | fn | — |
| `count` | fn | — |
| `disarm` | fn | Stop counting, and stop the pin from requesting an interrupt at all. |
| `entries` | fn | — |
| `flags` | fn | — |
| `fn` | const | — |
| `interrupt` | fn | — |
| `is_armed` | fn | — |
| `mask_vector` | fn | Stop the core from entering the channel's vector, leaving the detector armed. |
| `parity` | fn | — |
| `reset` | fn | Zero the counter and drop any flag the channel had latched. |
| `selection` | fn | — |
| `service` | fn | Serve one vector: acknowledge the pending channels of `parity` and count the armed ones. |
| `set_edges` | fn | Arm or disarm the edge masks without moving the channel's binding. |
| `unmask_vector` | fn | Let the core enter the channel's vector again. |

## Registers it touches

`Interrupt`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

See `examples/hal_gpio_interrupt.rs`
