#!/usr/bin/env python3
"""
Find vector tables in Intel HEX file by searching for ApplicationProperties pointers.

Based on Silicon Labs engineer guidance:
"We use unused exception vector (13) to point to the application properties"

Usage:
    python3 start-page.py firmware.hex 0x08002834 0x080c7cd4

This will search the hex file for vector tables that point to the specified
ApplicationProperties addresses.
"""

import sys

def parse_intel_hex(filename):
    """Parse Intel HEX file and return memory map"""
    memory = {}
    extended_addr = 0

    with open(filename, 'r') as f:
        for line in f:
            line = line.strip()
            if not line.startswith(':'):
                continue

            byte_count = int(line[1:3], 16)
            address = int(line[3:7], 16)
            record_type = int(line[7:9], 16)
            data = line[9:9+byte_count*2]

            if record_type == 0:  # Data record
                full_addr = extended_addr + address
                for i in range(0, len(data), 2):
                    byte_val = int(data[i:i+2], 16)
                    memory[full_addr + i//2] = byte_val
            elif record_type == 4:  # Extended linear address
                extended_addr = int(data, 16) << 16

    return memory

def find_vector_tables(hex_file, *app_props_addrs):
    """Find vector tables pointing to ApplicationProperties addresses"""
    print(f"Parsing file {hex_file}...")
    memory = parse_intel_hex(hex_file)

    # Convert app_props_addrs to integers if they're strings
    target_addrs = []
    for addr in app_props_addrs:
        if isinstance(addr, str):
            target_addrs.append(int(addr, 16))
        else:
            target_addrs.append(addr)

    # Search for vector tables
    # Vector table starts with SP (word 0) and PC (word 1)
    # Word 13 is at offset 0x34 (52 decimal)

    for addr in target_addrs:
        addr_bytes = addr.to_bytes(4, 'little')

        # Search memory for this address
        for mem_addr in range(0, max(memory.keys()) if memory else 0, 4):
            # Check if word 13 (offset 0x34) contains our target address
            word13_addr = mem_addr + 0x34

            if all(memory.get(word13_addr + i) == addr_bytes[i] for i in range(4)):
                # Found a potential vector table!
                # Verify it looks like a vector table (check SP and PC)
                sp = sum(memory.get(mem_addr + i, 0) << (8*i) for i in range(4))
                pc = sum(memory.get(mem_addr + 4 + i, 0) << (8*i) for i in range(4))

                # SP should be in RAM (0x20000000 range)
                # PC should be in flash and have Thumb bit set
                if 0x20000000 <= sp <= 0x2003FFFF and (pc & 1) and 0x08000000 <= (pc & ~1) <= 0x08180000:
                    print(f"vector table located at 0x{mem_addr:08X} pointing to application properties at 0x{addr:08X}")

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <firmware.hex> <app_props_addr1> [app_props_addr2] ...")
        print(f"Example: {sys.argv[0]} firmware.hex 0x08002834 0x080c7cd4")
        sys.exit(1)

    hex_file = sys.argv[1]
    app_props_addrs = sys.argv[2:]

    find_vector_tables(hex_file, *app_props_addrs)
