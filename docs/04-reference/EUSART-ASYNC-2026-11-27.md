# EUSART0 in asynchronous mode — measured 2026-11-27

Item 4 of the night queue (`_tmp/night/STATE-NIGHT2.md`), `B1b`. The limitation it closes is the one line in
`CLAUDE.md` that said "USART0 only".

## The record

`crates/efr32mg2x-hal/examples/hal_eusart_loopback.rs`, flashed with `scripts/flash.sh --confirm --run`, mailbox read
back with `scripts/read-mailbox.sh`. Four boots, `status=PASS` in all four. Three were byte-identical; the fourth (from
the `cargo fmt`-ed tree) differs in one field, `cal`, by one tick — the calibration window's own quantum:

```
eusart: ok=1 n=8 am=8 wr=8 err=0 ctl=0 div=297 cal=3290 ct=23 tt=23 bd=113975 if=0x000c2007 ft=0x000c2007 fc=0x000c2007 cf=0x00000002
```

| field | meaning | value |
|---|---|---|
| `ok` | the verdict below | `1` |
| `n` / `wr` | frames the pattern has / frames written | `8` / `8` |
| `am` | frames that came back identical | `8` |
| `err` | bytes the driver refused on an error flag | `0` |
| `ctl` | bytes received in the **control** run (loopback off) | `0` |
| `div` | value written to `CLKDIV.DIV` for 115 200 at 19 MHz | `297` |
| `cal` | SYSRTC ticks in a 100 ms board delay — the time base, calibrated in the same run | `3290` |
| `ct` | ticks for 80 bit times in the **control** run | `23` |
| `tt` | ticks for 80 bit times in the **loopback** run | `23` |
| `bd` | rate computed from `tt`: `80 * 32768 / tt` | `113 975` |
| `if` | `IF` after the loopback run | `0x000c2007` |
| `ft` / `fc` | `IF` after the timing run, and after a write-one to the error bits | `0x000c2007` / `0x000c2007` |
| `cf` | `IF` before the control run | `0x00000002` |

`0x000c2007` is `TXC`(0) | `TXFL`(1) | `RXFL`(2) | `TXIDLE`(13) | `STARTF`(18) | `SIGF`(19). None of `RXOF`(4),
`PERR`(8), `FERR`(9).

## The three runs

1. **Control, `CFG0.LOOPBK` off** — same eight bytes, same pads, RX pad pulled up, nothing looped: `ctl=0`. Without this
   run "eight bytes came back" would not distinguish an internal loop from a receiver reading noise off a floating pin
   — this project already measured a floating USART line producing 271 bytes (`hal_usart_link`).
2. **Loopback on** — the eight distinct bytes of the pattern (`A5 5A 00 FF 0F F0 12 34`) written one at a time and
   drained one at a time: `am=8`, `err=0`, no error flag. One at a time because the receiver's FIFO is shallow and the
   transmitter outruns a write-then-drain-later loop.
3. **Timing** — the same eight frames with the loopback still on and nobody draining, timed on SYSRTC. `ct` and `tt`
   agree at 23 ticks, so the transmitter's rate is the same with and without the loop, and the divide of 80 bit times by
   23 ticks (32 768 Hz) is 113 975 baud, within 1.1 % of the 115 200 asked for. The time base is calibrated in the same
   run: 3290 ticks per 100 ms of board delay is 32 900 Hz against a 32 768 Hz nominal.

## Facts the failures established (each is now a rule in the driver)

### 1. `CFG0` cannot be written with `EN` set

First version: `set_internal_loopback` wrote `CFG0.LOOPBK` on the running block, the way `Usart0` writes `CTRL.LOOPBK`.
The core died with a precise bus fault:

```
CFSR=0x00008200   BFARVALID | PRECISERR
HFSR=0x40000000   FORCED
BFAR=0x4b010008   EUSART0_S (0x4b010000) + 0x08 = CFG0
```

Stage marks (six `AtomicU32` in RAM) put the death after the control run and before the first loopback byte, which is
exactly the `CFG0` write.

### 2. Dropping `EN` is not instantaneous, and the enable bits do not survive the cycle

Second version added `EN = 0` before the `CFG0` write. It faulted at the same address: `EN` is not down on the next bus
access. `EN.DISABLING` (bit 1) is the status to wait for — the same lesson as the TIMER's `EN.DISABLING` and the PCNT's.

Third version waited for `DISABLING`, wrote `CFG0`, and raised `EN` — no fault, but no record either: the run stopped
after the third byte with the fourth stage never written, which is a `write_byte` blocked on `STATUS.TXFL`. `CMD.TXEN`
and `CMD.RXEN` are part of the block's state and the `EN` cycle leaves the transmitter disabled; they have to be written
again. With that line the run completes.

### 3. The EUSART divider is not the USART divider

With the USART arithmetic (`DIV = 256 * (f/(OVS*baud) - 1)`, `DIV = 2382` for 115 200 at 19 MHz) the port produced
**167 ticks for 80 bit times = 15 650 baud** — 7.3x slow, with the receiver still clean (`am=8`), because a loopback
does not care whether both ends agree with the nominal rate. The EUSART's `CLKDIV` carries **five** fraction bits:

```
DIV = 32 * (f / (OVS * baud) - 1)        -> 297 for 115 200 at 19 MHz
baud = f / (OVS * (1 + DIV/32))          -> 115 501 for the same numbers
```

With `DIV = 297` the wire produced 23 ticks = 113 975 baud. The function lives in `cores::usart::eusart_clkdiv` with a
host doc-test in `libraries/usart-core`; the driver no longer shares the USART's `clkdiv`.

The 1.1 % between 115 501 and 113 975 is the oscillator and the tick quantum, not the divider: at 23 ticks one tick is
4.3 % of the window, and `cal` itself reads 32 900 Hz instead of 32 768.

### 4. The RX FIFO is at least eight frames deep

The timing run wrote eight frames with nobody draining and `RXOF` never latched; `RXFL` stayed set, i.e. the bytes were
still in the FIFO at the end. The two-deep FIFO that made `hal_usart_rx_recovery` overflow a USART is not what this
block has.

## What this does not establish

- The pad. The loop is inside the block: nothing is proven about the level on PA0 (TX) or PC3 (RX), only that the
  shifter, the frame format, the divider and the read path agree with each other.
- Absolute rate accuracy against an external instrument: `bd` comes from the part's own 32 768 Hz branch, calibrated
  against the core-clock delay in the same run.
- Echo with a second machine, flow control, 9-bit frames, parity, two stop bits, or EM2. None was tried.
- Whether `IF`'s write-one-to-clear works: no error flag was ever set (`ft == fc == if`), so the mechanism is untested on
  this block. The driver reports both readings rather than claiming one.

## Where the code is

- `crates/efr32mg2x-hal/src/eusart/mod.rs` — `Eusart0`: `new_at`, `set_internal_loopback`, `write_byte`, `read_byte`,
  `read_frame`, `flush`, `rx_flags`, `clear_error_flags`, `divider`, `actual_baud`.
- `crates/efr32mg2x-hal/src/cores/usart.rs` — `eusart_clkdiv`, `eusart_actual_baud` (host doc-tests).
- `crates/efr32mg2x-hal/src/board.rs` — `Board::eusart0_async` (clock gate, `EUSART0CLKCTRL`, driver) and
  `Board::route_eusart0_async` (TX push-pull, RX input with pull-up).
- `crates/efr32mg2x-hal/examples/hal_eusart_loopback.rs` — this instrument.