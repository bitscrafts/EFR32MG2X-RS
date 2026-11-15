#!/usr/bin/env python3
"""A crate's README has to say what it uses from the layer below it -- not merely name the crate.

Naming the dependency is a one-line gesture that survives by accident; a reader wants to know what the
crate actually takes from below and why. So this check does two things per crate:

  1. the README must carry a section `What this crate uses from <lower>`;
  2. every module or type the source imports from that lower crate must be named somewhere in the README.

Part 2 is the part that cannot be satisfied by good intentions: the list comes from the source, so a new
import appears in the check the moment it is written. A crate that imports seven modules and documents
three fails here.

Stack checked (one direction only, as the dependency goes):

    efr32mg2x-hal                 <- efr32mg2x-pac
    efr32mg2x-xiao-bsp            <- efr32mg2x-hal  (+ efr32mg2x-pac, through it)
    efr32mg2x-expansion-base-bsp  <- efr32mg2x-hal

Exit 0 = every crate documents what it takes, 1 = something is missing (each miss is printed).
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Every crate README must send the reader to the runnable examples, the way the PAC's does: the crates
# document the API, the examples repository shows it running.
EXAMPLES_REPO = "github.com/bitscrafts/efr32mg2x-examples"

# crate directory -> {lower crate: crate directory of that lower crate}
STACK: dict[str, dict[str, str]] = {
    "crates/efr32mg2x-hal": {"efr32mg2x-pac": "crates/efr32mg2x-pac"},
    "crates/xiao-mg24-bsp": {
        "efr32mg2x-hal": "crates/efr32mg2x-hal",
        "efr32mg2x-pac": "crates/efr32mg2x-pac",
    },
    "crates/xiao-expansion-base-bsp": {"efr32mg2x-hal": "crates/efr32mg2x-hal"},
}

IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def imported(src: pathlib.Path, lower_snake: str) -> set[str]:
    """Top-level names this crate's source takes from `lower_snake` (e.g. efr32mg2x_pac).

    Two spellings count, and the second one is the one that matters in practice: crates here re-export
    the layer below as `crate::pac` / `crate::hal`, so most imports read `crate::pac::CmuS` rather than
    `efr32mg2x_pac::CmuS`. Checking only the full path found one name where the source used sixteen.
    """
    names: set[str] = set()
    alias = lower_snake.split("_")[-1]  # efr32mg2x_pac -> pac, efr32mg2x_hal -> hal
    patterns = [
        re.compile(rf"\b{lower_snake}::([A-Za-z_][A-Za-z0-9_]*)"),
        re.compile(rf"\bcrate::{alias}::([A-Za-z_][A-Za-z0-9_]*)"),
        # the bare alias, which is how most of this source actually spells it (`pac::CmuS`,
        # after `use efr32mg2x_pac as pac;` in lib.rs)
        re.compile(rf"(?<![A-Za-z0-9_:]){alias}::([A-Za-z_][A-Za-z0-9_]*)"),
    ]
    for f in src.rglob("*.rs"):
        text = f.read_text(encoding="utf-8")
        for pattern in patterns:
            names.update(m.group(1) for m in pattern.finditer(text))
    return names


def main() -> int:
    problems: list[str] = []
    for crate, lowers in STACK.items():
        readme = ROOT / crate / "README.md"
        if not readme.exists():
            problems.append(f"{crate}: no README.md")
            continue
        text = readme.read_text(encoding="utf-8")

        if EXAMPLES_REPO not in text:
            problems.append(f"{crate}/README.md: never points at {EXAMPLES_REPO}")

        for lower, lower_dir in lowers.items():
            lower_snake = lower.replace("-", "_")
            heading = f"What this crate uses from `{lower}`"
            if heading not in text:
                problems.append(f"{crate}/README.md: no section '{heading}'")
                continue

            # Only count a lower crate's own source imports; the PAC reached through the HAL is
            # covered by the HAL's own README, so it is described in prose rather than enumerated.
            used = imported(ROOT / crate / "src", lower_snake)
            undoc = sorted(n for n in used if n not in text)
            if undoc:
                problems.append(
                    f"{crate}/README.md: imports {lower} but never explains "
                    + ", ".join(f"`{n}`" for n in undoc)
                )
            else:
                print(f"OK   {crate}: documents its {lower} imports ({len(used)} names)")

    if problems:
        for p in problems:
            print(f"FAIL {p}")
        print("VERDICT: FAIL a crate does not say what it takes from the layer below")
        return 1
    print("VERDICT: PASS every crate says what it takes from the layer below")
    return 0


if __name__ == "__main__":
    sys.exit(main())
