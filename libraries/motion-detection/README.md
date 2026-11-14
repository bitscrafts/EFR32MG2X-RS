# motion-detection

Fall and motion detection from an accelerometer stream.

Thresholds with hysteresis, and a test suite calibrated on recorded drops -- which is also where the honest limit lives:
an impact threshold cannot separate a fall from a hard knock, so the crate reports what it can defend.

`no_std`, no dependencies.
