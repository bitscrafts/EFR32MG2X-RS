#!/usr/bin/env python3
"""Inventory every PAC example: which registers it touches, and with what API.

Purpose: the board was bricked once by a blind register write (HAL HFXO path
and, earlier, raw-pointer CMU pokes at the wrong addresses). Before anything
else is flashed, every example must be classified as
  TYPED   - uses PAC typed accessors (offsets from the SVD, safe to reason about)
  RAW     - uses write_volatile to a hardcoded address (needs address verification)
and flagged if it touches clock-source selection (the bricking class).
"""
from __future__ import annotations

import pathlib
import re

REPO = pathlib.Path(__file__).resolve().parent.parent
EXAMPLES = REPO / "crates" / "efr32mg24-pac" / "examples"

DANGER = re.compile(r"\b(sysclkctrl|oscencmd|hfxo|hfrco|clksel|dpll|emu|wdog|lock)\b", re.I)
RAW = re.compile(r"write_volatile|read_volatile|core::ptr::write|0x[0-9A-Fa-f_]{6,}")
TYPED = re.compile(r"pac::Peripherals|\.clken0\(\)|\.port[a-z]_model\(\)|\.port[a-z]_dout\(\)|gpio_s\.|cmu_s\.")


def main() -> None:
    for src in sorted(EXAMPLES.glob("*.rs")):
        text = src.read_text(encoding="utf-8", errors="replace")
        api = "RAW" if RAW.search(text) else ("TYPED" if TYPED.search(text) else "?")
        danger = sorted({m.group(0).lower() for m in DANGER.finditer(text)})
        addrs = sorted({a.strip() for a in re.findall(r"0x[0-9A-Fa-f_]{6,}", text)})
        print(f"{src.stem:<22} api={api:<6} danger={','.join(danger) or '-':<34} addrs={','.join(addrs[:6]) or '-'}")


if __name__ == "__main__":
    main()
