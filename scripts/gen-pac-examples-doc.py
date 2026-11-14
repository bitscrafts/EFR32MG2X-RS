#!/usr/bin/env python3
# Generate crates/efr32mg24-pac/docs/examples.md from the examples themselves.
#
# The list is generated because a hand-written list of 27 examples goes stale the first time one is renamed, and a
# documentation page that names examples which no longer exist is worse than no page. The gate runs this with --check,
# so the page cannot drift from the tree.
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "crates" / "efr32mg24-pac"
OUT = CRATE / "docs" / "examples.md"

HEAD = """## Examples

Every file here compiles for the target in CI. The date in the last column is the day the example was watched running
on the board itself, not merely built -- an example nobody has ever run is an example that does not work.

| example | what it shows | run on hardware |
|---|---|---|
"""


def describe(path):
    text = path.read_text()
    lines = []
    for line in text.splitlines():
        if not line.startswith("//!"):
            break
        lines.append(line[3:].strip())
    title = lines[0] if lines else ""
    body = " ".join(lines)
    m = re.search(r"VERIFIED ON HARDWARE \(([0-9-]+)\)", body)
    return title.rstrip("."), (m.group(1) if m else "--")


def main():
    check = "--check" in sys.argv
    rows = []
    for p in sorted((CRATE / "examples").glob("*.rs")):
        title, when = describe(p)
        if not title:
            title = p.stem.replace("_", " ")
        rows.append(f"| `{p.stem}` | {title} | {when} |")

    smallest = min((CRATE / "examples").glob("*.rs"), key=lambda p: p.stat().st_size)
    source = smallest.read_text().rstrip()

    body = HEAD + "\n".join(rows) + "\n\n"
    body += f"### One of them, in full: `{smallest.stem}`\n\n"
    body += "```rust,ignore\n" + source + "\n```\n"

    if check:
        current = OUT.read_text() if OUT.exists() else ""
        if current != body:
            print("FAIL examples.md is out of date (run scripts/gen-pac-examples-doc.py)")
            return 1
        print(f"OK examples.md matches the {len(rows)} examples in the tree")
        return 0

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(body)
    print(f"wrote {OUT.relative_to(ROOT)}: {len(rows)} examples, full source of {smallest.stem}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
