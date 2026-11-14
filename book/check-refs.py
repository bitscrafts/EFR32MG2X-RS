#!/usr/bin/env python3
"""Check that the book's references point at something real.

    python3 book/check-refs.py [--verbose]

Two classes, both silent in a PDF until you look at the printed page:

  * ``\\ref``/``\\cref``/``\\pageref``/``\\autoref`` with no matching ``\\label`` -- LaTeX prints "??" and carries
    on. This project had one: the text pointed at a section that had been merged away.
  * a repository path named in prose (``crates/…/lib.rs``, ``scripts/gate.sh``) that does not exist. A book that
    tells a reader to open a file that is not there is worse than one that says nothing.

Cross-checked against every ``.tex`` under ``book/latex``, so a label defined in one chapter and used in another
is fine.
"""
import re
import sys
from pathlib import Path

BOOK = Path(__file__).resolve().parent
REPO = BOOK.parent
LATEX = BOOK / "latex"

LABEL = re.compile(r"\\label\{([^}]*)\}")
REF = re.compile(r"\\(?:ref|cref|Cref|pageref|autoref|eqref)\{([^}]*)\}")
# A path a reader could be told to open: at least one slash and a known source extension.
PATH = re.compile(r"(?<![\w/.-])((?:crates|scripts|docs|book|libraries)/[\w./-]+\.(?:rs|toml|sh|py|cfg|md|tex))")


def main():
    verbose = "--verbose" in sys.argv
    tex = sorted(LATEX.rglob("*.tex"))
    labels, refs, paths = {}, {}, {}
    for f in tex:
        text = f.read_text(errors="replace")
        for m in LABEL.finditer(text):
            labels.setdefault(m.group(1), f"{f.relative_to(LATEX)}")
        for m in REF.finditer(text):
            # \cref{a,b} takes a list
            for name in m.group(1).split(","):
                refs.setdefault(name.strip(), []).append(f"{f.relative_to(LATEX)}")
        for m in PATH.finditer(text):
            paths.setdefault(m.group(1), set()).add(f"{f.relative_to(LATEX)}")

    dangling = {r: v for r, v in refs.items() if r not in labels}
    def resolves(path):
        if "..." not in path:
            return (REPO / path).exists()
        # An abbreviated path ("chapters/.../01.08-x.tex"): the file has to exist somewhere, and the tail has to
        # match, or the abbreviation is hiding a file that was renamed.
        name = path.rsplit("/", 1)[-1]
        return any(p.name == name for p in REPO.rglob(name))

    missing_files = {p: v for p, v in paths.items() if not resolves(p)}
    unused = {l: f for l, f in labels.items() if l not in refs}

    print(f"  labels defined: {len(labels)}   references used: {len(refs)}   paths named: {len(paths)}")
    print(f"  references with no label: {len(dangling)}")
    for r, where in sorted(dangling.items()):
        print(f"    DANGLING {r}  (used in {', '.join(sorted(set(where)))})")
    print(f"  paths that do not exist: {len(missing_files)}")
    for p, where in sorted(missing_files.items()):
        print(f"    MISSING  {p}  (named in {', '.join(sorted(where))})")
    if verbose:
        print(f"  labels never referenced: {len(unused)}")
        for l, where in sorted(unused.items()):
            print(f"    unused   {l}  ({where})")

    bad = len(dangling) + len(missing_files)
    print("VERDICT: PASS" if not bad else f"VERDICT: FAIL ({bad} broken reference(s))")
    return 0 if not bad else 1


if __name__ == "__main__":
    sys.exit(main())
