#!/usr/bin/env python3
"""Rewrite crate paths for the crates/ layout, in living docs and scripts only.

Historical records (CHANGELOG.md, docs/03-development/LOG.md, docs/99-archive/,
book*/, .archive/) are deliberately left untouched: they describe where files
were at the time, and rewriting them would falsify the record.

Path-like occurrences only: a crate directory name followed by "/" and not
already prefixed with "crates/".
"""
from __future__ import annotations

import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
CRATES = [
    "efr32mg24-pac",
    "efr32mg24-hal",
    "efr32mg24-rail-sys",
    "efr32mg24-bluetooth-sys",
    "efr32mg24-bootloader",
    "xiao-mg24-bsp",
    "bootloader-test-app",
]
SKIP_DIRS = {
    "crates",
    "target",
    "_tmp",
    ".git",
    "book",
    "book.backup.20260109_103322",
    ".archive",
    "docs/99-archive",
}
SKIP_FILES = {"CHANGELOG.md", "docs/03-development/LOG.md"}

PATTERN = re.compile(
    r"(?<!crates/)(?<![\w/.-])(" + "|".join(re.escape(c) for c in CRATES) + r")/"
)


def skipped(rel: pathlib.PurePath) -> bool:
    posix = rel.as_posix()
    if posix in SKIP_FILES:
        return True
    return any(posix == d or posix.startswith(d + "/") for d in SKIP_DIRS)


def main() -> int:
    changed = 0
    for path in sorted(REPO.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(REPO)
        if skipped(rel):
            continue
        if path.suffix not in {".md", ".sh", ".yml", ".yaml", ".toml", ".rs", ".py", ".json"}:
            continue
        text = path.read_text(encoding="utf-8", errors="surrogateescape")
        new = PATTERN.sub(lambda m: f"crates/{m.group(1)}/", text)
        if new != text:
            path.write_text(new, encoding="utf-8", errors="surrogateescape")
            n = len(PATTERN.findall(text))
            print(f"{n:>3}  {rel}")
            changed += 1
    print(f"\n{changed} files updated")
    return 0


if __name__ == "__main__":
    sys.exit(main())
