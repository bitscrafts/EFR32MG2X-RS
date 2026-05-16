#!/usr/bin/env python3
"""
Patch Gecko Bootloader Application Properties pointer in Intel HEX file
Patches vector table word 13 (offset 0x34) to point to APP_PROPERTIES structure
"""

import sys
import struct

def calculate_checksum(line_bytes):
    """Calculate Intel HEX checksum"""
    return (-sum(line_bytes)) & 0xFF

def patch_word13(input_hex, output_hex, app_props_addr):
    """
    Patch word 13 in Intel HEX file

    Args:
        input_hex: Input hex file path
        output_hex: Output hex file path
        app_props_addr: Address of APP_PROPERTIES structure (e.g., 0x08006220)

    Note: Word 13 must point to the MAGIC bytes, which are 8 bytes after
    the structure start (due to 8-byte header). This matches Arduino behavior.
    """
    print(f"=== Gecko Bootloader Word 13 Patcher ===")
    print(f"Input:  {input_hex}")
    print(f"Output: {output_hex}")
    print(f"APP_PROPERTIES structure address: 0x{app_props_addr:08X}")

    # CRITICAL: Word 13 must point to MAGIC (8 bytes after structure start)
    # Arduino has 8-byte header, then magic. Word 13 points to magic.
    magic_addr = app_props_addr + 8
    print(f"Word 13 will point to MAGIC at: 0x{magic_addr:08X} (structure + 8 bytes)")
    print()

    # Convert address to little-endian bytes
    addr_bytes = struct.pack('<I', magic_addr)
    print(f"Little-endian bytes: {' '.join(f'{b:02X}' for b in addr_bytes)}")
    print()

    with open(input_hex, 'r') as fin, open(output_hex, 'w') as fout:
        line_num = 0
        for line in fin:
            line_num += 1
            line = line.strip()

            # Line 5 contains offset 0x30-0x3F (word 13 at bytes 4-7)
            if line_num == 5 and line.startswith(':'):
                # Parse Intel HEX line
                byte_count = int(line[1:3], 16)
                address = int(line[3:7], 16)
                record_type = int(line[7:9], 16)

                # Extract data bytes
                data_hex = line[9:9+byte_count*2]
                data_bytes = bytes.fromhex(data_hex)

                # Replace bytes 4-7 with APP_PROPERTIES address
                patched_data = data_bytes[:4] + addr_bytes + data_bytes[8:]

                # Build new line
                new_line_data = [byte_count, address >> 8, address & 0xFF, record_type]
                new_line_data.extend(patched_data)

                # Calculate checksum
                checksum = calculate_checksum(new_line_data)

                # Format line
                patched_line = f":{byte_count:02X}{address:04X}{record_type:02X}"
                patched_line += ''.join(f'{b:02X}' for b in patched_data)
                patched_line += f'{checksum:02X}'

                fout.write(patched_line + '\n')

                print(f"✓ Patched line 5:")
                print(f"  Before: {line}")
                print(f"  After:  {patched_line}")
                print(f"  Word 13 now points to: 0x{magic_addr:08X} (MAGIC bytes)")
            else:
                # Pass through unchanged
                fout.write(line + '\n')

    print()
    print("=== Patching Complete ===")
    print(f"Output written to: {output_hex}")
    print()
    print("Verification:")
    with open(input_hex, 'r') as f:
        lines = f.readlines()
        print(f"  Line 5 before: {lines[4].strip()}")
    with open(output_hex, 'r') as f:
        lines = f.readlines()
        print(f"  Line 5 after:  {lines[4].strip()}")
    print()
    print("Ready to flash with OpenOCD!")

if __name__ == '__main__':
    if len(sys.argv) != 4:
        print(f"Usage: {sys.argv[0]} <input.hex> <output.hex> <app_props_addr>")
        print(f"Example: {sys.argv[0]} firmware.hex firmware_patched.hex 0x08006220")
        sys.exit(1)

    input_hex = sys.argv[1]
    output_hex = sys.argv[2]
    app_props_addr = int(sys.argv[3], 16)

    patch_word13(input_hex, output_hex, app_props_addr)
