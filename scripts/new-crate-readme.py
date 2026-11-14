#!/usr/bin/env python3
"""Scaffold a crate README with the parts that must not be hand-written.

Everything this prints is read out of the tree: module names, public-function counts, `unsafe` counts, stub markers,
which examples reach which module, and how many examples carry a hardware-verification marker. The parts a human owes
(what the crate is, the install trap, the known defects, the minimum program) come out as `<<… required>>`, so an
unfinished README reads as unfinished instead of as done.

    python3 scripts/new-crate-readme.py crates/<crate> [--write]
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def cargo_field(manifest, field):
    m = re.search(rf'^{field}\s*=\s*"([^"]*)"', manifest, re.M)
    if m:
        return m.group(1)
    m = re.search(rf'^{field}\.workspace\s*=\s*true', manifest, re.M)
    if m:
        ws = (ROOT / "Cargo.toml").read_text()
        m2 = re.search(rf'^{field}\s*=\s*"([^"]*)"', ws.split("[workspace.package]")[-1], re.M)
        return m2.group(1) if m2 else "?"
    return "?"


def modules(src):
    out = []
    for p in sorted(src.iterdir()):
        if p.name in ("lib.rs", "prelude.rs", "main.rs"):
            continue
        if p.is_dir() or p.suffix == ".rs":
            name = p.name if p.is_dir() else p.stem
            files = sorted(p.rglob("*.rs")) if p.is_dir() else [p]
            body = "\n".join(f.read_text(errors="ignore") for f in files)
            out.append({
                "name": name,
                "bytes": sum(f.stat().st_size for f in files),
                "pubfn": len(re.findall(r"pub (?:const )?fn ", body)),
                "unsafe": len(re.findall(r"\bunsafe\b", body)),
                "stub": len(re.findall(r"\b(todo!|unimplemented!)", body)) - len(re.findall(r"///.*(todo!|unimplemented!)", body)),
            })
    return out


def example_naming(mod):
    hits = []
    for ex in (ROOT / "crates").rglob("examples/**/*.rs"):
        body = ex.read_text(errors="ignore")
        if re.search(rf"\b{mod}::", body):
            hits.append(ex.name)
    return hits


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    crate = Path(sys.argv[1]).resolve()
    manifest = (crate / "Cargo.toml").read_text()
    name = cargo_field(manifest, "name")
    version = cargo_field(manifest, "version")
    src = crate / "src"
    mods = modules(src)

    ex_dir = crate / "examples"
    examples = sorted(ex_dir.glob("*.rs")) if ex_dir.exists() else []
    verified = sum(1 for e in examples if "VERIFIED ON HARDWARE" in e.read_text(errors="ignore"))

    reached = [m for m in mods if example_naming(m["name"])]
    unreached = [m for m in mods if not example_naming(m["name"])]

    lines = []
    w = lines.append
    w(f"# {name} — <<one line: what it is and for which part>> *required*\n")
    w("<<Two or three sentences: what it wraps, what it refuses to do, and what to use instead when this is not it.>>\n")
    w(f"> **Status**: {len(mods)} modules, {sum(m['pubfn'] for m in mods)} public functions, "
      f"{len(examples)} examples ({verified} carrying a hardware date). Groups below.\n")
    w("## Install\n")
    w("```toml")
    w(f"[dependencies]\n{name} = \"{version}\"")
    w("```\n")
    w("<<required: the trap that makes the first build fail, written as the error the user will see, then the fix.>>\n")
    w("## What is in the tree\n")
    for m in mods:
        flags = []
        if m["stub"] > 0:
            flags.append(f"{m['stub']} stub(s)")
        if m["unsafe"] > 0:
            flags.append(f"{m['unsafe']} unsafe")
        w(f"- `{m['name']}` — {m['pubfn']} public functions, {m['bytes']} bytes"
          + (f" ({', '.join(flags)})" if flags else ""))
    w("")
    w("## What has been measured, and what has not\n")
    w("**Exercised by an example (measurement still required — the script cannot know what you watched):**\n")
    for m in reached:
        w(f"- `{m['name']}` — reached by {', '.join(sorted(set(example_naming(m['name']))))[:120]}")
    w("")
    w("**Implemented, no example reaches it:** " + ", ".join(f"`{m['name']}`" for m in unreached) + "\n")
    w("## Known defects\n")
    w("<<required if any: the defect, the measured symptom, and where the detail lives. No defects? say so in one line.>>\n")
    w("## Examples\n")
    w(f"{len(examples)} files in `examples/`. {verified} of them carry a `VERIFIED ON HARDWARE (<date>)` marker in their "
      f"own documentation; the other {len(examples) - verified} have been built, not watched.\n")
    w("Regenerate the full table with `scripts/gen-examples-doc.py`; do not list examples by hand.\n")
    w("## The smallest working program\n")
    w("```rust,ignore\n<<required: copy it from a real example, do not write it from memory.>>\n```\n")
    w("## Licence\n")
    w("MIT OR Apache-2.0, linked to the public repository, never to a path outside the package.\n")
    w("## Before publishing this file\n")
    w("```bash")
    w("python3 scripts/check-publishable.py")
    w(f"cargo publish -p {name} --dry-run")
    w("```\n")

    text = "\n".join(lines)
    if "--write" in sys.argv:
        (crate / "README.md").write_text(text)
        print(f"wrote {crate / 'README.md'}")
    else:
        print(text)


if __name__ == "__main__":
    main()
