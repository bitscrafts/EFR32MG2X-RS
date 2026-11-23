# I2C after a NACK: closed, and the measurement that closed it

**Status: closed, 2026-11-23.** One refusal no longer costs the bus. The instrument is
`crates/efr32mg2x-hal/examples/hal_i2c_recovery.rs`, which does five things in one boot and puts the answers in the
mailbox record:

```
i2c-rec: antes=0x6a erro=1 depois=0x6a wr=1 rd6=1 nz=1 volta=0x40      status=PASS
```

in order: the control read answered (`antes`); the absent address came back as **Nack** and not a timeout (`erro=1`);
the device that *is* there answered again after the refusal (`depois`); a plain `write` landed (`wr=1`); a
**six-byte** burst came back with data that is not zero (`rd6`, `nz`); and the register read back the value that was
written (`volta=0x40`). `nz=1` is not decoration: a transfer that quietly returns zeros is the signature of a read
that never happened on this device (its register pointer returns to 0 at STOP), so a zero burst is refused.

## What was wrong

1. **The sticky `IF` flags were never cleared.** `IF` is RW on this part and each flag "needs to be manually cleared
   by software" (RM 22.3.8.2). The vendor clears them by writing **`IF_CLR`**, the clear-alias window at `+0x2000`
   (`I2C_IntClear`, `vendor445/inc/em_i2c.h:295-300`). The code wrote 1 to the **plain** register, which on this
   family does not clear a flag.
2. `write` checks `IF.NACK` **before** it loads a byte, and it has to: a refusal arrives before `TXC`, so waiting on
   `TXC` alone turns a NACK into a timeout. With the flag never cleared, the NACK of one transaction was still
   asserted when the next one began -- so the next transaction was refused **by the previous refusal**, without
   touching the bus. That is precisely what the markers showed: `IF = 0x109f9` immediately before the third
   `CMD.START`, bit 7 set, and no START ever expired.

The 2026-11-17 review had already found this class of defect once (H3: nine sites "cleared" flags by writing 1 to
`IF`). The `HwBus` written on 2026-11-22 reintroduced it in the two places the blocking path uses, and the same day's
document then explained the resulting ghost as a **level**: "`BUSHOLD` is a level: the bus is genuinely held".

**That reading was wrong, and the manual says so**: `STATE.BUSHOLD` means "the bus is currently being held by **this
I2C module**" (RM 22.5.5), and `IF.BUSHOLD` means "the bus becomes held by the I2C module" (RM 22.5.16). Both describe
the module paused waiting for software -- its own normal interaction, the way it holds SCL while it waits -- not
another device holding SDA low. Nothing was stuck on the wires.

## How the cause was attributed, in both directions

A pass does not attribute a cause, and the fix changed four things at once. So the control was run backwards: the
clear was put back on the plain `IF` and **every other change kept**.

| firmware | record | verdict |
|---|---|---|
| before any change | `antes=0x6a erro=1 depois=0x00` | FAIL |
| all four changes | `antes=0x6a erro=1 depois=0x6a wr=1 rd6=1 nz=1 volta=0x40` | PASS |
| **control**: only the clear reverted to plain `IF` | `antes=0x6a erro=1 depois=0x00` | **FAIL** |
| control reverted, fix restored | `... depois=0x6a ...` | PASS |

Same instrument, same board, same session, one binary apart. That makes the clear load-bearing and makes the
instrument demonstrably sensitive to it. It does **not** prove each of the other changes necessary: `ABORT` with
`STOP` in one command, `CLEARPC` and `CLEARTX` all run in the passing firmware, and none was removed on its own. What
the control proves is that they are not **sufficient** without the clear.

## The recovery sequence, and where each step comes from

`HwBus::abort` is now (RM 22.5.4 for the command bits, 22.3.8.4 for the bus-idle rule):

1. `CMD = ABORT | STOP` **in one write**: "abort the current transmission making the bus go idle. When used in
   combination with STOP, a STOP condition is sent as soon as possible before aborting". The old code wrote `ABORT`,
   waited, and then wrote `STOP` -- a STOP arriving when there is no transfer left to stop.
2. wait for `STATE.BUSY` to clear, bounded -- that is what says the module is off the bus.
3. `CMD = CLEARPC`: a command left pending by the refused transaction is otherwise consumed by the next one.
4. `CMD = CLEARTX`: "clear transmit buffer and shift register" -- without it the refused address byte is still queued
   when the next transaction begins.
5. clear every sticky flag through `IF_CLR`.

`I2c0::write` and `I2c0::read` were also rerouted through `cores::i2c`, as `write_read` already was, so every blocking
entry point runs the same host-tested sequence and aborts on every error path instead of returning with START still
asserted. `I2c1`'s `write_read`, still two transactions with a STOP in the middle, is one transaction now. The
duplicated state machines that did this inline are gone: 372 lines out of `src/i2c/mod.rs`, 146 in, plus one
`From<cores::i2c::Error>` in `types.rs` so the mapping is written once instead of at every entry point.

## What is still open

- **A follower that really holds SDA low needs clocks on SCL** -- nine of them, then a STOP, with the pins taken from
  the peripheral and handed back. No instrument in this repository produces that condition; the one measured above was
  the module holding its own bus, which `ABORT` releases in hardware. Before that recovery is worth writing, the
  condition needs to be recognisable, and the honest candidate is `STATE.BUSY` still set with `STATE.BUSHOLD` clear
  after an abort -- not an inference from a flag.
- **The blocking `embedded-hal` `transaction` still emits one STOP per operation.** The async implementation emits a
  repeated START between operations, which is what the trait promises; the blocking one does not. Not measured, not
  fixed, named here so it is not met as a surprise. The fix has the async shape: a `cores::i2c::transaction` over an
  operation list, host-tested in the sim before it reaches the part.

## The trail that led here (kept, because the wrong turn is the useful part)

The 2026-11-22 review's item 2 was "no hardware I2C path sends STOP or ABORT after an error, so one NACK can leave the
bus held". Two temporary counters in the shared primitives (`I2C_PASSOS`, `I2C_ULTIMA`, both since removed) gave:

```
I2C_PASSOS = 13      I2C_ULTIMA = 4 (abort)
1  WHO_AM_I 0x6A    start write write start write stop     6 passos   ok
2  read 0x55        start write -> Nack, abort             3 passos   refusa
3  WHO_AM_I 0x6A    start write write start               4 passos   PARA AQUI
```

**The first reading of that was wrong**: thirteen steps looked like the repeated START, and a second instrument
disproved it before it became a fix -- no START expired (`START_EXPIRADOS = 0`), the bus was idle at each one, and the
third transaction issues only one. Four counters in `start()` read `STATE` and `IF` immediately before the last
command and produced the number that did explain it, `IF = 0x109f9`, with the NACK of transaction 2 still set.

The count was right; the attribution was wrong; the third reading -- "`BUSHOLD` is a level, so the bus is genuinely
held" -- was wrong too, and it was the one written into this file as a conclusion. What is worth keeping is the
order that finally worked: measure the state the module is in, not the number of steps it took to get there, and put
the manual's meaning of each flag against the register value before naming a cause.

## Files

- `crates/efr32mg2x-hal/src/i2c/mod.rs` -- `nack_pendente`, `limpar_nack`, `limpar_flags`, the `abort` sequence, and
  the three blocking entry points.
- `crates/efr32mg2x-hal/src/i2c/types.rs` -- one `From<cores::i2c::Error>` instead of a match at every entry point.
- `crates/efr32mg2x-hal/examples/hal_i2c_recovery.rs` -- the instrument, now with the six-byte burst and the write
  round trip.
- `docs/04-reference/I2C0-READ-DEFECT.md` -- the older "I2C0 cannot read" thread, closed by the same two causes.