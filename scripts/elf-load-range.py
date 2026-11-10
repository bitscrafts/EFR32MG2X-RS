#!/usr/bin/env python3
"""Print the load address range of an ELF32 ARM executable.

Used as a hard guard before flashing: the XIAO MG24's Gecko bootloader lives at
0x08000000-0x08005FFF, and any image that loads below 0x08006000 overwrites it.
Only 32-bit little-endian ELF is supported (all Cortex-M output).

Exit codes: 0 = parsed, 2 = not a supported ELF, 3 = no loadable segments.
"""
from __future__ import annotations

import struct
import sys


class ElfError(Exception):
    pass


def load_range(path: str) -> tuple[int, int, list[tuple[int, int, int]]]:
    with open(path, "rb") as fh:
        data = fh.read()

    if data[:4] != b"\x7fELF":
        raise ElfError("not an ELF file")
    if data[4] != 1:
        raise ElfError("not ELF32")
    if data[5] != 1:
        raise ElfError("not little-endian")

    (e_phoff,) = struct.unpack_from("<I", data, 0x1C)
    (e_phentsize,) = struct.unpack_from("<H", data, 0x2A)
    (e_phnum,) = struct.unpack_from("<H", data, 0x2C)

    segments: list[tuple[int, int, int]] = []
    for i in range(e_phnum):
        off = e_phoff + i * e_phentsize
        (p_type, p_offset, p_vaddr, p_paddr, p_filesz, _memsz) = struct.unpack_from(
            "<IIIIII", data, off
        )
        if p_type == 1 and p_filesz > 0:  # PT_LOAD
            segments.append((p_paddr, p_filesz, p_vaddr))

    if not segments:
        raise ElfError("no loadable segments")

    lo = min(s[0] for s in segments)
    hi = max(s[0] + s[1] for s in segments)
    return lo, hi, segments


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: elf-load-range.py <elf>", file=sys.stderr)
        return 64
    try:
        lo, hi, segs = load_range(argv[1])
    except (OSError, ElfError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    print(f"load_min=0x{lo:08X} load_max=0x{hi - 1:08X} size={hi - lo} segments={len(segs)}")
    for paddr, filesz, vaddr in sorted(segs):
        print(f"  seg paddr=0x{paddr:08X} vaddr=0x{vaddr:08X} filesz={filesz}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
