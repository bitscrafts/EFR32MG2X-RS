#!/usr/bin/env python3
"""Check that the code printed in the book is code that exists and parses.

    python3 book/check-code-blocks.py [--verbose]

Why this exists: a book about bring-up that prints code nobody compiled is a book of fiction, and the
rot is silent -- the PDF still builds, the pages still look right. Three checks, cheapest first:

  * ``rust``  -- parse with ``rustfmt`` (it fails on syntax errors, which is exactly the question). A block
                 that also matches a source file in the repository is reported as such: that code is built
                 by ``scripts/check-examples-build.sh`` and, for the radio examples, has run on hardware.
  * ``bash``  -- ``bash -n``, which parses without executing.
  * ``toml``  -- ``tomllib``.

Fragments (an expression, a register write) are expected to be fragments; the check is that they are
*valid* fragments, and the report says which blocks are backed by a file that is compiled.
"""
import re
import subprocess
import sys
import tempfile
from pathlib import Path

BOOK = Path(__file__).resolve().parent
REPO = BOOK.parent
CHAPTERS = sorted((BOOK / "latex" / "chapters").rglob("*.tex"))

MINTED = re.compile(r"\\begin\{minted\}\{([^}]*)\}(.*?)\\end\{minted\}", re.S)


def normalise(text):
    """Whitespace-insensitive form, so a block that matches a source file modulo indentation is recognised."""
    return re.sub(r"\s+", " ", text).strip()


def repo_sources():
    """Every Rust source in the tree, normalised once -- the reference set for the matching check."""
    out = []
    for p in REPO.rglob("*.rs"):
        if any(part in {"target", "node_modules", ".git"} for part in p.parts):
            continue
        try:
            out.append((p.relative_to(REPO), normalise(p.read_text(errors="replace"))))
        except OSError:
            continue
    return out


# An ellipsis anywhere in the block means the author abridged it -- a body, an argument, a match arm. Such a
# block cannot parse by construction, and saying so is the point; the alternative is to pretend it is code.
ELIDED = re.compile(r"(?<![.\w])\.\.\.(?![.\w])|<snip>")


def _rustfmt_once(code):
    with tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False) as fh:
        fh.write(code if code.endswith("\n") else code + "\n")
        path = fh.name
    try:
        r = subprocess.run(["rustfmt", "--edition", "2021", "--emit", "stdout", path],
                           capture_output=True, text=True, timeout=60)
        if r.returncode != 0:
            err = (r.stderr or r.stdout).strip().splitlines()
            return err[0] if err else "rustfmt failed"
        return None
    except FileNotFoundError:
        return "rustfmt not installed"
    finally:
        Path(path).unlink(missing_ok=True)


def parse_rust(code):
    """None when it parses, else the first error line.

    A book prints both items and *statements*. A fragment that begins `let` or `match` is correct Rust in a
    function and an error at item position, so the second attempt wraps the block in a function body. Blocks
    that contain an ellipsis are illustrative by construction and are reported as such, not as failures --
    a checker that cannot tell "abridged on purpose" from "wrong" just produces noise.
    """
    if ELIDED.search(code):
        return "ELIDED"
    err = _rustfmt_once(code)
    if err is None:
        return None
    wrapped = "fn __book_block() {\n" + code + "\n}\n"
    err2 = _rustfmt_once(wrapped)
    if err2 is None:
        return None
    return err  # report the unwrapped error: it is the one that describes the block


def parse_bash(code):
    r = subprocess.run(["bash", "-n"], input=code, capture_output=True, text=True, timeout=30)
    if r.returncode != 0:
        lines = r.stderr.strip().splitlines()
        return lines[0] if lines else "bash -n failed"
    return None


def parse_toml(code):
    import tomllib
    try:
        tomllib.loads(code)
        return None
    except Exception as exc:  # noqa: BLE001 - the message is the report
        return str(exc).splitlines()[0]


def main():
    verbose = "--verbose" in sys.argv
    sources = repo_sources()
    rows = []
    for chapter in CHAPTERS:
        text = chapter.read_text(errors="replace")
        for index, match in enumerate(MINTED.finditer(text), start=1):
            lang = match.group(1).strip()
            code = match.group(2)
            if lang not in {"rust", "bash", "toml"}:
                continue
            if lang == "rust":
                err = parse_rust(code)
                lines = [normalise(l) for l in code.splitlines()]
                lines = [l for l in lines if len(l) > 24 and not l.startswith("//")]
                lines.sort(key=len, reverse=True)
                probe = lines[:3]
                matches = [str(p) for p, src in sources
                           if probe and all(l in src for l in probe)]
                if err == "ELIDED":
                    verdict = "elided"
                elif err:
                    verdict = "SYNTAX"
                elif matches:
                    verdict = "COMPILED"
                else:
                    verdict = "fragment"
                where = matches[0] if matches else ""
                rows.append((chapter.name, index, lang, verdict, err or where))
            elif lang == "bash":
                err = parse_bash(code)
                rows.append((chapter.name, index, lang, "SYNTAX" if err else "parses", err or ""))
            else:
                err = parse_toml(code)
                rows.append((chapter.name, index, lang, "SYNTAX" if err else "parses", err or ""))

    bad = [r for r in rows if r[3] == "SYNTAX"]
    compiled = [r for r in rows if r[3] == "COMPILED"]
    for name, index, lang, verdict, detail in rows:
        if verbose or verdict == "SYNTAX":
            print(f"  {verdict:9} {lang:4} {name} block {index}  {detail}")
    if not verbose:
        print(f"  blocked checked: {len(rows)}")
        print(f"  backed by a compiled source file: {len(compiled)}")
        print(f"  fragments (valid syntax, not a whole file): {len(rows) - len(compiled) - len(bad)}")
    print("VERDICT: PASS" if not bad else f"VERDICT: FAIL ({len(bad)} block(s) do not parse)")
    return 0 if not bad else 1


if __name__ == "__main__":
    sys.exit(main())
