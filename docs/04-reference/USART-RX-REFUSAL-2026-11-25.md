# The receiver that reported bytes nobody sent

Measured 2026-11-25 on the XIAO MG24. This is the closure of the worst instrument defect in this crate: a link that
reported 271 bytes with nothing connected to it.

## What it said before

```
PASS usart: echo n=271 last=00
```

`n=271` was the number of bytes the driver believed it had received on a pad with **nothing on it**. A floating line
produces start bits out of noise; each one sets `RXDATAV`; `read_byte` returned `Some(byte)` for every one of them. The
example printed PASS, because it had no way to know that the bytes it was counting did not exist.

## Why the obvious fix is not available

`STATUS` on this MCU has no per-byte error bit. The generated PAC lists `RXENS`, `TXENS`, `MASTER`, `RXBLOCK`, `TXTRI`,
`TXC`, `TXBL`, `RXDATAV`, `RXFULL`, `TXBDRIGHT`, `TXBSRIGHT`, `RXDATAVRIGHT`, `RXFULLRIGHT`, `TXIDLE`,
`TIMERRESTARTED`, `TXBUFCNT` — and nothing that says "this frame was broken". The only evidence is in `IF`:
`RXOF` at bit 4, `PERR` at 8, `FERR` at 9.

So the driver reads the byte, reads `IF`, and reports the verdict alongside it. The decision layer is pure arithmetic and
lives in `cores::usart::rx_problem`, host-tested four ways, with the defect flipped back on as a control (forcing
`rx_problem` to `None` turns two of those tests red).

## The two records, both taken with the same instrument

**Nothing on the RX pad** (the defect's scenario):

```
usart: echo n=0 rxerr=524 why=ferr
```

Zero bytes accepted, 524 refused and still counting, reason `ferr`. The 271 phantom bytes are gone, and the counter keeps
moving — which is the second half of the result: the receiver did not stall.

**The peripheral's own internal loopback** (`CTRL.LOOPBK`, paced one byte at a time):

```
PASS usart: selftest n=6 ok=6 bad=0 rxerr=0 pre=02 cleared=02 post=03
```

Six bytes written, six came back identical, none refused, and `pre`/`post` show no error bits at all. This is the
control that keeps the first record honest: a predicate that refused everything would also have produced
`n=0 rxerr=524`.

The internal loopback proves the shifter, the frame format and the read path. It proves **nothing** about the pad or the
level on it — for that the header needs a second machine, and that proof is still deferred (see the resume register).

## What the same session measured about `IF`

The first attempt at the selftest wrote the banner with the loopback already on. Thirty-four bytes into a two-deep FIFO
latched an overrun, and the record said:

```
usart: selftest n=6 ok=0 bad=6 rxerr=7 pre=1f cleared=1f post=1f
```

`pre=1f` has bit 4 set (`RXOF`). `cleared=1f` is the flags after writing one to each set bit — **unchanged**. So on this
part, through the secure instance the board hands out, the documented write-1 clear does not clear `RXOF`.

That is why the driver reports the verdict instead of swallowing the byte: a driver that refused and expected the next
byte to be clean would refuse for the rest of the program's life. Consuming the byte always is what keeps the receiver
alive, and the caller decides what to do with `Err(problem)`.

Two things this does not establish, and the next session should: whether the clear works through the **non-secure**
alias (`USART0_NS`), and whether the flags clear themselves once the FIFO is drained (the `n=0 rxerr=524` run suggests
`RXOF` is not the flag that stays latched there — it reports `ferr`, a per-byte condition).

## Where it lives

- `crates/efr32mg2x-hal/src/cores/usart.rs` — `rx_problem`, `RxProblem`, the `IF` bit constants, and the four host tests.
- `crates/efr32mg2x-hal/src/usart/mod.rs` — `read_byte() -> Option<Result<u8, RxProblem>>`, `rx_errors()`,
  `last_rx_problem()`, `rx_flags()`, `clear_rx_flags()`, `set_internal_loopback()`.
- `crates/efr32mg2x-hal/examples/hal_usart_link.rs` — both roles, and the `INTERNAL_LOOPBACK` control.