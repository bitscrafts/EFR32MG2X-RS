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

Nine plus four is thirteen, and the last primitive entered was the **repeated START**. So the third transaction
addresses the device, sends the register, and fails on the *second* `start()` -- the very same repeated START that
worked in the first transaction, before anything had been refused.

## What that rules in and out

- It is not the bus being held by the absent device: the address and register writes of the third transaction went
  through.
- It is not the fault of the NACK reporting: the error class is right.
- It is the repeated START after an aborted transfer. `CMD.ABORT` + `STOP` releases the pins but leaves the state
  machine in a shape where the next `CMD.START` completes and the one after it does not.

The next measurement is a marker inside `HwBus::start` between the command and the `pstart` wait -- whether the flag
never sets, or sets and is left over from before -- and a read of `STATE` and `IF` (including `BUSHOLD`) immediately
before that second `CMD.START`. Two candidates worth having in hand: clearing the transmit FIFO (`CMD.CLEARTX` /
`CMD.CLEARPC`), and a full re-init of the peripheral after an abort, which costs nothing at the frequency this bus is
used and is honest about what "recover" means.

The example fails its own criterion today, and it is committed that way on purpose: it is the target, and a target
that passes while the defect is live would be the defect.