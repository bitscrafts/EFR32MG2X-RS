#!/usr/bin/env python3
"""One memory layout per workspace, and it is not the whole flash.

Two defects lived here until 2026-11-27 and the second hid the first:

  * `efr32mg2x-pac`, `efr32mg2x-hal`, `embassy-efr32` and `xiao-mg24-bsp` each copied their own `memory.x` into a
    directory they then put on the linker search path. Two candidate layouts, and the linker takes the first match --
    so a product's reservation was decided by dependency order, not by the program;
  * the two files disagreed about the top 40 KB. The PAC's said `LENGTH = 1512K`, which is all the flash after the
    bootloader; the HAL's and the BSPs' said `1472K`, which leaves `0x0817_6000..0x0819_FFFF` to NVM3, where the
    Bluetooth stack keeps bonds. The PAC's came first in every graph, so **the reservation was silently ignored**: a
    program that grew into that window would have written over the pairing state of every bonded phone instead of
    failing to link.

Now one crate provides the layout and it reserves the window; a consumer that wants the whole flash passes its own
`-Tmemory.x`, which is a decision in one visible place.

Exit code 0 when there is exactly one provider outside the excluded bootloader apps, and its layout reserves NVM3.
"""
from __future__ import annotations

import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
# The Gecko bootloader test app is workspace-excluded on purpose: it links against a different bootloader offset
# (0x08004000) and is built by hand.
ALLOWED = {"efr32mg2x-pac", "bootloader-test-app"}
NVM3_TOP = 0x0817_6000
CHIP_TOP = 0x0818_0000  # 1536 KB = 0x180000, measured from 0x08000000


def flash_of(path: pathlib.Path) -> tuple[int, int] | None:
    text = path.read_text(errors="replace")
    match = re.search(r"FLASH\s*:\s*ORIGIN\s*=\s*(0x[0-9A-Fa-f_]+)\s*,\s*LENGTH\s*=\s*(\d+)K", text)
    if match is None:
        return None
    return int(match.group(1).replace("_", ""), 16), int(match.group(2)) * 1024


def main() -> int:
    problems: list[str] = []

    providers = sorted(p for p in REPO.glob("crates/*/memory.x"))
    names = {p.parent.name for p in providers}
    for extra in sorted(names - ALLOWED):
        problems.append(f"{extra} ships a memory.x; the layout is provided once, by efr32mg2x-pac")
    if "efr32mg2x-pac" not in names:
        problems.append("efr32mg2x-pac no longer ships memory.x, so nothing provides a layout")

    # A build script that copies a memory.x is a provider even without the file at the crate root (it could be
    # generated), so the same allowlist applies to the scripts.
    for script in sorted(REPO.glob("crates/*/build.rs")):
        text = script.read_text(errors="replace")
        code = "\n".join(line for line in text.splitlines() if not line.lstrip().startswith(("//", "/*", "*")))
        if "memory.x" in code and script.parent.name not in ALLOWED:
            problems.append(f"{script.parent.name}/build.rs puts a memory.x on the link path")

    pac = REPO / "crates/efr32mg2x-pac/memory.x"
    if pac.exists():
        layout = flash_of(pac)
        if layout is None:
            problems.append("efr32mg2x-pac/memory.x has no FLASH line this script can read")
        else:
            origin, length = layout
            top = origin + length
            if origin != 0x0800_6000:
                problems.append(f"PAC layout starts at {origin:#x}, not at the bootloader's end 0x08006000")
            if top != NVM3_TOP:
                problems.append(
                    f"PAC layout ends at {top:#x}; it must end at {NVM3_TOP:#x}, leaving "
                    f"{(CHIP_TOP - NVM3_TOP) // 1024} KB for NVM3"
                )

    if problems:
        print(f"  FAIL  one layout: {len(problems)} problem(s)")
        for problem in problems:
            print(f"        {problem}")
        return 1

    origin, length = flash_of(pac)  # type: ignore[misc]
    print(
        f"  PASS  one layout: efr32mg2x-pac only, {origin:#x} + {length // 1024}K, "
        f"{(CHIP_TOP - (origin + length)) // 1024} KB reserved for NVM3"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
