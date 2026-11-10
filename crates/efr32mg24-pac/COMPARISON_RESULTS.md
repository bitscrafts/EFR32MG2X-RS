# ApplicationProperties Comparison Results

## Critical Findings

After byte-by-byte comparison of working Arduino C vs non-working Rust firmware, I identified **THREE key differences**:

### 1. Missing 8-byte Header (FIXED)
**Before**: Rust started directly with magic signature
**After**: Added 8-byte header `02 01 02 02 02 03 02 04` matching Arduino exactly

### 2. Wrong app_type Field (FIXED)
**Before**: Rust used `app_type = 0`
**After**: Changed to `app_type = 0x22` (34 decimal) matching Arduino

### 3. Wrong version Byte Order (FIXED)
**Before**: Rust used `version = 0x00010000` (v1.0.0)
**After**: Changed to `version = 0x00000001` (v0.0.1) matching Arduino

### 4. Word 13 Pointer (FIXED)
**Before**: Pointed to structure start (0x08006220)
**After**: Points to MAGIC (0x08006228), 8 bytes after structure start, matching Arduino behavior

## Byte-by-Byte Verification

### Arduino C (at 0x080C6820, working):
```
Offset  Hex Bytes
0x00:   02 01 02 02 02 03 02 04  <- 8-byte header
0x08:   13 B7 79 FA C9 25 DD B7  <- MAGIC (Word 13 points here: 0x080C6828)
0x10:   AD F3 CF E0 F1 B6 14 B8  <- MAGIC continued
0x18:   01 02 00 00 00 00 00 00  <- struct_version (0x0201), signature_type (0)
0x20:   FF FF FF FF 22 00 00 00  <- signature_location (0xFFFFFFFF), app_type (0x22)
0x28:   01 00 00 00 00 00 00 00  <- version (0x00000001), capabilities (0)
0x30:   00 00 00 00 00 00 00 00  <- product_id (zeros)
0x38:   00 00 00 00 00 00 00 00
```

### Rust (at 0x08006220, now FIXED):
```
Offset  Hex Bytes
0x00:   02 01 02 02 02 03 02 04  <- 8-byte header ✓
0x08:   13 B7 79 FA C9 25 DD B7  <- MAGIC (Word 13 points here: 0x08006228) ✓
0x10:   AD F3 CF E0 F1 B6 14 B8  <- MAGIC continued ✓
0x18:   01 02 00 00 00 00 00 00  <- struct_version (0x0201), signature_type (0) ✓
0x20:   FF FF FF FF 22 00 00 00  <- signature_location (0xFFFFFFFF), app_type (0x22) ✓
0x28:   01 00 00 00 00 00 00 00  <- version (0x00000001), capabilities (0) ✓
0x30:   00 00 00 00 00 00 00 00  <- product_id (zeros) ✓
0x38:   00 00 00 00 00 00 00 00  ✓
```

## Result

**PERFECT MATCH!** The Rust ApplicationProperties now matches Arduino byte-for-byte.

## Files Changed

1. `examples/blink_bare.rs`:
   - Added 8-byte header field to ApplicationProperties struct
   - Set app_type to 0x22 (matching Arduino)
   - Set version to 0x00000001 (matching Arduino byte order)

2. `patch_word13.py`:
   - Updated to point Word 13 to magic (structure + 8 bytes)
   - Added clear documentation of Arduino behavior

3. Generated files:
   - `blink_bare.hex` - Unpatched hex file
   - `blink_bare_patched.hex` - Word 13 patched to 0x08006228

## Next Steps

Flash `blink_bare_patched.hex` to device and test. This is the most promising attempt yet since it matches the working Arduino firmware exactly in all ApplicationProperties fields.

## Safety Protocol

Per established protocol:
1. Have microphone example hex ready for restore if needed
2. Flash test firmware
3. Check LED for blink
4. If no blink or device unresponsive, flash microphone example to verify device not bricked
