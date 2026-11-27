#!/usr/bin/env python3
"""Group a crate's examples by the features each one declares.

Why: one union of features per crate is the wrong instrument. Measured 2026-11-27 on `efr32mg2x-radio`, seven examples,
four distinct feature vectors -- with the union enabled, the two examples that need nothing stop *linking*
(`rust-lld` fails), so a union build can neither prove the five nor leave the two alone. Building each group with
exactly its own `required-features` proves all seven and keeps the number of cargo invocations at the number of
distinct vectors, which is one or two for most crates.

Input: a crate directory. Output, one line per group, tab separated:

    <features or -> <TAB> <example> <TAB> <example> ...

An example with no `[[example]]` block, or one without `required-features`, belongs to the group with no features.
"""
import pathlib
import re
import sys


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: example-groups.py <crate-dir>", file=sys.stderr)
        return 2
    crate = pathlib.Path(sys.argv[1])
    manifest = (crate / "Cargo.toml").read_text()
    examples = sorted(p.stem for p in (crate / "examples").glob("*.rs")) if (crate / "examples").is_dir() else []
    if not examples:
        return 0

    declared: dict[str, list[str]] = {}
    for block in re.findall(r"\[\[example\]\](.*?)(?=\n\[|\Z)", manifest, re.S):
        name = re.search(r'name\s*=\s*"([^"]+)"', block)
        features = re.search(r"required-features\s*=\s*\[([^\]]*)\]", block)
        if name:
            values = [f.strip().strip('"') for f in features.group(1).split(",")] if features else []
            declared[name.group(1)] = sorted(f for f in values if f)

    groups: dict[tuple[str, ...], list[str]] = {}
    for example in examples:
        groups.setdefault(tuple(declared.get(example, [])), []).append(example)

    for features, names in sorted(groups.items(), key=lambda item: (len(item[0]), item[0])):
        print("\t".join([",".join(features) or "-", *names]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
