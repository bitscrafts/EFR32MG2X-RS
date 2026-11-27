#!/usr/bin/env python3
"""Does every public module have a README, and does the README still describe the module?

Four findings, each fatal:

  missing     a `pub mod x;` with no `src/x/README.md`
  placeholder a README still carrying a `TODO(measured)` marker
  stale       a public item the source defines and the README never names (the drift nobody notices)
  empty       zero modules inspected -- a check that inspects nothing passes by vacuity, so it fails instead

The generator and the template for these READMEs live in the Hermes skill `hal-module-readme`
(software-development/hal-module-readme/{templates,scripts}); this file is the repository's own copy, because a CI
runner has no `~/.hermes` and the gate must check what the repository contains.

Usage: check-module-readmes.py --hal crates/<crate> [--max-findings N]
Exit:  0 clean, 1 any finding.
"""

import argparse
import re
import sys
from pathlib import Path

MOD = re.compile(r"^pub mod ([a-z_][a-z0-9_]*)\s*;", re.MULTILINE)
ITEM = re.compile(r"^\s*pub (?:fn|struct|enum|trait|const|type|union) ([A-Za-z_][A-Za-z0-9_]*)", re.MULTILINE)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hal", required=True)
    parser.add_argument("--max-findings", type=int, default=8)
    args = parser.parse_args()

    hal = Path(args.hal).resolve()
    lib = hal / "src" / "lib.rs"
    if not lib.is_file():
        print(f"  FAIL  module READMEs: no {lib} to read")
        return 1

    names = MOD.findall(lib.read_text())
    if not names:
        print("  FAIL  module READMEs: inspected 0 modules -- a check that finds nothing cannot pass")
        return 1

    findings: list[str] = []
    inspected = 0
    for name in names:
        readme = hal / "src" / name / "README.md"
        directory = hal / "src" / name
        # A module that is a single `src/x.rs` may still have a `src/x/` directory holding only the README, so the
        # directory counts as the source only when it really holds code.
        sources = sorted(directory.rglob("*.rs")) if directory.is_dir() else []
        if not sources:
            single = hal / "src" / f"{name}.rs"
            sources = [single] if single.is_file() else []
        inspected += 1

        if not readme.is_file():
            findings.append(f"missing: `{name}` has no src/{name}/README.md")
            continue

        text = readme.read_text()
        if "TODO(measured)" in text:
            findings.append(f"placeholder: src/{name}/README.md still carries a TODO(measured) marker")

        for path in sources:
            for item in ITEM.findall(path.read_text()):
                if item.startswith("_"):
                    continue
                # The item must be a ROW of the API table, not a word somewhere in the prose: a name mentioned in a
                # sentence passes a substring test while the table a reader scans has already lost it. Measured with a
                # control that replaced a row and still passed the substring version of this check.
                if not re.search(rf"^\|\s*`{re.escape(item)}`", text, re.MULTILINE):
                    findings.append(
                        f"stale: src/{name}/README.md has no API row for `{item}`, which the source defines"
                    )

    if findings:
        print(f"  FAIL  module READMEs: {len(findings)} finding(s) across {inspected} module(s)")
        for finding in findings[: args.max_findings]:
            print(f"        {finding}")
        if len(findings) > args.max_findings:
            print(f"        ... and {len(findings) - args.max_findings} more")
        return 1

    print(f"  PASS  module READMEs: {inspected} module(s), every one documented and current")
    return 0


if __name__ == "__main__":
    sys.exit(main())
