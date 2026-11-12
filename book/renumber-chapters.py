"""Renumber every chapter of the book in physical order.

The book grew by insertion: Part III added four wireless chapters after the numbering had been
planned, testing was written next to a placeholder that already claimed chapter 14, and the BSP
chapter was slotted in between. The result was a duplicate label (`ch:chapter-14' twice) and a
`\\chapthumb' that disagreed with the printed number.

This script assigns 1..N in the order the chapters actually appear, rewrites the label and the
thumbnail of every chapter (inline or included), and prints the mapping so the result can be checked
against the table of contents.
"""

import pathlib
import re

BOOK = pathlib.Path(__file__).resolve().parent
LATEX = BOOK / "latex"
MAIN = LATEX / "main.tex"


def main() -> int:
    text = MAIN.read_text()
    lines = text.split("\n")
    out = []
    n = 0
    i = 0
    mapping = []

    while i < len(lines):
        line = lines[i]

        # a chapter written in this file
        if line.startswith("\\chapter{"):
            n += 1
            out.append(line)
            i += 1
            # replace the first following \label{ch:...} and \chapthumb{...}
            while i < len(lines) and not lines[i].startswith("\\chapter{"):
                if lines[i].startswith("\\label{ch:"):
                    out.append(f"\\label{{ch:chapter-{n:02d}}}")
                    mapping.append((n, line[9:].rstrip("}")))
                    i += 1
                    continue
                if lines[i].startswith("\\chapthumb{"):
                    out.append(f"\\chapthumb{{{n}}}")
                    i += 1
                    continue
                if lines[i].startswith("\\input{chapters/"):
                    break
                out.append(lines[i])
                i += 1
            continue

        # a chapter that lives in its own directory
        if line.startswith("\\input{chapters/chapter-") and "chapter-" in line:
            n += 1
            target = re.search(r"\\input\{(chapters/[^}]+)\}", line)
            out.append(line)
            if target:
                path = LATEX / (target.group(1) + ".tex")
                if path.exists():
                    body = path.read_text()
                    body = re.sub(r"\\label\{ch:[^}]*\}", f"\\\\label{{ch:chapter-{n:02d}}}", body, count=1)
                    body = re.sub(r"\\chapthumb\{[^}]*\}", f"\\\\chapthumb{{{n}}}", body, count=1)
                    path.write_text(body)
                    title = re.search(r"\\chapter\{([^}]*)\}", body)
                    mapping.append((n, title.group(1) if title else path.stem))
            i += 1
            continue

        out.append(line)
        i += 1

    MAIN.write_text("\n".join(out))

    print(f"{n} chapters, renumbered in physical order:")
    for number, title in mapping:
        print(f"  {number:2d}. {title}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
