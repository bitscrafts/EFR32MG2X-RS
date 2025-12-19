# CI/CD Issues and Resolutions

This document tracks common CI/CD failures and their solutions to prevent recurring issues.

**Last Updated**: December 19, 2025

---

## Issue #1: UTF-8 Encoding Errors

**Date**: December 19, 2025
**File**: `efr32mg24-hal/examples/07_timer_pwm.rs`

### Problem

Code Quality workflow failed with:
```
error: couldn't read `07_timer_pwm.rs`: stream did not contain valid UTF-8
byte `146` is not valid utf-8
```

### Root Cause

Unicode arrow character "→" (byte 146) used in comments instead of ASCII "->".

### Solution

Replace all Unicode characters with ASCII equivalents:
- "→" replaced with "->"
- "←" would be replaced with "<-"
- Any non-ASCII characters must be ASCII-compatible

### Prevention

1. Always use ASCII characters in source code comments
2. Run `cargo fmt --all` before committing
3. Test locally with `cargo fmt --all -- --check`
4. Editor should be configured to warn about non-ASCII characters

### Code Example

**WRONG**:
```rust
// Fade in: 0% → 100% brightness
for duty in 0..=100u8 {
    timer.set_duty_cycle(PwmChannel::Channel0, duty).unwrap();
}
```

**CORRECT**:
```rust
// Fade in: 0% -> 100% brightness
for duty in 0..=100u8 {
    timer.set_duty_cycle(PwmChannel::Channel0, duty).unwrap();
}
```

---

## Issue #2: Formatting Violations (rustfmt)

**Date**: December 19, 2025
**Files**:
- `efr32mg24-hal/examples/07_timer_pwm.rs`
- `efr32mg24-hal/src/adc/mod.rs`

### Problem

Code Quality workflow failed with rustfmt diff showing code not matching `cargo fmt` style:

```diff
- timer
-     .set_duty_cycle(PwmChannel::Channel0, duty)
-     .unwrap();
+ timer.set_duty_cycle(PwmChannel::Channel0, duty).unwrap();
```

### Root Cause

Code was manually formatted in a different style than rustfmt's default.

### Solution

1. Always run `cargo fmt --all` after editing code
2. Never manually format code - let rustfmt handle it
3. Configure editor to format on save

### Prevention

1. **Before every commit**: Run `cargo fmt --all`
2. **Local testing**: Run `cargo fmt --all -- --check` to verify
3. **Editor setup**: Enable format-on-save in VS Code/IDE
4. **Pre-commit hook** (optional):
```bash
#!/bin/sh
cargo fmt --all -- --check || {
    echo "Code not formatted! Run 'cargo fmt --all' before committing."
    exit 1
}
```

---

## Issue #3: Clock API Mismatch

**Date**: December 19, 2025
**Files**:
- `efr32mg24-hal/examples/07_timer_pwm.rs`
- `efr32mg24-hal/examples/08_adc.rs`

### Problem

Build errors:
```
error[E0599]: no method named `freeze` found for enum `Result`
error[E0599]: no method named `delay_ms` found for struct `efr32mg24_hal::delay::Delay`
```

### Root Cause

1. `Clocks::new()` returns `Result<(Clocks, CMU), Error>`, not `Clocks` directly
2. `delay_ms()` method requires `DelayNs` trait to be in scope

### Solution

**Clock API** - Use destructuring + `.expect()` + separate `.freeze()`:
```rust
// CORRECT:
let (clocks, cmu) = Clocks::new(
    dp.cmu_s,
    ClockConfig {
        hfxo: Some(HfxoConfig::new(39_000_000)),
        lfxo: Some(Default::default()),
    },
)
.expect("Clock configuration failed");

let frozen_clocks = clocks.freeze(cmu);
```

**Delay API** - Import prelude:
```rust
use efr32mg24_hal::prelude::*;  // Brings DelayNs trait into scope
```

### Prevention

1. Check existing examples for correct API usage patterns
2. Read compiler errors carefully - they often suggest the fix
3. All new examples should import `prelude::*` by default

---

## Issue #4: Missing Trait Imports

**Date**: December 19, 2025
**File**: `efr32mg24-hal/examples/07_timer_pwm.rs`

### Problem

```
error[E0599]: no method named `delay_ms` found for struct `efr32mg24_hal::delay::Delay`
= help: items from traits can only be used if the trait is in scope
help: trait `DelayNs` which provides `delay_ms` is implemented but not in scope
```

### Root Cause

`embedded-hal` traits (like `DelayNs`) must be in scope to use their methods.

### Solution

Always import the prelude in examples:
```rust
use efr32mg24_hal::prelude::*;
```

### Prevention

**Standard example template**:
```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_hal::{
    // ... specific imports ...
    prelude::*,  // ALWAYS include this
};
use panic_halt as _;
```

---

## Pre-Commit Checklist

Before pushing any code changes, **ALWAYS** run these checks locally:

```bash
# 1. Format all code
cargo fmt --all

# 2. Verify formatting (should have no output)
cargo fmt --all -- --check

# 3. Run clippy with deny warnings
cargo clippy -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf --no-deps -- -D warnings

# 4. Build all examples
cargo build --examples --features rt --target thumbv8m.main-none-eabihf --release

# 5. Check for non-ASCII characters (optional but recommended)
grep -rn --include="*.rs" -P '[^\x00-\x7F]' src/ examples/ || echo "No non-ASCII characters found"
```

**All checks must pass** before committing code.

---

## CI/CD Workflow Files

Our GitHub Actions workflows check:

1. **Code Quality** (`.github/workflows/ci.yml`):
   - `cargo fmt --all -- --check` (formatting)
   - `cargo clippy -- -D warnings` (linting)

2. **Build and Test**:
   - `cargo build --examples --features rt`
   - `cargo test`

3. **Documentation**:
   - `cargo doc --no-deps`

---

## Common Error Messages and Solutions

### "stream did not contain valid UTF-8"
→ Replace Unicode characters with ASCII equivalents

### "Diff in /path/to/file.rs"
→ Run `cargo fmt --all`

### "no method named X found for Y"
→ Check if trait needs to be imported (add `prelude::*`)

### "no method named `freeze` found for enum `Result`"
→ Use `let (clocks, cmu) = Clocks::new(...).expect("msg");` then `clocks.freeze(cmu)`

---

## Documentation Standards

When documenting issues:

1. **Date**: When the issue occurred
2. **Files**: Which files were affected
3. **Problem**: Error message or failure description
4. **Root Cause**: Why it happened
5. **Solution**: How it was fixed
6. **Prevention**: How to avoid it in the future
7. **Code Examples**: Show wrong and correct versions

---

**Maintained By**: EFR32MG24 HAL Development Team
**Review Frequency**: After each CI/CD failure
