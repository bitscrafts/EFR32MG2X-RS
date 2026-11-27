# sensors

BME680 / BME688 driver: temperature, pressure and humidity.

## What it is

BME680 / BME688 driver: temperature, pressure and humidity.

## The calibration layout, which is not what the register order suggests

Bosch stores the trim in three blocks — 0x8A (23 bytes), 0xE1 (14 bytes) and
0x00 (5 bytes) — and the fields are **not** in register order. The block at
0x8A begins with `par_t2` and `par_t3`; `par_t1` lives at coefficient index
31/32, i.e. inside the second block. Reading 0x8A as `par_t1` gave -1.01 C on
a 32 C bench, and that mistake is why this comment exists.

`par_h1` and `par_h2` also share a byte: index 24 holds `par_h2`'s low
nibble in its high half and `par_h1`'s lsb in its low half. Treating it as a
plain lsb byte made humidity saturate at 100 %.

## Mode changes

Bosch's `set_op_mode` contract: write **sleep** first, then the wanted mode.
Skipping that makes the sensor ignore the write silently — the bus ACKs, and
`ctrl_meas` still reads back as sleep.

## Provenance

The compensation formulas and the coefficient indices were cross-checked
against the working implementation in <repo>
(same sensor family, same silicon, verified on hardware there) and then
re-verified here against a room at ~32 C and ~1015 hPa.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `ADDR_DEFAULT` | const | Default 7-bit address (SDO/SA0 low). |
| `ADDR_PRIMARY` | const | Default address when the sensor's SDO pin is pulled low. |
| `ADDR_SECONDARY` | const | Alternate address when SDO is pulled high. |
| `AccelOdr` | enum | — |
| `AccelRange` | enum | — |
| `Bme68x` | struct | A detected BME68x with its calibration already loaded. |
| `Config` | struct | — |
| `ImuError` | enum | — |
| `Lsm6ds3trc` | struct | A configured LSM6DS3TR-C. |
| `Motion` | struct | — |
| `Reading` | struct | One compensated measurement. |
| `SensorError` | enum | What can go wrong while talking to the sensor. |
| `WHO_AM_I_LSM6DS3` | const | What WHO_AM_I must read on an LSM6DS3 / LSM6DS3TR-C. |
| `free` | fn | Give the bus back. |
| `new` | fn | Detect the sensor at 0x76 or 0x77, check the id and load the trim. |
| `read` | fn | Read one sample. |
| `read_with` | fn | Trigger one forced measurement and wait for it by polling `new_data`. |
| `with_address` | fn | Same, for a part strapped to the alternate address. |
| `with_config` | fn | Verify the identity and configure the given accelerometer range and rate. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

See `examples/hal_imu_read.rs`
