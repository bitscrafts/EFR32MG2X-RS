# I2C after a NACK: what is measured, and where it stops

The 2026-11-22 review's item 2 was "no hardware I2C path sends STOP or ABORT after an error, so one NACK can leave the
bus held". The instrument is `crates/efr32mg2x-hal/examples/hal_i2c_recovery.rs`, which does three things in one boot
and puts the three answers in the mailbox record.

## The chain of measurements

```
before any fix     i2c-rec: antes=0x6a  erro=2  depois=0x00     erro=2 is Timeout
after the fix      i2c-rec: antes=0x6a  erro=1  depois=0x00     erro=1 is Nack
```

**`erro` is fixed and that is not cosmetic.** A read from `0x55`, where nothing is wired, used to be reported as
`Timeout`; it is now `Nack`. Two causes, both found by measuring:

1. the refusal arrives **before** `TXC` is set -- the master stops and the byte never "leaves" -- so a wait on `TXC`
   alone turns a NACK into a timeout. The wait now checks the flag as it polls.
2. the flag was read through the **non-secure view**, which always reads zero. It is now read through the same window
   it is cleared on.

`F5` from the review is fixed in the same change: the shared primitive macro takes the instance's secure window as a
parameter, so `I2c1` clears its own flag instead of `I2c0`'s.

## Where the recovery stops, and the evidence

`Bus::abort` exists now: the core calls it on every error path instead of returning with START asserted, and the
implementation issues `CMD.ABORT`, waits for the master to leave busy, issues `STOP`, and clears `NACK` and
`BUSHOLD`. The bus still does not come back, and the markers say exactly where it gives up.

Two counters were added temporarily to the shared primitives (`I2C_PASSOS`, `I2C_ULTIMA`), then removed:

```
I2C_PASSOS = 13      I2C_ULTIMA = 4 (abort)
```

Instrumented primitives are `start`(1), `write`(2), `stop`(3), `abort`(4). Counting the three transactions:

```
1  WHO_AM_I 0x6A    start write write start write stop     6 passos   ok
2  read 0x55        start write -> Nack, abort             3 passos   refusa
3  WHO_AM_I 0x6A    start write write start               4 passos   PARA AQUI
```

Nine plus four is thirteen, and the last primitive entered was `abort`. **The first reading of that was wrong**: it
looked like the repeated START, and a second instrument disproved it before it became a fix.

Four counters were placed in `start()` -- calls, expiries, and `STATE` and `IF` read immediately before the command:

```
START_INICIOS = 4    t1 uses two, t2 one, t3 one
START_EXPIRADOS = 0  no START timed out, at any point
START_ESTADO = 0     the bus was idle at the last one
START_FLAGS = 0x109f9  flags left over from the aborted transfer
```

So the third transaction issues **one** START and dies after it, which puts the failure in a `write` -- the `txbl`
wait or the byte itself -- and not in the START at all. The earlier paragraph drew a conclusion from a count of
thirteen without checking which primitive it was; the count was right and the attribution was wrong.

## What that rules in and out

- It is not the bus being held by the absent device: the address and register writes of the third transaction went
  through.
- It is not the fault of the NACK reporting: the error class is right.
- It is **not** the repeated START: no START expired, and the third transaction only issues one. What is left is the
  `write` that follows -- the `txbl` wait, or the byte being refused -- with `IF = 0x109f9` sitting there from the
  aborted transfer, which is the number to decode next.

## The decode, and the mechanism it exposes

`IF = 0x109f9`, read immediately before the last `CMD.START`, against the PAC's bit map:

```
bit  0  start      a START condition happened
bit  3  txc        transfer completed
bit  4  txbl       transmit buffer level
bit  5  rxdatav    receive data valid
bit  6  ack
bit  7  nack       <-- the refusal from transaction 2, still set
bit  8  mstop      leader STOP
bit 11  bushold    <-- the bus is still held
bit 16  sstop      follower STOP
```

**The NACK from the previous transaction is still asserted when the next one begins**, and so is `BUSHOLD`. And the
`write` primitive now checks the NACK flag *inside* the `txbl` wait -- which is how a refusal became distinguishable
from a timeout in the first place -- so the first write of transaction 3 reads the old flag and returns `Nack` without
touching the bus at all. Transaction 3 is refused by a ghost of transaction 2.

`abort()` clears `NACK` and `BUSHOLD` through the instance's own window, and they are set again at the next read,
which says the clear does not clear the condition: `BUSHOLD` is a level -- the bus is still held -- and a
write-one-to-clear on a flag whose condition persists is a no-op dressed as a fix.

The next measurement is a counter on the `txbl` wait inside
`write` -- whether it expires or the byte is NACKed -- Two candidates worth having ready: clearing the transmit FIFO
(`CMD.CLEARTX` / `CMD.CLEARPC`), which is what a leftover `0x109f9` suggests, and a full re-init of the peripheral
after an abort, which costs nothing at the frequency this bus is used and is honest about what "recover" means.

The example fails its own criterion today, and it is committed that way on purpose: it is the target, and a target
that passes while the defect is live would be the defect.