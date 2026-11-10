#!/usr/bin/env python3
"""List ELF32 section headers with address, size and flags.

Needed because load segments hide which section actually lands in the
bootloader region: an image can have one PT_LOAD at 0x08000000 that is only a
few hundred bytes of a single misplaced section.
"""
from __future__ import annotations

import struct
import sys

SHT_NOBITS = 8
FLAGS = {1: "W", 2: "A", 4: "X"}


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: elf-sections.py <elf>", file=sys.stderr)
        return 64
    data = open(argv[1], "rb").read()
    if data[:4] != b"\x7fELF" or data[4] != 1 or data[5] != 1:
        print("error: not a 32-bit little-endian ELF", file=sys.stderr)
        return 2

    (e_shoff,) = struct.unpack_from("<I", data, 0x20)
    (e_shentsize,) = struct.unpack_from("<H", data, 0x2E)
    (e_shnum,) = struct.unpack_from("<H", data, 0x30)
    (e_shstrndx,) = struct.unpack_from("<H", data, 0x32)

    def sh(i: int) -> tuple[int, int, int, int, int, int, int]:
        return struct.unpack_from("<IIIIII", data, e_shoff + i * e_shentsize)[:7]  # type: ignore[return-value]

    hdr = sh(e_shstrndx)
    strtab_off = hdr[4]
    rows = []
    for i in range(e_shnum):
        name, stype, flags, addr, off, size, _ = struct.unpack_from(
            "<IIIIIII", data, e_shoff + i * e_shentsize
        )
        end = data.index(b"\x00", strtab_off + name)
        sname = data[strtab_off + name : end].decode("utf-8", "replace")
        rows.append((addr, sname, stype, flags, size))

    for addr, sname, stype, flags, size in sorted(rows):
        if not size or sname.startswith(".debug") or sname.startswith(".comment"):
            continue
        f = "".join(v for bit, v in FLAGS.items() if flags & bit)
        nobits = " NOBITS" if stype == SHT_NOBITS else ""
        print(f"0x{addr:08X}  {sname:<28} {size:>7}  {f}{nobits}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
