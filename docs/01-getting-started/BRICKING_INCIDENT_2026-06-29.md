# Bricking Incident Report - 2026-06-29

**Status**: Board debug-locked. SWD Access Port inaccessible. Requires JTAG/SWD recovery.

---

## Summary

Flashing and running the HAL `02_blink.rs` example with HFXO clock
configuration caused a hard fault that permanently locked the EFR32MG24's
debug Access Port (AP). The board cannot be accessed via SWD (CMSIS-DAP)
after the crash, even after power-cycling. The Gecko bootloader + crashing
app cycle continues on every boot, re-locking the debug AP.

---

## Timeline of Events

1. **PAC blink_xiao verified working** (earlier in session)
   - `probe-rs run` with PAC `blink_xiao` (HFRCO, no HFXO) -> LED blinked
   - Board was fully functional, SWD accessible

2. **HAL 02_blink.rs built with HFXO config** (original code)
   - `Clocks::new` with `hfxo: Some(HfxoConfig::new(39_000_000))`
   - ELF linked at 0x08006000 (memory.x was already fixed)

3. **First `probe-rs run` with HFXO version -> CRASH**
   ```
   Firmware exited unexpectedly: Exception
   Frame 0: <unknown function @ 0x08000170> @ 0x08000170
   ```
   - The app crashed immediately. PC at 0x08000170 (bootloader region).
   - The debug AP became inaccessible after this crash.

4. **`probe-rs download` with HFXO version -> succeeded**
   - Flash write succeeded (0.65s) - debug AP was still marginally accessible
   - But this wrote the crashing HFXO app to 0x08006000

5. **Rebuilt 02_blink.rs with HFRCO config** (hfxo: None)
   - Removed HFXO configuration to avoid the crash

6. **`probe-rs run` with HFRCO version -> FAILED**
   ```
   Error: Connecting to the chip was unsuccessful.
   Target device did not respond to request.
   ```
   - Debug AP fully locked by this point

7. **USB replug (power-cycle) -> no recovery**
   - `probe-rs info`: "No access ports found on this chip"
   - The crashing HFXO app is still in flash at 0x08006000 and runs every boot

8. **All recovery attempts failed**:
   - `probe-rs erase`: "Target device did not respond"
   - `probe-rs download` (blink_xiao): "Target device did not respond"
   - `openocd reset halt`: "Error connecting DP: cannot read IDR"
   - `openocd` with `connect_assert_srst`: same error
   - `openocd` with 100 kHz clock: same error
   - `pyocd` with `cortex_m` target: found AP#0, #2, #3 but all fault
   - `pyocd` with `under-reset` mode: same AP faults
   - `arduino-cli upload` (uses openocd): same DP/AP errors
   - Serial port listen: no bootloader output received

---

## Root Cause: HAL Clocks::new HFXO Bug

The HAL's `Clocks::new` in `efr32mg24-hal/src/clock/clocks.rs:72-123`
has a **fatal bug** in the HFXO configuration path:

```rust
// clocks.rs:80-89
cmu.sysclkctrl().modify(|_r, w| {
    w.clksel().hfxo()      // <-- SWITCHES SYSCLK TO HFXO FIRST
});
// TODO: Wait for HFXO to stabilize   <-- ADMITS THE BUG
cortex_m::asm::delay(Self::OSC_TIMEOUT);  <-- NEVER EXECUTES
```

**The sequence is backwards.** It switches SYSCLK to HFXO BEFORE HFXO is
running. When `probe-rs run` bypasses the Gecko bootloader, HFXO is not
running (only the default FSRCO is active). Switching SYSCLK to a dead
oscillator causes:

1. The CPU clock stops instantly (no HFXO clock to drive the core)
2. The `delay()` never executes (CPU is frozen)
3. The CPU may execute 1-2 garbage instructions before the clock fully stops
4. A hard fault occurs
5. The fault handler address is in the bootloader region (0x08000170)

**The correct sequence should be:**
```rust
// 1. Enable HFXO oscillator
cmu.oscencmd().write(|w| w.hfxoen().set_bit());

// 2. Wait for HFXO to stabilize (poll STATUS register)
while !cmu.status().read().hfxordy().is_ready() {}

// 3. THEN switch SYSCLK to HFXO
cmu.sysclkctrl().modify(|_r, w| w.clksel().hfxo());
```

### Why This Bricks the Debug AP

The exact mechanism by which the HFXO crash locks the debug AP is unclear,
but the evidence is conclusive:

- Debug AP was fully accessible before the HFXO crash (PAC blink_xiao worked)
- Debug AP became permanently locked after the HFXO crash
- Power-cycling does not restore debug AP access
- The crashing app runs on every boot (bootloader -> jump to crashing app ->
  crash -> reset -> repeat), which may re-trigger the debug lock each cycle

Possible mechanisms:
1. The SYSCLK switch to a dead oscillator stops the debug port clock (derived
   from SYSCLK/HFCLK), making the AP unresponsive
2. The crash triggers a secure debug lock in the DCU (Debug Control Unit)
3. The garbage instructions executed before the clock stops write to a debug
   lock register
4. The repeated crash-reset cycle corrupts the debug AP state machine

---

## Current Board State

- **SAMD11 debugger bridge**: Working (probe-rs list finds it)
- **EFR32MG24 SWD DP (Debug Port)**: Responds (DPIDR 0x6ba02477 readable)
- **EFR32MG24 SWD AP (Access Port)**: LOCKED - cannot read IDR
- **pyOCD findings**: AP#0 (WAIT ACK), AP#2 (FAULT ACK on ROM table),
  AP#3 (WAIT ACK)
- **Serial port**: /dev/cu.usbmodem24D15EE43 - no bootloader output detected
- **Flash content at 0x08006000**: HFXO version of 02_blink.rs (crashes on boot)
- **Bootloader at 0x08000000**: Presumed intact (not overwritten by any flash
  operation - all ELFs were linked at 0x08006000)

---

## Recovery Options (for tomorrow)

### Option 1: External SWD/JTAG debugger

Connect an external SWD debugger (ST-Link, J-Link, DAPLink) directly to the
XIAO MG24's SWD pads on the bottom of the board. This bypasses the SAMD11
CMSIS-DAP bridge and may have different timing/voltage characteristics that
allow the AP to respond.

SWD pads on XIAO MG24 bottom:
- SWCLK
- SWDIO
- 3V3
- GND
- (nRESET may or may not be available)

### Option 2: Serial XMODEM upload (if bootloader supports it)

The Gecko bootloader may support serial upload if:
1. The force-boot pin is held low during power-on
2. The app's ApplicationProperties is invalid (CRC fails)
3. The bootloader is configured with serial upload capability

If the bootloader supports serial upload:
1. Open serial port at 115200 baud
2. Power-cycle the board
3. Send the upload command within the bootloader's wait window
4. Upload a .gbl (Gecko Bootloader) file via XMODEM

Note: The Arduino core uses SWD (openocd) for upload, not serial, which
suggests serial upload may not be configured in the bootloader.

### Option 3: Gecko bootloader recovery via openocd (if AP recovers)

If an external debugger can get AP access:
```bash
OPENOCD=/path/to/openocd
SCRIPTS=/path/to/openocd/scripts
$OPENOCD -s "$SCRIPTS" -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program efr32mg24-bootloader/arduino-working-examples/microphone_volume_with_bootloader.hex verify; reset run; shutdown"
```

This writes the full image (bootloader at 0x08000000 + app at 0x08006000).
After this, app-only flashes at 0x08006000 boot correctly via the bootloader.

### Option 4: Mass erase via JTAG debug port

If an external JTAG debugger can access the DP, a mass erase command
may clear the debug lock and erase all flash. This would restore the chip
to its default state. The Gecko bootloader would need to be reflashed after.

---

## Required Fixes (before any HAL example is flashed again)

### Fix 1: HAL Clocks::new HFXO sequence (CRITICAL)

File: `efr32mg24-hal/src/clock/clocks.rs:72-123`

The HFXO configuration must enable HFXO, wait for stabilization, THEN switch
SYSCLK. The current code switches first and waits second, which is fatal
when HFXO is not already running.

### Fix 2: HAL 02_blink.rs should default to HFRCO

The blink example should use `hfxo: None` (HFRCO) by default. HFXO should
only be used after the clock configuration bug is fixed and verified on
hardware.

### Fix 3: Safety check before HFXO switch

Add a runtime check: only switch SYSCLK to HFXO if HFXO is confirmed running
(STATUS register bit). If HFXO is not ready, fall back to HFRCO and log a
warning. This prevents the bricking scenario even if the clock config is
misused.

### Fix 4: Document the bricking risk

Add a warning to the HAL clock module README and to any example that uses
HFXO configuration:

```
WARNING: The HFXO configuration in Clocks::new can brick the board if HFXO
is not running before SYSCLK is switched. Do NOT use hfxo: Some(...) with
probe-rs run (which bypasses the bootloader). Only use HFXO after the
bootloader has started HFXO, or after the Clocks::new bug is fixed.
```

---

## Lessons Learned

1. **Never switch SYSCLK to an oscillator before it's running.** Always
   enable, wait for ready, then switch.

2. **HAL clock code can brick hardware.** A clock misconfiguration can
   lock the debug AP permanently. HAL clock code must be verified via
   `probe-rs download` + power-cycle (bootloader path) before using
   `probe-rs run` (direct execution path).

3. **Always have a recovery plan.** Before flashing any clock-related code,
   ensure a recovery path exists (known-good hex, external debugger, serial
   bootloader).

4. **probe-rs run bypasses the bootloader.** This means the clock tree is
   in its default FSRCO state. Any HAL code that configures clocks must
   handle the from-scratch case safely.

5. **The flash process is not rock solid.** The `probe-rs run` command
   flashes AND immediately runs the code. If the code crashes in a way that
   locks debug access, the board is bricked. Consider using
   `probe-rs download` (flash only) + manual reset instead of
   `probe-rs run` (flash + run) for clock-related code.

---

## File References

- HAL Clocks::new (the bug): `efr32mg24-hal/src/clock/clocks.rs:72-123`
- HAL 02_blink.rs (the example that bricked the board):
  `efr32mg24-hal/examples/02_blink.rs`
- Recovery hex: `efr32mg24-bootloader/arduino-working-examples/microphone_volume_with_bootloader.hex`
- PAC blink_xiao (verified working, uses HFRCO only):
  `efr32mg24-pac/examples/blink_xiao.rs`
- Quick flash guide: `docs/01-getting-started/QUICK_FLASH_GUIDE.md`
- Hardware details: `docs/04-reference/hardware/XIAO_MG24_HARDWARE.md`

---

**Document created**: 2026-06-29
**Board status**: Debug-locked, requires external SWD recovery
**Next step**: Connect external JTAG/SWD debugger tomorrow to attempt recovery