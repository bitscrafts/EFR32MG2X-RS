# Quick Flash Guide for XIAO MG24 (CMSIS-DAP)

**TL;DR**: Use probe-rs with the PAC `blink_xiao` example. App address is
`0x08006000` (above the 24 KB Gecko bootloader at `0x08000000`).

**Verified working**: 2026-06-29 (PAC `blink_xiao` blinks the onboard LED on PA7).

---

## Hardware Setup

- **XIAO MG24 Sense** connected via USB
- **Interface**: CMSIS-DAP (onboard SAMD11 bridge)
- **Serial Port**: `/dev/cu.usbmodem24D15EE43`
- **No external debugger needed!**
- **Chip**: `EFR32MG24B220F1536IM48` (use the full ID with `IM48` suffix)

---

## Why probe-rs Instead of Commander?

**Simplicity Commander**: Designed for J-Link debuggers
**probe-rs**: Supports CMSIS-DAP directly

When connected via USB (CMSIS-DAP), use **probe-rs**.
When connected via J-Link, use **commander**.

---

## Flash with probe-rs (CMSIS-DAP) -- VERIFIED WORKING

### Method 1: Build + flash + run in one step (recommended)

```bash
cd efr32mg24-pac
probe-rs run --chip EFR32MG24B220F1536IM48 \
  crates/efr32mg24-pac/target/thumbv8m.main-none-eabihf/release/examples/blink_xiao
```

`probe-rs run` flashes the ELF at its linked addresses (vector table at
`0x08006000`) and runs it immediately. It does NOT erase the bootloader region
(`0x08000000-0x08005FFF`) because the ELF's sections are linked above
`0x08006000`.

> **Measured 2026-11-10**: before `--nmagic` was added to `.cargo/config.toml`,
> every ELF also carried a 372-byte `PT_LOAD` at `0x08000000` (the ELF header,
> page-aligned down). `probe-rs` skips it — verified against the board's flash —
> but openocd and Commander are not known to. Images built now have no segment
> below `0x08006000`. See `FLASHING.md`.
>
> On the flash host, use `scripts/flash.sh`, which refuses any image whose
> `load_min` is under `0x08006000` and verifies by read-back.

### Method 2: Two-step (build first, then flash + run)

```bash
# 1. Build (the --features rt is required for cortex-m-rt)
cd efr32mg24-pac
cargo build --release --example blink_xiao --features rt

# 2. Flash + run
probe-rs run --chip EFR32MG24B220F1536IM48 \
  target/thumbv8m.main-none-eabihf/release/examples/blink_xiao
```

### Expected behavior

The onboard user LED (PA7, active low) blinks at approximately 1 Hz. This
proves the full PAC infrastructure works: PAC typed register accessors ->
CMU_S clock gate -> GPIO_S port config -> DOUT read-modify-write toggle ->
cortex-m-rt entry -> linker memory map -> probe-rs SWD flash.

---

## Flash Address Safety

**CRITICAL**: The Gecko bootloader occupies `0x08000000 - 0x08005FFF` (24 KB).
Applications MUST be linked at `ORIGIN = 0x08006000` (the PAC `memory.x` already
has this correct). Linking at `0x08000000` overwrites the bootloader and bricks
the board's serial-upload capability (see `CRITICAL_SAFETY_PROTOCOL.md`).

Only the PAC crate's `memory.x` is correct right now. The HAL and BSP
`memory.x` files still have `ORIGIN = 0x08000000` and MUST be fixed to
`0x08006000` before any HAL/BSP example is flashed (follow-on task).

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

Confirm the chip is reachable:
```bash
probe-rs info --chip EFR32MG24B220F1536IM48
```

---

## Gecko Bootloader Recovery

If a known-good app hex stops booting after a power cycle (symptom: the
bootloader is present at `0x080002c0` but does not jump to the app), restore the
bootloader + a working app with the recovery image:

```bash
openocd -s <openocd_scripts> -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program crates/efr32mg24-bootloader/arduino-working-examples/microphone_volume_with_bootloader.hex verify; reset run; shutdown"
```

This writes the full image (bootloader at `0x08000000` + app at `0x08006000`).
After this, app-only flashes at `0x08006000` boot correctly via the bootloader.
See `docs/04-reference/hardware/XIAO_MG24_HARDWARE.md` for details.

---

## Troubleshooting

**Error: No probe found**
```bash
# Replug USB, then:
probe-rs list
```

**LED does not blink after `probe-rs run`**
- Check the example targets PA7 (the user LED), not PB2. The BSP `pins.rs`
  incorrectly claims PB2/D10 is the LED; the LED is PA7 (Arduino pin 20).
- Check the ELF is linked at `0x08006000` (not `0x08000000`):
  `arm-none-eabi-size -A <elf> | grep vector_table` should show `134242304`
  (`0x08006000`).
- If using a HAL/BSP example, ensure its `memory.x` has `ORIGIN = 0x08006000`.

**Bootloader does not jump to the app after power cycle**
- The bootloader may have been corrupted. Use the Gecko bootloader recovery
  procedure above to restore a known-good bootloader + app image.

---

## Key Pin/Register Reference (verified against SVD)

| Resource | Address | Notes |
|----------|---------|-------|
| Gecko bootloader | `0x08000000 - 0x08005FFF` | 24 KB, do not overwrite |
| Application flash | `0x08006000 - 0x0817FFFF` | 1512 KB, app link origin |
| `CmuS` (secure CMU) | `0x4000_8000` | `CLKEN0` at +`0x064`, `gpio` = bit 26 |
| `GpioS` (secure GPIO) | `0x4003_C000` | Port registers are packed (stride ~0x30) |
| User LED | PA7 (Arduino pin 20) | Active low; no `DOUTTGL` (use RMW on `DOUT`) |

---

Last updated: 2026-06-29