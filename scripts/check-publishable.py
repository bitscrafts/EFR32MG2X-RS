#!/usr/bin/env python3
# Check what would actually be published, before publishing it.
#
# The failure this exists for: ef32mg2x-hal 0.1.0 went to crates.io with a README announcing "Phase 5 Tier 2 - USART
# Complete, I2C/SPI/Timers Next" while i2c, spi and timer were all in the tree, and pointing at a docs/PHASE2_PLAN.md
# that does not exist. `cargo publish --dry-run` compiles the package; it never reads it.
#
# So this reads the package the way a stranger does:
#   * every relative link in the README must resolve inside the package
#   * every file included with include_str! must be in the package
#   * no document may assert a state the tree contradicts
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

CRATES = {
    "efr32mg2x-pac": {
        "dir": "crates/efr32mg24-pac",
        # Phrases that shipped a false status to crates.io. Any of them inside the package is a failure.
        "forbidden": [
            r"Phase \d+ Tier \d+",
            r"I2C/SPI/Timers Next",
            r"Planned next",
            r"blink timing issue",
            r"Partial Success",
            r"NEXT TEST",
        ],
    },
    "efr32mg2x-hal": {
        "dir": "crates/efr32mg24-hal",
        "forbidden": [
            r"Phase \d+ Tier \d+",
            r"Planned next",
            r"Planned\b",
            r"In Progress",
            r"Phase 2 Complete",
        ],
    },
}

LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
INCLUDE = re.compile(r'include_str!\("([^"]+)"\)')


def packaged_files(crate_dir: Path):
    out = subprocess.run(
        ["cargo", "package", "--list", "--allow-dirty", "--manifest-path", str(crate_dir / "Cargo.toml")],
        capture_output=True,
        text=True,
    )
    if out.returncode != 0:
        raise SystemExit(f"cargo package --list failed: {out.stderr[-400:]}")
    return {line.strip() for line in out.stdout.splitlines() if line.strip()}


def main():
    failures = []
    for name, spec in CRATES.items():
        d = ROOT / spec["dir"]
        files = packaged_files(d)
        readme = d / "README.md"
        text = readme.read_text() if readme.exists() else ""

        for link in LINK.findall(text):
            if link.startswith(("http://", "https://", "#", "mailto:")):
                continue
            target = (d / link).resolve()
            try:
                rel = target.relative_to(d.resolve())
            except ValueError:
                failures.append(f"{name}: README links outside the crate: {link}")
                continue
            if str(rel) not in files and not any(f.startswith(str(rel) + "/") for f in files):
                failures.append(f"{name}: README links to {link}, which is not in the package")

        for src in d.rglob("*.rs"):
            for inc in INCLUDE.findall(src.read_text(errors="ignore")):
                target = (src.parent / inc).resolve()
                try:
                    rel = target.relative_to(d.resolve())
                except ValueError:
                    failures.append(f"{name}: {src.name} includes a file outside the crate: {inc}")
                    continue
                if str(rel) not in files:
                    failures.append(f"{name}: {src.name} includes {inc}, which is not in the package")

        for doc in list(d.rglob("*.md")):
            body = doc.read_text(errors="ignore")
            for pat in spec["forbidden"]:
                for m in re.finditer(pat, body):
                    line = body[: m.start()].count("\n") + 1
                    failures.append(f"{name}: {doc.relative_to(d)}:{line} says '{m.group(0)}'")

        print(f"  {name}: {len(files)} files packaged, README links and includes checked")

    if failures:
        print("\nFAIL what would be published does not match the tree:")
        for f in failures:
            print(f"  {f}")
        return 1
    print("\nPASS the package says what the tree does")
    return 0


if __name__ == "__main__":
    sys.exit(main())
