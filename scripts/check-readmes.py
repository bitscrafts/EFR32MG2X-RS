#!/usr/bin/env python3
"""Check every README in this repo against the tree it describes.

Three questions, because each has already cost real time here:

  1. does every relative link resolve?
  2. does every path it names in backticks exist?
  3. does it point at a local machine (/projects, /root, /tmp, _tmp)?

Usage: python3 scripts/check-readmes.py [--check]
Exit 1 (only with --check) when anything is wrong.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKIP = ("target/", ".git/", "vendor/", "blobs/", ".claude/")
LINK = re.compile(r"\]\(([^)#\s]+)")
TICK = re.compile(r"`([A-Za-z0-9_./-]+\.[A-Za-z0-9]+)`")
LOCAL = re.compile(r"(/projects/|/root/|/tmp/|_tmp/)")

problems = 0

# A name cited in prose is not a path: `pac_led_blink` in a README means examples/pac_led_blink.rs, and
# reporting that as missing is the false positive that makes a checker useless. A name is accepted when a
# file of that basename exists anywhere in the repo; only a name that exists nowhere is a real drift.
BASENAMES = {f.name for f in ROOT.rglob("*") if f.is_file()}
EXTENSOES = (".rs", ".md", ".toml", ".sh", ".py", ".json", ".svg", ".c", ".h", ".ld", ".csv", ".bin", ".yml")


def readmes():
    for p in sorted(ROOT.rglob("README.md")):
        rel = p.relative_to(ROOT).as_posix()
        if not any(s in rel + "/" for s in SKIP):
            yield p, rel


for path, rel in readmes():
    text = path.read_text(errors="replace")
    for m in LINK.finditer(text):
        alvo = m.group(1).strip()
        if alvo.startswith(("http://", "https://", "mailto:", "#")):
            continue
        if not (path.parent / alvo).exists() and not (ROOT / alvo).exists():
            print(f"{rel}: link does not resolve: {alvo}")
            problems += 1
    for m in TICK.finditer(text):
        nome = m.group(1)
        if not ("/" in nome or "." in nome):
            continue
        # `0.7.7` is a version and `-Tlink.x` is a linker flag, not files
        if re.fullmatch(r"\d+(\.\d+)+", nome) or nome.startswith("-"):
            continue
        if not ("/" in nome or nome.endswith(EXTENSOES)):
            continue
        # `OUT_DIR/bindings.rs` is where Cargo puts a generated file at build time -- not a path in the repo
        if nome.split("/")[0] in ("OUT_DIR", "$OUT_DIR", "${OUT_DIR}"):
            continue
        if (ROOT / nome).exists() or (path.parent / nome).exists():
            continue
        # a path that names a file of the crate it sits in, written from the crate root
        crate = path.parent
        if (crate / nome).exists():
            continue
        if nome.rsplit("/", 1)[-1] in BASENAMES:
            continue
        # a doc may name a file to say it is gone -- that is the opposite of drift
        inicio = text.rfind("\n", 0, m.start())
        fim = text.find("\n", m.end())
        linha = text[inicio + 1: fim if fim > 0 else len(text)].lower()
        if any(k in linha for k in ("removed", "deleted", "no longer", "superseded", "gone")):
            continue
        print(f"{rel}: names a path that does not exist: {nome}")
        problems += 1
    for m in LOCAL.finditer(text):
        linha = text[: m.start()].count("\n") + 1
        print(f"{rel}:{linha}: points at a local machine: {m.group(1)}")
        problems += 1

total = sum(1 for _ in readmes())
if problems == 0:
    print(f"OK {total} README files: links resolve, named paths exist, nothing points at a local machine")
else:
    print(f"FAIL {problems} problem(s) across {total} README files")
sys.exit(1 if (problems and "--check" in sys.argv) else 0)
