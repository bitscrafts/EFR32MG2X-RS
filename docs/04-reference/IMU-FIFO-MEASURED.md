# The IMU FIFO: measured, and not justified yet

**What prompted this.** The 2026-11-22 review's ordered item 5 asked for "the GPIO interrupt and the IMU FIFO for the
cadence". The interrupt half has no wire on this board (`HAL-GAP-ANALYSIS-2026-11-18.md`: INT1/INT2 are no-connects).
This is the FIFO half, measured before it is built.

## What the driver already does

`crates/efr32mg2x-hal/src/sensors/lsm6ds3trc.rs:190` reads **one 12-byte burst** per sample, `0x22..0x2D`
(gyro + accel in a single auto-incrementing read). It is not six register reads.

That matters because the FIFO argument is a throughput argument, and throughput was never the problem:

```
one sample, burst of 12 bytes          315 us   (400 kHz, 9 bits per byte with ack)
at the configured 416 Hz ODR           13.1% of the bus
FIFO of 32 samples, 384 bytes          8.7 ms per drain, 11.3% of the bus at 416 Hz
bytes per sample                       14.0 -> 12.1
overhead saved per sample              14%
```

**1.8 percentage points of bus occupancy.** For that, the price is a 384-byte buffer, watermark programming, a drain
API in the driver, and a board test -- and at 416 Hz the bus is idle 87% of the time either way. On the measurement,
the FIFO does not earn its complexity.

The review's number would have been right for a driver that read six registers per sample: that is six start/stop
pairs per sample instead of one, and a FIFO then saves far more than 14%.

## The one thing the FIFO would buy, which is not throughput

**Contiguity.** The sensor free-runs at its ODR; the MCU reads whenever the loop gets there. If the loop reads slower
than 416 Hz, some samples are never read -- the window is then a sample of a sample, with gaps nobody records. The
FIFO is the only way to guarantee that the 256 samples in a window are 256 *consecutive* samples.

Whether that matters is a question about the product, not about the driver: for an RMS over a stationary noise
exposure the gaps change nothing measurable; for a crest factor or for anything that has to bound the peak inside the
window, they do.

## Decision

**Not building it now**, on this measurement, and not on a preference. The condition that would change the answer is
written above: if the dosimeter's window has to be contiguous, the FIFO becomes necessary and the same 8.7 ms drain
is the cost of it -- one transaction per 32 samples instead of 32 transactions.

What would be built, when that condition is met:

- `set_fifo_mode(watermark)` -- `FIFO_CTRL3` for the gyro/accel decimation, `FIFO_CTRL5` for mode and ODR;
- `drain(&mut [Motion; N])` -- one burst from `FIFO_DATA_OUT`, N from `FIFO_STATUS1/2` (the fill level);
- the watermark drain in the product's loop, with `disc` counting the samples dropped when the FIFO overflows --
  because an overflow that nobody counts is a silent gap, which is the defect class this project keeps meeting.