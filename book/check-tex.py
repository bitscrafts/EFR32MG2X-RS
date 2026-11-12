#!/usr/bin/env python3
"""Refuse the LaTeX mistakes that this book has already paid for.

Each check below corresponds to a defect that produced a build which either failed with a message
naming an unrelated macro, or -- worse -- succeeded while dropping content. Run it before every
compile; `book/BUILD.md` runs it as step 2.

Exit code is the number of defects found (0 = clean).
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent


def tex_files():
    yield from sorted(ROOT.rglob("*.tex"))


def strip_comments(text):
    """Drop `%` comments so a commented-out example is not reported as a defect."""
    return "\n".join(re.sub(r"(?<!\\)%.*$", "", line) for line in text.splitlines())


def check_codeinline_in_argument(text, name):
    """`\\codeinline` is verbatim-like: it cannot appear inside another command's argument."""
    defects = []
    for cmd in ("section", "subsection", "subsubsection", "caption", "chapter"):
        for m in re.finditer(r"\\" + cmd + r"\*?\{", text):
            start = m.end()
            depth, i = 1, start
            while i < len(text) and depth:
                if text[i] == "{":
                    depth += 1
                elif text[i] == "}":
                    depth -= 1
                i += 1
            arg = text[start : i - 1]
            if "\\codeinline{" in arg:
                line = text[: m.start()].count("\n") + 1
                defects.append(
                    f"{name}:{line}: \\codeinline inside \\{cmd}{{}} -- "
                    f"use plain text (the error names '\\oldstylenums' and points elsewhere)"
                )
    return defects


def check_minted_balance(text, name):
    """A `minted` block opened and closed with fences, or not closed at all, loses content."""
    defects = []
    opened = None
    for n, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith("\\begin{minted}"):
            if opened is not None:
                defects.append(f"{name}:{n}: nested \\begin{{minted}} (opened at line {opened})")
            opened = n
        elif stripped.startswith("\\end{minted}"):
            if opened is None:
                defects.append(f"{name}:{n}: \\end{{minted}} without a matching \\begin")
            opened = None
        elif opened is not None and stripped.startswith("```"):
            defects.append(
                f"{name}:{n}: markdown fence inside minted (opened at line {opened}) -- "
                f"use \\end{{minted}}"
            )
    if opened is not None:
        defects.append(f"{name}:{opened}: \\begin{{minted}} never closed")
    return defects


def check_tree_glyphs(text, name):
    """Box-drawing characters need xelatex plus the preamble's font setup; flag them for review."""
    if any(g in text for g in "─│┌┐└┘├┤"):
        if "\\begin{minted}" not in text:
            return [
                f"{name}: box-drawing characters outside a minted block -- "
                f"they will vanish in the body font"
            ]
    return []


def main():
    defects = []
    for path in tex_files():
        name = path.relative_to(ROOT).as_posix()
        text = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
        defects += check_codeinline_in_argument(text, name)
        defects += check_minted_balance(text, name)
        defects += check_tree_glyphs(text, name)

    for d in defects:
        print("DEFECT " + d)
    print(f"{len(defects)} defect(s) across {len(list(tex_files()))} files")
    return min(len(defects), 125)


if __name__ == "__main__":
    sys.exit(main())
