# I2C0 read defect — every measurement taken on 2026-11-10/11

**Status: open.** Reads through the on-chip I2C0 master do not complete on this
die, while writes do. The workaround in the examples is a bit-banged master
driven through PAC GPIO registers (`examples/common/i2c_bitbang.rs`). This file
exists so nobody re-walks the same fourteen flashes.

## Symptom

`START + address(W) + register` works and the device ACKs. The read phase
(`repeated START + address(R)` or a second transaction) ACKs the address and
then no byte arrives: `STATUS.RXDATAV` never sets, `IF.RXDATAV` never sets,
`IF.RXUF` never sets.

## Measurements, in order (each line = one flash + one SWD read)

| # | Change tried | Result |
|---|---|---|
| 1 | `CMD.START` then `TXDATA` (first address) | `STATE=0xd7` BUSY\|MASTER\|TRANSMITTER\|NACKED, state field 6 = DATAACK; PSTART pending, never cleared |
| 2 | `TXDATA` then `CMD.START` (vendor order, `em_i2c.c:582`) | writes ACK correctly |
| 3 | ACK/NACK taken from `STATUS.TXC` | reported **all** addresses as present (`mask=0f`) — TXC only means "byte shifted" |
| 4 | ACK/NACK taken from `IF.ACK` / `IF.NACK` | `mask=0e`; 0x3C wrongly absent, 0x76+0x77 both present |
| 5 | `CTRL.AUTOACK=1` | no change |
| 6 | `CMD.ACK` issued before reading the data byte | no change (still step 6 = no data) |
| 7 | Two transactions (write pointer, STOP, START + read) | no change |
| 8 | `CLEARTX` + `CLEARPC` before each transfer | `TXBUFCNT=0` confirmed; no change |
| 9 | Clear **all** 22 `IF` bits (was clearing only 5) | no change |
| 10 | `IEN = 0xFFFF_FFFF` (emlib enables these) | no change |
| 11 | Wait for `IF.MSTOP` after STOP (emlib's `i2cStateWFStopSent`) | no change |
| 12 | `EN=0; EN=1` + rewrite `CTRL`/`CLKDIV` before each transfer | no change |
| 13 | Repeat `CMD.NACK` before the repeated START (per `em_i2c.c:659`) | no change |
| 14 | Errata **I2C_E303** workaround (dummy read when `RXFULL=1` and `RXDATAV=0`) | no change — `RXFULL` never sets either |

## The one thing that does work

The **first** read on a freshly enabled engine returns a byte:

```
first=00/0x00        (step 0 = success)
r=00/0x0x            (second and third consecutive read-only transactions also work)
bme=06/0x00          (any read after a *write* transaction: no data)
rtc=01/0x00          (next transfer: address not even ACKed)
```

Control that rules out a broken ACK path: probing 0x55 (nothing there) NACKs,
and `TXBUFCNT` reads 0 after `CLEARTX`. So the master really is sampling SDA and
really is emptying its FIFO.

Conclusion: something about a completed **write** leaves the engine in a state
that breaks the following read, and none of the documented clean-up paths
(abort, clear PC, clear TX, clear all flags, disable/enable) resets it. That is
the thread to pull next: read the I2C chapter of the EFR32MG24 reference manual
for a state that survives `EN=0`, or compare against `I2C_Transfer` run
interrupt-driven (emlib drives its state machine from the interrupt handler and
never polls a dirty engine).

## Why the examples do not wait for this

The Expansion Base work needed a working bus to prove the sensors, and a
bit-banged master is still PAC-only: it drives `PORTC_MODEL`, `PORTC_DOUT` and
`PORTC_DIN` through the generated register types, with no HAL and no crate. The
bus runs at ~90 kHz, which is irrelevant for a sensor refresh every two seconds.

When I2C0 works, only `common/i2c_bitbang.rs` needs a sibling implementation —
every device example talks to the `Bus` interface, so the swap is local.

## Facts about the board this established

- Both I2C lines idle high (PC04/PC05 read `0x30`), so the Expansion Base
  pull-ups are present.
- Devices present: **0x3C** (SSD1306), **0x51** (PCF8563), **0x76/0x77** (BME68x).
- The BME68x answers 0x76 (variant byte `0x00` = BME680 gas profile).
- The PCF8563 reports `VL=1` (oscillator stopped at some point), so its time is
  not trustworthy until it is set; the date read was `2038-10-01`.
