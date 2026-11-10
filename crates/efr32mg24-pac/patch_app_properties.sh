#!/bin/bash
# Post-build script to patch Gecko Bootloader Application Properties pointer
# This script patches vector table word 13 (offset 0x34) to point to APP_PROPERTIES structure
#
# Usage: ./patch_app_properties.sh <input.hex> <output.hex>

set -e

INPUT_HEX="$1"
OUTPUT_HEX="$2"

if [ -z "$INPUT_HEX" ] || [ -z "$OUTPUT_HEX" ]; then
    echo "Usage: $0 <input.hex> <output.hex>"
    echo "Example: $0 blink_pac.hex blink_pac_patched.hex"
    exit 1
fi

if [ ! -f "$INPUT_HEX" ]; then
    echo "Error: Input file '$INPUT_HEX' not found"
    exit 1
fi

echo "=== Gecko Bootloader Application Properties Patcher ==="
echo ""
echo "Input:  $INPUT_HEX"
echo "Output: $OUTPUT_HEX"
echo ""

# Step 1: Find APP_PROPERTIES symbol address in ELF file
# We need the ELF file corresponding to the hex file
ELF_FILE="${INPUT_HEX%.hex}"
if [ ! -f "$ELF_FILE" ]; then
    echo "Error: ELF file '$ELF_FILE' not found (needed to extract APP_PROPERTIES address)"
    exit 1
fi

# Extract APP_PROPERTIES address from ELF
APP_PROPS_ADDR=$(arm-none-eabi-nm "$ELF_FILE" | grep " APP_PROPERTIES" | awk '{print $1}')

if [ -z "$APP_PROPS_ADDR" ]; then
    echo "Error: APP_PROPERTIES symbol not found in ELF file"
    echo "Make sure the firmware includes the ApplicationProperties_t structure"
    exit 1
fi

echo "APP_PROPERTIES address: 0x$APP_PROPS_ADDR"

# Convert to little-endian bytes for Intel HEX
BYTE0=$((16#${APP_PROPS_ADDR:6:2}))
BYTE1=$((16#${APP_PROPS_ADDR:4:2}))
BYTE2=$((16#${APP_PROPS_ADDR:2:2}))
BYTE3=$((16#${APP_PROPS_ADDR:0:2}))

printf "Little-endian bytes: %02X %02X %02X %02X\n" $BYTE0 $BYTE1 $BYTE2 $BYTE3
echo ""

# Step 2: Parse hex file and patch word 13 (offset 0x34)
# Word 13 is at vector table offset 0x34 (52 bytes from start)
# In Intel HEX format, this is line 5 (each line has 16 bytes of data)
# Line 5 starts at offset 0x30, so word 13 is at bytes 4-7 of line 5

echo "Patching vector table word 13 (offset 0x34)..."

# Copy input to output
cp "$INPUT_HEX" "$OUTPUT_HEX"

# Use awk to patch line 5 (offset 0x30-0x3F)
awk -v byte0=$(printf "%02X" $BYTE0) \
    -v byte1=$(printf "%02X" $BYTE1) \
    -v byte2=$(printf "%02X" $BYTE2) \
    -v byte3=$(printf "%02X" $BYTE3) '
BEGIN { line_num = 0 }
{
    line_num++

    # Line 5 contains offset 0x30-0x3F (word 13 at bytes 4-7)
    if (line_num == 5 && substr($0, 1, 1) == ":") {
        # Intel HEX format: :LLAAAATTDD...DDCC
        # Extract header (chars 1-9)
        header = substr($0, 1, 9)

        # Extract data bytes (chars 10-41, 32 hex chars = 16 bytes)
        data = substr($0, 10, 32)

        # Replace bytes 4-7 (chars 9-16 of data) with APP_PROPERTIES address
        data_before = substr(data, 1, 8)   # Bytes 0-3
        data_after = substr(data, 17)      # Bytes 8-15
        data_patched = data_before byte0 byte1 byte2 byte3 data_after

        # Recalculate checksum
        # For simplicity, we will recompute it properly
        byte_count = 16
        addr = 0x6030
        rec_type = 0x00

        # Sum all bytes
        sum = byte_count
        sum += int(addr / 256)  # High byte
        sum += int(addr % 256)  # Low byte
        sum += rec_type

        # Add data bytes
        for (i = 1; i <= 32; i += 2) {
            byte_val = ("0x" substr(data_patched, i, 2))
            sum += strtonum(byte_val)
        }

        # Checksum is two s complement of LSB
        checksum = (256 - (sum % 256)) % 256
        checksum_hex = sprintf("%02X", checksum)

        # Output patched line
        print header data_patched checksum_hex

        printf "  Patched line 5: word 13 = 0x%s%s%s%s -> 0x%02X%02X%02X%02X\n", \
               byte3, byte2, byte1, byte0, \
               strtonum("0x" byte3), strtonum("0x" byte2), \
               strtonum("0x" byte1), strtonum("0x" byte0) > "/dev/stderr"
    } else {
        # Pass through unchanged
        print $0
    }
}
' "$INPUT_HEX" > "$OUTPUT_HEX.tmp"

mv "$OUTPUT_HEX.tmp" "$OUTPUT_HEX"

echo ""
echo "=== Patching Complete ==="
echo "Output written to: $OUTPUT_HEX"
echo ""
echo "Verification:"
echo "Line 5 before:"
sed -n '5p' "$INPUT_HEX"
echo "Line 5 after:"
sed -n '5p' "$OUTPUT_HEX"
echo ""
echo "Ready to flash with OpenOCD!"
