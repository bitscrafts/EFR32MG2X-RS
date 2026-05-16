# Quick Flash Guide for XIAO MG24 (CMSIS-DAP)

**TL;DR**: Use probe-rs, not commander, when connected via USB (CMSIS-DAP).

---

## Hardware Setup

- **XIAO MG24 Sense** connected via USB
- **Interface**: CMSIS-DAP (onboard SAMD11 bridge)
- **Serial Port**: `/dev/cu.usbmodem24D15EE43`
- **No external debugger needed!**

---

## Why probe-rs Instead of Commander?

**Simplicity Commander**: Designed for J-Link debuggers
**probe-rs**: Supports CMSIS-DAP directly

When connected via USB (CMSIS-DAP), use **probe-rs**.
When connected via J-Link, use **commander**.

---

## Flash with probe-rs (CMSIS-DAP)

### Method 1: Direct Flash (Fastest)

```bash
# Build and flash in one command
probe-rs run --chip EFR32MG24B220F1536 --release --example 00_minimal_blink
```

### Method 2: Two-Step (More Control)

```bash
# 1. Build
cargo build --release --example 00_minimal_blink

# 2. Flash
probe-rs flash --chip EFR32MG24B220F1536 \
  target/thumbv8m.main-none-eabihf/release/examples/00_minimal_blink
```

---

## Flash Address Safety

**IMPORTANT**: If bootloader is present (which it is), flash to **0x00004000**:

```bash
# Generate binary
cargo objcopy --release --example 00_minimal_blink -- -O binary blink.bin

# Flash to correct address
probe-rs flash --chip EFR32MG24B220F1536 \
  --base-address 0x00004000 \
  blink.bin
```

---

## Verify Device Detection

```bash
probe-rs list
```

Expected output:
```
The following debug probes were found:
[0]: Seeed Studio XIAO MG24 (Sense) CMSIS-DAP -- 2886:0062:24D15EE4 (CMSIS-DAP)
```

---

## Complete Workflow

```bash
# 1. Check device
probe-rs list

# 2. Build
cd xiao-mg24-bsp
cargo build --release --example 00_minimal_blink

# 3. Flash and run
probe-rs run --chip EFR32MG24B220F1536 \
  --release \
  --example 00_minimal_blink
```

---

## Expected Behavior (00_minimal_blink)

1. **On flash/reset**: LED blinks rapidly 10 times (5 seconds) = Debug trap
2. **After debug trap**: LED blinks slowly (1 Hz) = Normal operation

---

## Troubleshooting

**Error: No probe found**
```bash
# Replug USB, then:
probe-rs list
```

**Error: Flash verification failed**
```bash
# Bootloader may be incompatible, use recovery procedure
# (Requires J-Link)
```

**Device doesn't respond after flash**
- Wait 5 seconds for debug trap
- Check if LED is blinking (may be dim)
- Press reset button on XIAO

---

Last updated: 2026-01-03
