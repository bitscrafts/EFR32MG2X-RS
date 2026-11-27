# dma

Interrupt-driven LDMA completion -- [C2-DMA.4].

## What it is

Interrupt-driven LDMA completion -- [C2-DMA.4].

[`Channel::transfer`] and [`Channel::transfer_chain`] finish by reading `IF` in a loop: the core spins while the
controller works. This module adds the other way to end a transfer -- the controller requests `LDMA_IRQ` when a
descriptor's count runs out, the handler acknowledges the flag and records the channel, and the caller sleeps in
`WFI` until the record appears. Between arming and completion the core executes no loop at all.

# Three bits, and which one this item measures

A done interrupt reaches the core only if three things line up:

| where | bit | who sets it here |
|---|---|---|
| descriptor `CTRL` | `DONEIEN`, bit 20 | `start` always sets it |
| controller `IEN` | `CHDONE[channel]` | [`Gate`]: the one thing the two configurations differ in |
| core `NVIC` | the `LDMA_IRQ` vector | `start` unmasks it in both configurations |

The control for this item is therefore one bit wide: same descriptor, same vector, same data, `IEN.CHDONE`
cleared. If the flag still latches and nothing wakes, the wake came from the gate and not from the transfer
finishing. If instead the deadline is what ends the wait, the wait cannot be completed by polling either --
it has no poll path.

# Who installs the handler

This HAL does not define `LDMA_IRQHandler`, for the same reason it does not define the two GPIO vectors
([`crate::extint`]): a library that takes a vector from every application is a library the application cannot
configure. The handler is one call:

```ignore
#[no_mangle]
pub extern "C" fn LDMA_IRQHandler() {
efr32mg2x_hal::dma::irq::service();
}
```

A handler that does not call [`service`] leaves the channel's flag set and the vector re-enters for ever: `IF`
is a level, not a pulse, and this part's `IF` is cleared through the `+0x2000` alias, not by writing 1 to the
plain register. [`service`] clears every bit it reads, which is the whole reason the acknowledgement lives here
and not in a caller's macro.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `ChainMode` | enum | — |
| `Channel` | struct | DMA Channel (Type-Safe) Represents a single DMA channel with compile-time channel number validation. |
| `ChannelState` | struct | — |
| `Dma` | struct | DMA Controller Owns the LDMA peripheral and provides access to all 8 DMA channels. |
| `DmaError` | enum | — |
| `Gate` | enum | — |
| `Pending` | struct | A transfer that is running, and whose destination buffer this value owns until it ends. |
| `PeripheralRequest` | enum | — |
| `Segment` | struct | One leg of a chained transfer: the `src` elements copied into `dst`, and nothing else. |
| `TransferElement` | trait | Trait for types that can be transferred via DMA This trait defines types that have a valid DMA transfer size. |
| `TransferSize` | enum | — |
| `Wait` | struct | — |
| `buffer` | fn | The destination the controller is filling. |
| `channel0` | fn | Returns a mutable reference to Channel 0 # Example ```rust,no_run let mut dma = Dma::new(dp.ldma_s); let mut ch0 = dma.channel0(); ```. |
| `channel_busy` | fn | True while the controller still has `channel` enabled, without needing the handle. |
| `channel_enabled` | fn | True while the controller still has `channel` enabled, without needing the handle. |
| `channel_state` | fn | Read back what a channel is currently holding. |
| `completed` | fn | — |
| `entries` | fn | — |
| `flag` | fn | — |
| `gate` | fn | — |
| `get_position` | fn | Get current DMA write position in circular buffer Returns the index of the next buffer element that will be written. |
| `is_busy` | fn | True while the controller still has this channel enabled. |
| `is_enabled` | fn | True while the controller still has this channel enabled. |
| `new` | fn | A leg that copies `src` into `dst`. |
| `reset` | fn | Drop the record and the hardware flag for `channel`, leaving `IEN` as it is. |
| `service` | fn | Serve the vector: acknowledge every flag it reports and record the channels that finished. |
| `start_interrupt` | fn | Start a memory-to-memory transfer that ends with an interrupt, without waiting for it. |
| `start_peripheral_to_memory` | fn | Start continuous peripheral-to-memory transfer with circular buffer Configures the DMA channel for continuous transfers from a peripheral to a circular buffer in memory. |
| `stop` | fn | Stop continuous DMA transfer Disables the DMA channel, stopping any ongoing transfers. |
| `take` | fn | Creates a new DMA controller Takes ownership of the LDMA peripheral and initializes it. |
| `transfer` | fn | Performs a blocking memory-to-memory transfer Copies data from `src` to `dst` using DMA. |
| `transfer_chain` | fn | Copy each segment in order, with the descriptors chained through the controller's own `LINK`. |
| `trigger` | fn | Ask the controller to service `channel` once, without a peripheral request. |
| `wait` | fn | Sleep until the handler records `channel`, or until `expired` says the window is over. |

## Registers it touches

`CmuS`, `Interrupt`, `LdmaS`, `LdmaxbarS`, `Peripherals`

## What was measured

```
    dmairq: ok=1 n=2047 am=2047 irq=1 sp=1 tk=0 g=1 ad=1 af=0 cm=2047 cirq=0 csp=6 ctk=6 cg=0 cd=0 cf=1
    dmalink: ok=1 d1=8 d2=8 s2=1 lk=0x0 cto=1 c1=8 c2=0 c1r=1 clk=0x0 keep=0xdeadbeef ctl=0x81f0008 ssrc=0x2003ee7c sdst=0x2003eebc
```

Controls and the full story: [DMA-IRQ-2026-11-27.md](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/DMA-IRQ-2026-11-27.md), [DMA-LINKED-2026-11-27.md](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/DMA-LINKED-2026-11-27.md)

## What is not proven

- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))
- ⏳ Continuous conversion and multi-channel scan (deferred; the microphone path uses LDMA instead). (PLAN.md (`../../../../docs/03-development/PLAN`))
- Do NOT delete the deferred design: Zigbee and Thread stay on the roadmap, and this item exists so. (QUEUE-ITEM-B10-ieee802154-sys.md (`../../../../docs/05-operations/QUEUE-ITEM-B10-ieee802154-sys`))

## Using it

See `examples/hal_dma_irq.rs`, `examples/hal_dma_link.rs`, `examples/hal_dma_stop.rs`
