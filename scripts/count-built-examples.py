#!/usr/bin/env python3
"""Count the examples cargo actually built, from its own JSON stream.

Why this exists: the gate used to print `$pkg: $count examples` where `$count` came from `ls examples/*.rs`. An example
with `required-features` that the invocation does not enable is *skipped*, not failed, so a build that compiles two of
seven prints a PASS that names seven. Measured 2026-11-27 on `efr32mg2x-radio`: 7 example files, 2 built, gate said
"7 examples".

Input: the file cargo wrote with `--message-format=json-render-diagnostics` (JSON on stdout, human diagnostics on
stderr). Output: the number of distinct example targets that produced a compiler-artifact.

A counter, not a verdict: the gate decides what a shortfall means.
"""
import json
import sys


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: count-built-examples.py <cargo-json>", file=sys.stderr)
        return 2
    names = set()
    with open(sys.argv[1], encoding="utf-8", errors="replace") as stream:
        for line in stream:
            line = line.strip()
            if not line.startswith("{"):
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("reason") != "compiler-artifact":
                continue
            target = event.get("target") or {}
            if "example" in (target.get("kind") or []) and target.get("name"):
                names.add(target["name"])
    print(len(names))
    return 0


if __name__ == "__main__":
    sys.exit(main())
