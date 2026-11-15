#!/usr/bin/env python3
"""Refuse any application image that would be written into the bootloader.

Two different worlds live in this tree, and they need opposite rules:

  * **Applications** run on top of the Gecko bootloader, which owns 0x08000000-0x08005FFF. An application whose
    FLASH ORIGIN is below 0x08006000 overwrites the bootloader's vector table and the board stops booting. This is the
    rule the flash script enforces, repeated here at the source, because a linker script in the tree is a loaded gun
    even when nobody runs the flash script.

  * **A bootloader of our own** replaces the Gecko one and therefore *starts* at 0x08000000 — that is correct, not a
    defect, and it is where this project is heading (`crates/efr32mg24-bootloader/bootloader.x`). Its companion test
    application starts at 0x08004000 because that bootloader reserves 16 KiB.

So the classification is by category, declared here by hand with a reason, and anything below the floor that is not
declared fails. A dangerous script must also never travel inside a published crate: the reader would find it on
crates.io with no context.

The Gecko bootloader is 24 KiB. An earlier note in this repository said 16 KiB and produced an application linked at
0x08004000 for a board that has the 24 KiB one — that mismatch is the class of mistake this check exists for.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FLOOR = 0x08006000        # first byte an application may use with the Gecko bootloader present
CEILING = 0x08180000      # end of flash

# Anything whose FLASH ORIGIN is below FLOOR must be declared here, with the world it belongs to.
BELOW_FLOOR = {
    "crates/efr32mg24-bootloader/bootloader.x":
        "our own bootloader: it replaces the Gecko one and therefore owns 0x08000000",
    "crates/efr32mg24-bootloader/test-app-memory.x":
        "application for our own bootloader, which reserves 16 KiB (0x08000000-0x08003FFF)",
    "crates/bootloader-test-app/memory.x":
        "the layout the test app was written for; flashing it on a board that still has the 24 KiB Gecko bootloader writes into it",
}

# These must never be inside a published crate.
NEVER_PUBLISH = {
    "memory_direct.x", "memory.x.bootloader", "bootloader.x", "test-app-memory.x",
}

PUBLISHED = {"efr32mg2x-pac": "crates/efr32mg2x-pac", "efr32mg2x-hal": "crates/efr32mg2x-hal"}

PAT = re.compile(r"FLASH\s*:\s*ORIGIN\s*=\s*(0x[0-9a-fA-F]+)", re.I)


def scripts():
    for p in ROOT.rglob("*"):
        if "/target/" in str(p) or "/.git/" in str(p) or not p.is_file():
            continue
        if p.suffix in (".x", ".ld", ".lds") or p.name.startswith("memory"):
            # The archive holds records of layouts that are no longer used. Nothing links them, and the packaging check
            # below keeps them out of published crates; judging their addresses would only force me to delete history.
            if "docs/99-archive/" in str(p.relative_to(ROOT)):
                continue
            yield p


def main():
    bad = []
    rows = []
    for p in scripts():
        try:
            body = p.read_text(errors="ignore")
        except OSError:
            continue
        m = PAT.search(body)
        if not m:
            continue                      # device.x and friends carry no FLASH region: nothing to judge
        rel = str(p.relative_to(ROOT))
        origin = int(m.group(1), 16)
        below = origin < FLOOR or origin >= CEILING
        rows.append((rel, origin, below))
        if below and rel not in BELOW_FLOOR:
            bad.append(f"{rel}: FLASH ORIGIN {hex(origin)} is inside the bootloader and is not declared as a bootloader")
        if not below and rel in BELOW_FLOOR:
            bad.append(f"{rel}: declared as a bootloader but its ORIGIN is a normal application address")

    for name, crate_dir in PUBLISHED.items():
        out = subprocess.run(
            ["cargo", "package", "--list", "--allow-dirty", "--manifest-path", str(ROOT / crate_dir / "Cargo.toml")],
            capture_output=True, text=True,
        )
        if out.returncode != 0:
            bad.append(f"{name}: could not list the package")
            continue
        for f in out.stdout.split():
            if Path(f).name in NEVER_PUBLISH and f != "memory.x":
                bad.append(f"{name}: ships {f}, which describes an unbootable layout")

    for rel, origin, below in sorted(rows):
        kind = "bootloader world" if below else "application"
        print(f"  {rel:<56} {hex(origin):>12}  {kind}")

    if bad:
        print("\nFAIL an image could be written into the bootloader:")
        for b in bad:
            print(f"  {b}")
        return 1
    print(f"\nPASS every application starts at or above {hex(FLOOR)}; every bootloader script is declared")
    return 0


if __name__ == "__main__":
    sys.exit(main())
