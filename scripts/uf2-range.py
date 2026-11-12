#!/usr/bin/env python3
"""Print the flash range a UF2 file would write, and refuse anything unsafe.

A UF2 is not a black box: every 512-byte block carries its target address and payload length. So the
same discipline used for the EFR32 images (scripts/elf-load-range.py) applies to the Nordic board:
read the addresses out of the file, then decide.

On the Adafruit Feather nRF52840 Express:
    0x00000000 .. 0x00025FFF   MBR and the circuit's own lower region
    0x00026000 .. 0x000F3FFF   application  <- the only range we may write
    0x000F4000 .. 0x000FFFFF   UF2 bootloader and its settings  <- NEVER touch

Writing into the bootloader region is how these boards get bricked: the bootloader is what accepts
UF2 files in the first place, so losing it means losing the only way in without SWD.

Usage:
    python3 scripts/uf2-range.py <file.uf2> [--board feather-nrf52840-express]

Exits 0 when the file is inside the application region, 1 otherwise.
"""

import struct
import sys

MAGIC0 = 0x0A324655
MAGIC1 = 0x9E5D5157
MAGIC_END = 0x0AB16F30
BLOCK = 512

# Board memory maps. Only the ranges we have evidence for; adding a board means measuring its
# bootloader, not guessing.
BOARDS = {
    "feather-nrf52840-express": (0x00026000, 0x000F4000),
}


def blocks(path):
    data = open(path, "rb").read()
    if len(data) % BLOCK:
        raise SystemExit(f"{path}: size {len(data)} is not a multiple of {BLOCK}")
    for i in range(0, len(data), BLOCK):
        b = data[i : i + BLOCK]
        m0, m1, _flags, addr, size, _no, _total, _fsize = struct.unpack("<8I", b[:32])
        end_magic = struct.unpack("<I", b[-4:])[0]
        if m0 != MAGIC0 or m1 != MAGIC1 or end_magic != MAGIC_END:
            raise SystemExit(f"{path}: block {i // BLOCK} is not a UF2 block (bad magic)")
        if size > 476:
            raise SystemExit(f"{path}: block {i // BLOCK} claims {size} payload bytes (max 476)")
        yield addr, size


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 1
    path = sys.argv[1]
    board = "feather-nrf52840-express"
    if "--board" in sys.argv:
        board = sys.argv[sys.argv.index("--board") + 1]
    if board not in BOARDS:
        raise SystemExit(f"unknown board {board}; known: {', '.join(BOARDS)}")
    low, high = BOARDS[board]

    ranges = []
    for addr, size in blocks(path):
        if size:
            ranges.append((addr, addr + size))
    if not ranges:
        raise SystemExit(f"{path}: contains no payload")

    ranges.sort()
    merged = [list(ranges[0])]
    for start, end in ranges[1:]:
        if start <= merged[-1][1]:
            merged[-1][1] = max(merged[-1][1], end)
        else:
            merged.append([start, end])

    load_min = merged[0][0]
    load_max = merged[-1][1] - 1
    total = sum(e - s for s, e in merged)

    print(f"file={path}")
    print(f"board={board} app_region=0x{low:08X}..0x{high - 1:08X}")
    print(f"load_min=0x{load_min:08X} load_max=0x{load_max:08X} size={total} bytes segments={len(merged)}")

    bad = [r for r in merged if r[0] < low or r[1] > high]
    if bad:
        for s, e in bad:
            if e <= low:
                where = "below the application region"
            elif s >= high:
                where = "INSIDE THE BOOTLOADER REGION -- flashing this can brick the board"
            else:
                where = "crosses the application boundary"
            print(f"REFUSED 0x{s:08X}..0x{e - 1:08X} {where}")
        print("VERDICT: REFUSED")
        return 1

    print("VERDICT: SAFE (application region only)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
