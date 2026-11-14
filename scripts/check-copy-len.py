#!/usr/bin/env python3
"""Find `slice[at..at + N].copy_from_slice(b"...")` where N is not the literal's length.

Why this exists: three separate examples panicked at run time on exactly this mistake --

    msg[at..at + 6].copy_from_slice(b" len=")     # six into five
    msg[at..at + 15].copy_from_slice(b"gatt: ...") # fifteen into sixteen
    msg[at..at + 5].copy_from_slice(b"c p=")       # five into four

Each one compiled, each one was found only by running the firmware on the board, and each one cost a flash cycle.
The mistake is invisible to the compiler because the length is a separate literal from the bytes, so it belongs in a
check.

Exit code 0 when nothing is found, 1 when something is. Prints one line per defect:

    path:line: N=6 but the literal is 5 bytes: b" len="
"""
import re
import sys
from pathlib import Path

PATTERN = re.compile(
    r'\[\s*\w+\s*\.\.\s*\w+\s*\+\s*(?P<n>\d+)\s*\]\s*\.\s*copy_from_slice\s*\(\s*b"(?P<lit>(?:[^"\\]|\\.)*)"',
)


def strip_comments(text: str) -> str:
    """Blank out `//` comments, keeping newlines so line numbers stay true.

    Without this the checker reports the doc comment in `common/mailbox.rs` that *explains* this very mistake -- a
    check that fails on its own documentation is a check somebody will disable.
    """
    out = []
    for line in text.split("\n"):
        idx = line.find("//")
        if idx >= 0:
            # A `//` inside a string literal would be misread, but the pattern this checker looks for is a numeric
            # slice bound followed by `copy_from_slice`, which never contains a string before the comment marker in
            # practice; the risk of a false negative is cheaper than the false positive above.
            line = line[:idx]
        out.append(line)
    return "\n".join(out)


def literal_len(lit: str) -> int:
    """Length in bytes of the contents of a Rust byte-string literal (escapes included, not interpreted)."""
    out = 0
    i = 0
    while i < len(lit):
        if lit[i] == "\\" and i + 1 < len(lit):
            i += 2
        else:
            i += 1
        out += 1
    return out


def main() -> int:
    roots = [Path(a) for a in sys.argv[1:]] or [Path("crates")]
    defects = 0
    checked = 0
    for root in roots:
        files = [root] if root.is_file() else sorted(root.rglob("*.rs"))
        for path in files:
            try:
                text = path.read_text(errors="replace")
            except OSError:
                continue
            for m in PATTERN.finditer(strip_comments(text)):
                checked += 1
                n = int(m.group("n"))
                want = literal_len(m.group("lit"))
                if n != want:
                    line = text.count("\n", 0, m.start()) + 1
                    print(f'{path}:{line}: N={n} but the literal is {want} bytes: b"{m.group("lit")}"')
                    defects += 1
    print(f"{defects} defect(s) across {checked} copy(s) checked")
    return 1 if defects else 0


if __name__ == "__main__":
    sys.exit(main())
