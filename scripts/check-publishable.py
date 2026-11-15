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
        "dir": "crates/efr32mg2x-pac",
        # Phrases that shipped a false status to crates.io. Any of them inside the package is a failure.
        "required": [r"0x08006000", r"--nmagic"],
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



IDENTITY_DEP = re.compile(r'^([A-Za-z0-9_-]+)\s*=\s*(?:"([^"]+)"|\{([^}]*)\})', re.M)
OWN_DEP = re.compile(r'^([A-Za-z0-9_-]+)\s*=\s*"([^"]+)"', re.M)


def check_identity(name, d, readme_text, failures):
    """The page must describe the crate it is on: same name, same version, and only features that exist."""
    manifest = (d / 'Cargo.toml').read_text()
    pkg = re.search(r'^name\s*=\s*"([^"]+)"', manifest, re.M).group(1)
    mver = re.search(r'^version\s*=\s*"([^"]+)"', manifest, re.M)
    if mver:
        version = mver.group(1)
    else:
        ws = (ROOT / 'Cargo.toml').read_text()
        version = re.search(r'^version\s*=\s*"([^"]+)"', ws, re.M).group(1)
    feats = set(re.findall(r'^([A-Za-z0-9_-]+)\s*=\s*\[', manifest, re.M))

    title = next((l for l in readme_text.splitlines() if l.startswith('# ')), '')
    if pkg not in title:
        failures.append(f'{name}: README title is "{title}" but the package is {pkg}')

    # Any dependency-looking line that names an efr32 crate must name *this* crate, at *this* version.
    for m in IDENTITY_DEP.finditer(readme_text):
        dep, quoted, braced = m.group(1), m.group(2), m.group(3)
        if 'efr32' not in dep:
            continue
        if dep != pkg:
            failures.append(f'{name}: README installs "{dep}", which is not this package ({pkg})')
            continue
        if quoted and quoted != version:
            failures.append(f'{name}: README pins {dep} = "{quoted}" but the version is {version}')
        if braced:
            v = re.search(r'version\s*=\s*"([^"]+)"', braced)
            if v and v.group(1) != version:
                failures.append(f'{name}: README pins {dep} version "{v.group(1)}" but the version is {version}')
            fm = re.search(r'features\s*=\s*\[([^\]]*)\]', braced)
            if fm:
                for f in re.findall(r'"([^"]+)"', fm.group(1)):
                    if f not in feats:
                        failures.append(f'{name}: README offers feature "{f}" which the manifest does not define')

    # Features offered in a table or list must exist too.
    for f in re.findall(r'^-\s`([A-Za-z0-9_-]+)`\s-', readme_text, re.M):
        if f not in feats and not f.startswith('0x'):
            failures.append(f'{name}: README lists feature "{f}" which the manifest does not define')



OWN_REPO = re.compile(r"https://github\.com/bitscrafts/EFR32MG2X-RS/(?:blob|tree)/master/([^)\s\"'`]+)")


def check_own_urls(name, d, failures):
    for doc in [d / "README.md"] + sorted(d.rglob("*.md")):
        if not doc.exists():
            continue
        for m in OWN_REPO.finditer(doc.read_text(errors="ignore")):
            target = m.group(1).rstrip(".,;:")
            if not (ROOT / target).exists():
                failures.append(f"{name}: {doc.name} links to {target}, which does not exist in this repository")



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

        readme_text = (d / "README.md").read_text() if (d / "README.md").exists() else ""
        for need in spec.get("required", []):
            if not re.search(need, readme_text):
                failures.append(f"{name}: README never mentions {need}, and this crate ships a linker script")

        check_identity(name, d, readme_text, failures)
        check_own_urls(name, d, failures)

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
