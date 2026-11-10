# Build Optimization Guide

**Last Updated**: December 28, 2025

This document explains how to optimize build times and avoid unnecessary recompilation in the EFR32MG24 workspace.

---

## Critical Rule: Never Clean the Entire Workspace

**NEVER run `cargo clean` without a package specifier!**

The PAC (Peripheral Access Crate) contains **138,448 lines** of generated code and takes **~3-8 minutes** to compile. Rebuilding it unnecessarily wastes time and resources.

### ❌ Wrong (Rebuilds Everything Including PAC)

```bash
cargo clean              # DON'T DO THIS
cargo build              # Now rebuilds PAC (~8 min)
```

### ✅ Correct (Selective Cleaning)

```bash
# Clean only the package you're working on
cargo clean -p efr32mg24-hal
cargo build -p efr32mg24-hal
```

---

## Package Build Times

Understanding build times helps you avoid unnecessary work:

| Package | Lines of Code | Build Time (Clean) | Build Time (Incremental) |
|---------|---------------|-------------------|--------------------------|
| **efr32mg24-pac** | 138,448 | ~3-8 minutes | <1s (if untouched) |
| **efr32mg24-rail-sys** | 3,974 (generated) | ~4m 23s | <1s (if untouched) |
| **efr32mg24-bluetooth-sys** | 4,307 (generated) | ~25s | <1s (if untouched) |
| **efr32mg24-hal** | ~4,500 | ~30-60s | <1s (if untouched) |
| **xiao-mg24-bsp** | ~500 | <10s | <1s (if untouched) |

**Total clean workspace build**: ~8-14 minutes
**Total incremental build (no changes)**: <3 seconds

---

## Selective Cleaning Workflow

### When Working on HAL Code

```bash
# 1. Clean only the HAL package
cargo clean -p efr32mg24-hal

# 2. Build only the HAL package
cargo build -p efr32mg24-hal

# 3. Or build with examples
cargo build -p efr32mg24-hal --examples
```

**Build time**: ~30-60 seconds (PAC and -sys crates reused from cache)

### When Working on BSP Code

```bash
# Clean only BSP
cargo clean -p xiao-mg24-bsp

# Build BSP
cargo build -p xiao-mg24-bsp
```

**Build time**: ~10 seconds (everything else cached)

### When Working on Examples

```bash
# Build specific example (no clean needed)
cargo build --example 14_radio_ble_beacon

# Or build all examples
cargo build --examples -p efr32mg24-hal
```

**Build time**: <5 seconds per example (incremental)

### When Working on -sys Crates

```bash
# Clean specific -sys crate
cargo clean -p efr32mg24-bluetooth-sys

# Rebuild
cargo build -p efr32mg24-bluetooth-sys
```

**Build time**: ~25 seconds (bindgen regeneration)

---

## When to Clean Each Package

### ✅ Clean efr32mg24-hal When:
- Adding new peripheral modules
- Modifying existing HAL code
- Changing HAL Cargo.toml dependencies
- **Never clean for**: Documentation changes, example additions

### ✅ Clean efr32mg24-bluetooth-sys When:
- Updating Bluetooth blobs
- Modifying build.rs
- Changing bindgen configuration
- **Never clean for**: Documentation only

### ✅ Clean efr32mg24-pac When:
- Regenerating from new SVD file
- Switching device variants
- **Never clean for**: Literally anything else (it never changes)

### ✅ Clean Entire Workspace When:
- Switching Rust toolchain versions
- Major dependency updates
- Corrupted build artifacts (very rare)
- **Frequency**: Almost never needed

---

## Incremental Build Best Practices

### 1. Let Cargo Track Changes

Cargo's incremental compilation is smart. It tracks:
- File modification times
- Dependency graphs
- Feature flag changes

**Just run `cargo build`** - it will only rebuild what changed.

### 2. Use Specific Targets

```bash
# Build specific example
cargo build --example 14_radio_ble_beacon

# Build specific package
cargo build -p efr32mg24-hal

# Build specific binary
cargo build --bin my-binary
```

### 3. Check Before Building

```bash
# See what would be compiled
cargo build --dry-run

# Check without building
cargo check -p efr32mg24-hal
```

**cargo check** is much faster than **cargo build** for catching errors.

---

## Example-Specific Workflows

### Adding a New Example

```bash
# 1. Create the example file
touch crates/efr32mg24-hal/examples/14_radio_ble_beacon.rs

# 2. Write the code
# ... edit file ...

# 3. Build just this example (no clean needed!)
cargo build --example 14_radio_ble_beacon --release

# Build time: <5 seconds (incremental)
```

### Modifying an Existing Example

```bash
# 1. Edit the example
# ... edit crates/efr32mg24-hal/examples/06_spi.rs ...

# 2. Rebuild just that example
cargo build --example 06_spi

# Build time: <3 seconds (incremental)
```

---

## Common Mistakes to Avoid

### ❌ Mistake 1: Cleaning After Documentation Changes

```bash
# You edited a README.md
vim crates/efr32mg24-hal/src/gpio/README.md

# DON'T DO THIS:
cargo clean -p efr32mg24-hal  # ❌ Unnecessary

# DO THIS:
cargo build  # ✅ No rebuild needed (docs don't affect compilation)
```

### ❌ Mistake 2: Cleaning Before Every Build

```bash
# DON'T DO THIS:
cargo clean && cargo build  # ❌ Wastes 8-14 minutes every time

# DO THIS:
cargo build  # ✅ Incremental build (<3 seconds if nothing changed)
```

### ❌ Mistake 3: Building Entire Workspace for One Package

```bash
# You changed HAL code

# DON'T DO THIS:
cargo build --workspace  # ❌ Rebuilds everything

# DO THIS:
cargo build -p efr32mg24-hal  # ✅ Just the package you need
```

---

## Build Time Optimization Tips

### 1. Use cargo check During Development

```bash
# Fast error checking (no codegen)
cargo check -p efr32mg24-hal

# Build time: ~10 seconds vs ~60 seconds for full build
```

### 2. Use Release Mode Only When Needed

```bash
# Development (faster compile, slower runtime)
cargo build --example 14_radio_ble_beacon

# Release (slower compile, faster runtime)
cargo build --example 14_radio_ble_beacon --release
```

**Dev build**: ~3 seconds
**Release build**: ~8 seconds (with optimizations)

### 3. Parallel Builds

Cargo builds dependencies in parallel by default. You can control this:

```bash
# Use more CPU cores (if available)
cargo build -j 8

# Use fewer cores (to keep system responsive)
cargo build -j 2
```

### 4. Avoid Feature Flag Changes

Changing feature flags forces rebuild:

```bash
# Different feature combinations = different builds
cargo build --features "rt"           # Build 1
cargo build --features "rt,defmt"     # Build 2 (rebuilds)
cargo build --features "rt"           # Build 3 (rebuilds again!)
```

**Solution**: Pick your features and stick with them during development.

---

## Measuring Build Times

### Full Workspace Clean Build

```bash
time cargo clean && cargo build --workspace --release
```

**Expected**: 8-14 minutes

### Incremental Build (No Changes)

```bash
time cargo build --workspace
```

**Expected**: <3 seconds

### Single Example Build

```bash
time cargo build --example 14_radio_ble_beacon
```

**Expected**: 2-5 seconds (first time), <1 second (subsequent)

---

## CI/CD Optimization

For GitHub Actions or other CI:

```yaml
# Cache cargo build artifacts
- uses: actions/cache@v3
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

# Build only what's needed
- run: cargo build -p efr32mg24-hal --examples
```

**First CI run**: ~8-14 minutes
**Subsequent runs**: ~1-2 minutes (cached)

---

## Summary: Quick Reference

| Situation | Command | Build Time |
|-----------|---------|------------|
| Changed HAL code | `cargo clean -p efr32mg24-hal && cargo build -p efr32mg24-hal` | ~60s |
| Added new example | `cargo build --example 14_radio_ble_beacon` | <5s |
| Modified example | `cargo build --example 14_radio_ble_beacon` | <3s |
| Changed docs only | `cargo build` (no clean!) | <1s |
| Changed BSP | `cargo clean -p xiao-mg24-bsp && cargo build -p xiao-mg24-bsp` | ~10s |
| Check for errors | `cargo check -p efr32mg24-hal` | ~10s |
| Full workspace | `cargo build --workspace` | ~60s (incremental) |

**Golden Rule**: When in doubt, **don't clean**. Let Cargo's incremental compilation do its job.

---

**Remember**: The PAC is **138,448 lines** of generated code. Rebuilding it takes **3-8 minutes**. Only clean it when absolutely necessary (almost never).

<!-- META: last_updated=2025-12-28 version=1.0.0 status=active maintained_by=orchestrator -->
