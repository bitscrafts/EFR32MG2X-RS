# I2C0 read defect — every measurement taken on 2026-11-10/11, and how it closed

**Status: closed, 2026-11-23.** The on-chip I2C0 master reads. It was never the die. The fourteen measurements below
all shared one blind spot: every attempt to clear the `IF` flags wrote to the **plain** register, and on this family a
flag is cleared by writing the **`IF_CLR`** alias at `+0x2000` (the vendor's `I2C_IntClear`, `em_i2c.h:295-300`). So
"cleared all 22 bits, no change" (row 9) was not a measurement of the engine -- it was a measurement of a write that
cleared nothing.

The evidence today, one boot, on this board:

```
i2c-rec: antes=0x6a erro=1 depois=0x6a wr=1 rd6=1 nz=1 volta=0x40      status=PASS
```

`rd6=1 nz=1` is a six-byte burst read of the IMU's acceleration registers with data that is not zero; `wr=1` is a
plain `write` that landed; `volta=0x40` is the register read back. The full chain, the control that attributes the
cause, and the recovery sequence after a NACK are in
[I2C-RECOVERY-STATE.md](I2C-RECOVERY-STATE.md).

## What it actually was

1. **The last byte of a reception was read with ACK instead of NACK.** The follower therefore kept driving data and
   the master never closed the reception: address ACKed, no byte ever became valid, `RXDATAV` never set. Fixed on
   2026-11-22; the correct sequence was already in `cores::i2c::write_read` (`read(i != last)`), and the async driver's
   first version was written from memory instead of from it.
2. **The sticky flags were read through the non-secure view**, which always reads zero on this peripheral, so a
   refusal was reported as a timeout and a stale flag was invisible. Fixed on 2026-11-22.
3. **The flags were cleared through the wrong window** (above). Fixed on 2026-11-23, and that is the one that made
   the bus survive a refusal.

## Symptom, as it presented in 2026-11-10

`START + address(W) + register` works and the device ACKs. The read phase (`repeated START + address(R)` or a second
transaction) ACKs the address and then no byte arrives: `STATUS.RXDATAV` never sets, `IF.RXDATAV` never sets,
`IF.RXUF` never sets.

## Measurements, in order (each line = one flash + one SWD read)

Rows 8 and 9 are annotated with what they could not have tested. The rest stand as measured.

| # | Change tried | Result |
|---|---|---|
| 1 | `CMD.START` then `TXDATA` (first address) | `STATE=0xd7` BUSY\|MASTER\|TRANSMITTER\|NACKED, state field 6 = DATAACK; PSTART pending, never cleared |
| 2 | `TXDATA` then `CMD.START` (vendor order, `em_i2c.c:582`) | writes ACK correctly |
| 3 | ACK/NACK taken from `STATUS.TXC` | reported **all** addresses as present (`mask=0f`) — TXC only means "byte shifted" |
| 4 | ACK/NACK taken from `IF.ACK` / `IF.NACK` | `mask=0e`; 0x3C wrongly absent, 0x76+0x77 both present |
| 5 | `CTRL.AUTOACK=1` | no change |
| 6 | `CMD.ACK` issued before reading the data byte | no change (still step 6 = no data) |
| 7 | Two transactions (write pointer, STOP, START + read) | no change |
| 8 | `CLEARTX` + `CLEARPC` before each transfer | `TXBUFCNT=0` confirmed; no change. `CLEARTX` is in the fix today, so this row says it is not sufficient alone, not that it is useless |
| 9 | Clear **all** 22 `IF` bits (was clearing only 5) | no change — **the clear went to the plain `IF` and cleared nothing**; this is the row that hid the defect |
| 10 | `IEN = 0xFFFF_FFFF` (emlib enables these) | no change |
| 11 | Wait for `IF.MSTOP` after STOP (emlib's `i2cStateWFStopSent`) | no change |
| 12 | `EN=0; EN=1` + rewrite `CTRL`/`CLKDIV` before each transfer | no change |
| 13 | Repeat `CMD.NACK` before the repeated START (per `em_i2c.c:659`) | no change |
| 14 | Errata **I2C_E303** workaround (dummy read when `RXFULL=1` and `RXDATAV=0`) | no change — `RXFULL` never sets either |

## The anomaly that pointed at it

The **first** read on a freshly enabled engine returned a byte:

```
first=00/0x00        (step 0 = success)
r=00/0x0x            (second and third consecutive read-only transactions also work)
bme=06/0x00          (any read after a *write* transaction: no data)
rtc=01/0x00          (next transfer: address not even ACKed)
```

Reads worked until something was written; after a write, the first attempt returned nothing and the next one did not
even get its address acknowledged -- which is the signature of a flag left asserted, not of a broken read path. That
was written down on 2026-11-10 as "something about a completed write leaves the engine in a state that breaks the
following read", and the state was the NACK flag, never cleared.

## Why part of the board still uses the bit-banged bus

`I2C0` reaches **one** pin pair: its route registers hold one port and one pin. It serves the IMU on PB2/PB3. The
Expansion Base's bus is on PC4/PC5, so it stays on `bus::BitBangI2c` -- a different bus, not a workaround for a
defect. Peripherals and addresses are named in `docs/04-reference/IMU-INVESTIGATION.md` and
`docs/03-development/EXPANSION-BOARD-STATUS-2026-11-11.md`.

## Facts about the board this established

- Both Expansion Base I2C lines idle high (PC04/PC05 read `0x30`), so its pull-ups are present.
- Devices present: **0x3C** (SSD1306), **0x51** (PCF8563), **0x76/0x77** (BME68x).
- The BME68x answers 0x76 (variant byte `0x00` = BME680 gas profile).
- The PCF8563 reports `VL=1` (oscillator stopped at some point), so its time is not trustworthy until it is set; the
  date read was `2038-10-01`.