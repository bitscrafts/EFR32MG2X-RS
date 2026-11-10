# Hardware Notes - XIAO MG24 Sense

## CMSIS-DAP Debug Interface

**Important**: The XIAO MG24 Sense module uses **SAMD11** as the CMSIS-DAP interface.

### Debug Interface Details

- **Debugger MCU**: Microchip SAMD11
- **Protocol**: CMSIS-DAP (Arm standard debug protocol)
- **Connection**: USB to PC, SWD to EFR32MG24
- **Firmware**: Likely custom Seeed Studio implementation

### OpenOCD Configuration

The SAMD11 CMSIS-DAP interface should work with standard OpenOCD CMSIS-DAP configuration:

```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg
```

### Important Notes

1. **Driver**: Should work with standard HID drivers (no special drivers needed on macOS/Linux)
2. **Speed**: Typical CMSIS-DAP speeds (up to 10 MHz SWD clock)
3. **Features**: Full debug + flash programming support

### Flashing with OpenOCD

```bash
# Flash bootloader + application
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "program combined.hex verify" \
  -c "reset run" \
  -c "exit"
```

### Alternative Flashing Tools

If OpenOCD has issues, alternatives include:

1. **probe-rs**: Modern Rust-based tool
   ```bash
   probe-rs run --chip EFR32MG24B220F1536IM48 combined.hex
   ```

2. **pyOCD**: Python-based debugger
   ```bash
   pyocd flash -t efr32mg24 combined.hex
   ```

3. **Segger J-Link** (if converted from CMSIS-DAP firmware)

### SAMD11 CMSIS-DAP Capabilities

- ✅ Flash programming via SWD
- ✅ Reset target
- ✅ Halt/resume
- ✅ Breakpoints (hardware and software)
- ✅ Register read/write
- ✅ Memory read/write
- ✅ Single-step execution

### Known Limitations

- Max SWD clock typically 10 MHz (slower than dedicated J-Link)
- USB bandwidth may limit large flash operations
- No JTAG support (SWD only)

### Troubleshooting

**If OpenOCD can't find device**:
1. Check USB connection
2. Try: `lsusb | grep -i "seeed\|samd\|cmsis"` (Linux/macOS)
3. Check permissions (may need udev rules on Linux)
4. Try unplugging/replugging USB cable

**If flashing is slow**:
- Normal for SAMD11 CMSIS-DAP
- Large binaries (>100 KB) may take 10-30 seconds

**If connection is unstable**:
- Try shorter/better USB cable
- Reduce SWD clock speed in OpenOCD config

---

**Reference**: XIAO MG24 Sense hardware documentation
