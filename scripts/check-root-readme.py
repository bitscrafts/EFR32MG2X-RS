#!/usr/bin/env python3
"""Compare the counts on the root README against the tree.

The root README is the first page a stranger reads, and it carried "Crates: 12 total", "12 peripheral modules" and
"Examples: 83 files, 26 of them checked" while the workspace had 31 member crates, the HAL had 29 public modules and
`scripts/check-examples-build.sh` built 114 example targets. A count on a page that nobody checks is the same defect
the HAL README check closed for one crate, so this closes it for the root page.

Counted here, never remembered:

  workspace crates      `members` in Cargo.toml, refusing a member listed twice
  libraries/            libraries/*/Cargo.toml that are workspace members
  HAL public modules    `pub mod` lines in crates/efr32mg2x-hal/src/lib.rs
  example targets       the rule scripts/check-examples-build.sh uses for its verdict line: every `*.rs` directly
                        under a member crate's examples/, minus the packages that script leaves out

Usage: scripts/check-root-readme.py [README]
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The same three packages scripts/check-examples-build.sh skips: they are built by their own scripts.
NOT_IN_EXAMPLE_LIST = {"efr32mg2x-bootloader", "efr32mg2x-bootloader-test-app", "efr32mg2x-cmsis-stubs"}


def package_name(crate_dir: Path) -> str:
    text = (crate_dir / "Cargo.toml").read_text()
    m = re.search(r'^name\s*=\s*"([^"]+)"', text, re.M)
    return m.group(1) if m else ""


def member_entries() -> list[str]:
    text = (ROOT / "Cargo.toml").read_text()
    m = re.search(r"^members\s*=\s*\[(.*?)^\]", text, re.S | re.M)
    if not m:
        raise SystemExit("FAIL Cargo.toml has no members list")
    return re.findall(r'"([^"]+)"', m.group(1))


def main() -> int:
    readme = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "README.md"
    text = readme.read_text()
    problems: list[str] = []

    entries = member_entries()
    duplicates = sorted({e for e in entries if entries.count(e) > 1})
    if duplicates:
        problems.append(f"Cargo.toml lists a member twice: {', '.join(duplicates)}")
    unique = list(dict.fromkeys(entries))

    # A directory under libraries/ counts only if it is a workspace member; a scratch crate left in the tree is not
    # something the README promises.
    member_dirs = {str((ROOT / e).resolve()) for e in unique}
    libraries = [
        d
        for d in sorted((ROOT / "libraries").iterdir())
        if d.is_dir() and (d / "Cargo.toml").exists() and str(d.resolve()) in member_dirs
    ]

    hal_lib = ROOT / "crates/efr32mg2x-hal/src/lib.rs"
    modules = len(re.findall(r"^pub mod", hal_lib.read_text(), re.M))

    examples = 0
    for examples_dir in sorted((ROOT / "crates").glob("*/examples")):
        if not examples_dir.is_dir():
            continue
        if package_name(examples_dir.parent) in NOT_IN_EXAMPLE_LIST:
            continue
        examples += len(list(examples_dir.glob("*.rs")))

    def stated(pattern: str, label: str) -> int | None:
        m = re.search(pattern, text, re.M)
        if not m:
            problems.append(f"README has no `{label}` line, so its number cannot be checked")
            return None
        return int(m.group(1))

    # The `Examples` line may state the number or name the script that prints it. Both are honest; only the first can go
    # stale, and it did four times in one day. When the README delegates, the claim being checked is that the script it
    # names exists -- and the count is still computed here, so a PASS prints the real number either way.
    # Careful: a line that both states a number *and* names the script must still be checked. The first version of this
    # branch skipped any line containing the script's name, and a control that put `999 example targets` next to that
    # name passed -- a false green, found by running the control instead of trusting the change.
    delegates = re.search(r"^\*\*Examples\*\*: .*scripts/check-examples-build\.sh", text, re.M) is not None
    numbers = re.search(r"^\*\*Examples\*\*: (\d+) example targets", text, re.M)
    if numbers is not None:
        stated_examples = int(numbers.group(1))
    elif delegates:
        stated_examples = None
        if not (ROOT / "scripts/check-examples-build.sh").exists():
            problems.append(
                "README points at scripts/check-examples-build.sh for the example count, and there is no such file"
            )
    else:
        stated_examples = stated(r"^\*\*Examples\*\*: (\d+) example targets", "Examples")

    checks = [
        (stated(r"^\*\*Workspace\*\*: (\d+) crates", "Workspace"), len(unique), "workspace crates"),
        (stated(r"^- `libraries/`: (\d+) --", "libraries/"), len(libraries), "libraries"),
        (stated(r"^\*\*HAL modules\*\*: (\d+) public modules", "HAL modules"), modules, "HAL public modules"),
        (stated_examples, examples, "example targets"),
    ]
    for said, real, label in checks:
        if said is not None and said != real:
            problems.append(f"README says {said} {label}; the tree has {real}")

    if problems:
        for problem in problems:
            print(f"FAIL {problem}")
        return 1

    print(
        f"PASS {len(unique)} crates, {len(libraries)} libraries, {modules} HAL modules, "
        f"{examples} example targets"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())