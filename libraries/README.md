# libraries/

Domain crates: code that decides *what to do* with readings, and does not know what
produced them.

## The rule for entering this folder

A crate may live here only if all three hold. They are checked by `scripts/gate.sh`, not
by good intentions:

1. **No hardware dependency.** `cargo tree` must not contain `efr32mg24-*`, `cortex-m-rt`,
   or any other crate from `crates/`. Not a normal dependency, not a build dependency.
2. **It builds and tests on the host.** `cargo test` on `x86_64-unknown-linux-gnu`, in
   under a few seconds, with no target triple and no linker script.
3. **It is reusable by an application that is not this board.** If the only caller is one
   example in this repository, it belongs in that example, not here.

The reason is not tidiness. The single property that makes this code testable is that it
has nothing to fake: no registers, no bus, no clock. One dependency on a HAL would turn
every `cargo test` into a cross-compilation and the tests would stop being written.

## What is here

| crate | what it decides |
|---|---|
| `motion-detection` | from a stream of accelerometer magnitudes and the interval between them, whether a fall happened: free fall, then a landing, then quiet. Nine host tests; the constants come from nine real drops. |

## What belongs here next, by the same rule

Named only to make the boundary concrete — none of these exist yet:

- **attitude estimation** (complementary or Madgwick filter) if the gyroscope is put to use:
  it is arithmetic over samples, and it must be testable against a recording.
- **a sensor-fusion or calibration crate** if drift compensation becomes more than a moving
  average.
- **protocol parsers** for anything that arrives as bytes (NMEA, a binary frame) — parsing is
  policy, and it is the part that most deserves host tests.

## What does not belong here

- **Drivers.** Anything that reads or writes a register belongs in `crates/efr32mg24-hal`.
- **Wiring.** Which pin, which address, which power gate: that is a BSP.
- **Application sequencing.** A state machine that decides *when to sample* and *what to
  display* is the application; the classifier is a function it calls.
