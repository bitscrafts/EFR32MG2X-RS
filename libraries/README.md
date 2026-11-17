# libraries/

Domain and protocol crates: code that decides **what to do**, and knows nothing about how the
hardware does it.

## The rule for entering this folder

A crate may live here only if all three hold. They are checked by `scripts/gate.sh`, not by good
intentions:

1. **No hardware dependency.** `cargo tree` must not contain `efr32mg24-*`, `cortex-m-rt`, or any
   other crate from `crates/`. Not a normal dependency, not a build dependency.
2. **It builds and tests on the host.** `cargo test` on `x86_64-unknown-linux-gnu`, in under a few
   seconds, with no target triple and no linker script.
3. **It is reusable by an application that is not this board.** If the only caller is one example in
   this repository, it belongs in that example, not here.

## Naming, standardised 2026-11-17

```
efr32mg2x-<peripheral>-core     the decision logic of ONE peripheral, measured on the host
                                i2c-core · dma-core · usart-core · spi-core · clock-core · gpio-core · adc-core
```

A `-core` crate exists because the HAL cannot be compiled for the host (it contains ARM `asm!`), so
a defect there could be read and never measured. Each core is the part of a driver that decides
something -- a sequence, a divider, a guard, a flag -- with the register access left to the HAL,
which calls the core rather than reimplementing it.

Domain crates are **not** cores and keep their own names, because they decide about the world rather
than about a peripheral:

```
motion-detection   whether a fall happened, from accelerometer magnitudes        (9 host tests)
aes-modes          AES-CMAC, AES-CTR, AES-ECB over a caller-supplied cipher     (7 host tests)
aes-ctr-radio      the counter convention the Bluetooth stack expects           (4 host tests)
bt-heap            first-fit heap for the stack, with canaries and a ring       (12 host tests)
```

## What is here

| crate | what it decides | measured by |
|---|---|---|
| `motion-detection` | free fall, then a landing, then quiet -- from magnitudes and the interval between them | 9 tests, constants from nine recorded drops |
| `aes-modes` | CMAC/CTR/ECB over a block cipher the caller supplies | 7 tests, RFC 4493 vectors |
| `aes-ctr-radio` | the counter convention for radio payloads | 4 tests |
| `bt-heap` | first-fit allocation for the stack, with canaries | 12 tests |
| `i2c-core` | a transaction: repeated START, address phases, ACK/NACK | 5 tests + `examples/simulate` |
| `dma-core` | who owns the buffer while the channel writes | 4 tests + `examples/simulate` |
| `usart-core` | the baud divider, in 64 bits, with the field width enforced | 6 tests + `examples/divider` |
| `spi-core` | the synchronous divider (`CFG2.SDIV`) and what "finished" means (`TXIDLE`, not `TXFL`) | 6 tests + `examples/flush` |

## What belongs here next, by the same rule

```
clock-core    which band, which wait states, which PCLK and radio prescaler for a target frequency
gpio-core     the pin model fields, and a read-modify-write that touches one pin
adc-core      the port/pin encoding for a conversion, and the analogue-bus allocation
timer-core    prescaler and top for a wanted period
sysrtc-core   deadline arithmetic across the 32-bit wrap
wdog-core     timeout encoding, and the lock sequence order
```

Each of them is arithmetic or sequence, and each has at least one defect in the review that a host
test would have caught: H10 and H21/H22 live in the clock, H17 in the GPIO, H13 in the ADC, H25 and
H26 in the SYSRTC, H27 in the watchdog.
