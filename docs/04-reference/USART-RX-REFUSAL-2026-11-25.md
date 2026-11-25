# The receiver that reported bytes nobody sent

Measured 2026-11-25 on the XIAO MG24, over three attempts. This is the closure of the worst instrument defect in this
crate: a link that reported 271 bytes with nothing connected to it.

## What it said before

```
PASS usart: echo n=271 last=00
```

`n=271` was the number of bytes the driver believed it had received on a pad with **nothing on it**. A floating line
produces start bits out of noise; each one sets `RXDATAV`; `read_byte` returned `Some(byte)` for every one of them, and the
example printed PASS because it had no way to know the bytes did not exist.

## Attempt one, and why it was wrong

The first fix refused any byte whose `IF` flags said the frame was broken. The classification was correct arithmetic and
its host tests were green — with the defect flipped back on, two of them go red. On the part it produced:

```
usart: selftest n=6 ok=0 bad=6 rxerr=7 pre=1f cleared=1f post=1f
```

Six good bytes, all refused, after one earlier overrun. The flags in `IF` are **level latches**, and this is the list of
mechanisms measured against them, each one leaving `RXOF` at `1f`:

| mechanism | result |
|---|---|
| drain the FIFO completely (3 bytes) | bit 2 clears, `RXOF` stays |
| write 1 to the set bits, secure window | unchanged |
| write 1 to the set bits, non-secure window (`USART0_NS`) | unchanged |
| write the whole register as zero | unchanged |
| `EN` off, `EN` on | unchanged |
| `CMD.CLEARRX` | unchanged |

A driver that refused on those flags therefore refused every byte for the rest of the program's life. That table is why
`clear_rx_flags`, `clear_rx_flags_ns`, `reset_receiver` and the write-zero variant existed for a day and are gone: a public
method that does not work is a trap, and the measurements belong in this file instead.

## Attempt two, which is the answer

The per-byte verdict was never in `IF`. It is in **`RXDATAX`**: bits 0-8 are the byte, bit 14 that byte's parity error and
bit 15 its framing error. Reading `RXDATA` popped the byte and threw the verdict away; reading `IF` looked at a latch that
describes the receiver, not the frame.

`read_byte` reads `RXDATAX` now and returns `Option<Result<u8, RxProblem>>`: `None` is nothing arrived, `Some(Err(..))` is a
byte that arrived and whose own frame was broken. `IF.RXOF` is reported separately by `overran()` — "an overrun happened" —
and never as a verdict about a byte.

## The records

Nothing on the RX pad, measured while the `IF`-based path was still in place (the defect's own scenario):

```
usart: echo n=0 rxerr=524 why=ferr
```

Zero bytes accepted, 524 refused and still counting: the 271 phantom bytes are gone, and the counter keeps moving, so the
receiver did not stall.

Paced internal loopback (`CTRL.LOOPBK`, one byte at a time) on a fresh part:

```
usart: selftest n=6 ok=6 bad=0 rxerr=0
```

Overrun made deliberately, then recovery measured — the property a product depends on:

```
PASS usartrec: overran=1 err_pre=0 drained=3 err_drain=0 ok=6 bad=0
```

An overrun latched (`overran=1`), three bytes were left in the FIFO, and the six bytes sent afterwards were all accepted and
identical. **That is what attempt one could not deliver**: a receiver that survives its first glitch.

The two loopback records are controls for each other: a predicate that refused everything would produce `n=0`, and one that
accepted everything would produce `n=6 bad=0` even on noise — which is why the overrun record also has to show the latch.

## What is still not proven

The internal loopback proves the shifter, the frame format and the read path — past the pads. It proves nothing about the
pad or the level on it, and nothing about a real second machine. The external witness is still deferred: the USB-serial
adapter does not enumerate on the host (port 1-7, `error -71`), and the board's own CDC (`/dev/ttyACM0`) is not wired to
this UART (routing `USART0` to `PA8`/`PA9` produced 102 bytes of `0xC0`). See the resume register.

## Where it lives

- `crates/efr32mg2x-hal/src/cores/usart.rs` — `rx_problem`, `RxProblem`, the `IF` bit constants, and the host tests.
- `crates/efr32mg2x-hal/src/usart/mod.rs` — `read_byte` (via `RXDATAX`), `overran`, `rx_flags`, `rx_errors`,
  `last_rx_problem`, `flush_receiver`, `set_internal_loopback`.
- `crates/efr32mg2x-hal/examples/hal_usart_rx_recovery.rs` — the overrun and recovery instrument.