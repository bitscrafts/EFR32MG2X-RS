# acmp

`ACMP0`: the analogue comparator, used here as a ladder that measures a voltage instead of a pin that compares one.

## What it is

`ACMP0`: the analogue comparator, used here as a ladder that measures a voltage instead of a pin that compares one.

# What this driver encodes, and which line of it was paid for with a fault

1. **The bus gate is `CMU_CLKEN1.ACMP0`, not `CLKEN0`.** Unlike `IADC0`, `LDMA` or `TIMER0`, the comparator's gate
lives in the second bank. With the gate clear the block does not answer at all: the first register read faults
precisely at `ACMP0_S + 0x04` (`CFSR = 0x0000_8200`, `BFAR = 0x4900_8004`, measured 2026-11-27). Nothing in the
block's own registers says so, and a driver copied from another peripheral compiles and faults.
2. **`INPUTCTRL` is not writable while `EN = 0`.** The same run measured a precise fault at `ACMP0_S + 0x14`
(`BFAR = 0x4900_8014`) for exactly that write, so [`Acmp0::new`] enables the block first and
[`Acmp0::compare`] is only reachable afterwards. `CFG` and `CTRL` are writable disabled -- that is what the enable
sequence configures.
3. **`SYNCBUSY.INPUTCTRL` is the handshake.** The input mux is in the analogue domain; the write is accepted and
takes effect over several cycles, and `SYNCBUSY` is the only thing that says when. Every configuration here waits
on it, bounded.
4. **`VREFDIV` is a ladder tap, not a divisor of one.** `VREFOUT = VREFIN * VREFDIV / 63`, with `63` the pass-through
tap; `0` is zero volts. The arithmetic is in [`ladder_mv`], which is host-tested, so a sweeping instrument does
not have to trust the driver's own mental arithmetic.
5. **The output follows `CTRL.NOTRDYVAL` while the block is not ready.** Measured with the block disabled: with
`NOTRDYVAL = 0` the status bit reads 0, with `NOTRDYVAL = 1` it reads 1, with the inputs left where they were. So
a disabled comparator is not "a comparator that says low", and an instrument that never checks
[`Acmp0::is_ready`] cannot tell a measurement from a configured default.

# What this driver does not do

The interrupt path (`IF`/`IEN`, rise/fall), the capacitive-sense resistor bank, hibernate mode, the GPIO output
route, `ACMP1`, and any input on a pin are not exposed: no instrument for them exists in this crate, and a driver
without one is a claim. The measured record and what each run rules out are in
`../../../../docs/../../../../docs/04-reference/ACMP-LADDER-2026-11-27.md`.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Acmp0` | struct | The comparator, owned by one caller. |
| `Error` | enum | — |
| `Source` | enum | — |
| `code` | fn | — |
| `compare` | fn | Point the comparator at two sources and one ladder tap. |
| `disable` | fn | Drop `EN` and wait for `EN.DISABLING`. |
| `enable` | fn | Raise `EN` and wait for `STATUS.ACMPRDY`. |
| `flags` | fn | — |
| `inputs` | fn | — |
| `is_ready` | fn | — |
| `new` | fn | Take `ACMP0`, gate its bus clock, reset it, and leave it enabled and configured with the reset bias. |
| `output` | fn | — |
| `set_not_ready_value` | fn | What the output reads while the block is not ready. |
| `status` | fn | — |

## Registers it touches

`Acmp0S`, `CmuS`

## What was measured

```
    acmp: ok=1 n=252 c1=43 c2=22 c3=43 e=111 f=0 u=0 lo=833 hi=853 sw=01 off=01 st=0x00000000 ic=0x003f2012
    acmp: ok=1 n=252 c1=44 c2=22 c3=44 e=111 f=0 u=0 lo=853 hi=873 sw=01 off=01 st=0x00000000 ic=0x003f2012
```

Controls and the full story: [ACMP-LADDER-2026-11-27.md](../../../../docs/04-reference/ACMP-LADDER-2026-11-27.md)

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

See `examples/hal_acmp_ladder.rs`
