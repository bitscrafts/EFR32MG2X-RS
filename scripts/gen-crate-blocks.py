#!/usr/bin/env python3
"""Render the README blocks that must never be written by hand.

The failure this replaces, measured: the PAC's Usage section announced `peripherals.GPIO_S` (the field is `gpio_s`) and
`pac::CorePeripherals` (absent), pinned `efr32mg2x-pac = "0.1.0"` when the package is `efr32mg2x-pac` at another
version, and listed dependencies the crate does not have. Every one of those is a value the manifest or the source
already knows, so every one of them is generated here and checked by the gate.

    python3 scripts/gen-crate-blocks.py <crate-dir>            # rewrite the marked blocks
    python3 scripts/gen-crate-blocks.py <crate-dir> --check    # fail if they are out of date

Blocks are delimited in the README:

    <!-- gen:usage -->   ... <!-- /gen:usage -->
    <!-- gen:features --> ... <!-- /gen:features -->
    <!-- gen:dependencies --> ... <!-- /gen:dependencies -->
    <!-- gen:peripherals --> ... <!-- /gen:peripherals -->
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEMPLATE = ROOT / "templates" / "crate-usage.tmpl"


def manifest_value(crate_dir, field, default=None):
    text = (crate_dir / "Cargo.toml").read_text()
    m = re.search(rf"^{field}\s*=\s*\"([^\"]*)\"", text, re.M)
    if m:
        return m.group(1)
    if re.search(rf"^{field}\.workspace\s*=\s*true", text, re.M):
        ws = (ROOT / "Cargo.toml").read_text().split("[workspace.package]")[-1]
        m2 = re.search(rf"^{field}\s*=\s*\"([^\"]*)\"", ws, re.M)
        if m2:
            return m2.group(1)
    return default


def features(crate_dir):
    text = (crate_dir / "Cargo.toml").read_text()
    m = re.search(r"^\[features\]\n(.*?)(?=^\[|\Z)", text, re.M | re.S)
    if not m:
        return {}
    out = {}
    for line in m.group(1).splitlines():
        mm = re.match(r"^([A-Za-z0-9_-]+)\s*=\s*\[(.*)\]", line.strip())
        if mm:
            out[mm.group(1)] = [x.strip().strip('"') for x in mm.group(2).split(",") if x.strip()]
    return out


def deps(crate_dir):
    text = (crate_dir / "Cargo.toml").read_text()
    m = re.search(r"^\[dependencies\]\n(.*?)(?=^\[|\Z)", text, re.M | re.S)
    if not m:
        return []
    out = []
    for line in m.group(1).splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        mm = re.match(r"^([A-Za-z0-9_-]+)\s*=\s*(?:\"([^\"]+)\"|\{(.*)\})?", line)
        if not mm:
            continue
        name, plain, braced = mm.group(1), mm.group(2), mm.group(3) or ""
        ver = plain or (re.search(r'version\s*=\s*"([^"]+)"', braced) or [None, "workspace"])[1]
        out.append((name, ver))
    return out


def peripherals(crate_dir):
    lib = (crate_dir / "src" / "lib.rs").read_text()
    fields = re.findall(r"pub ([a-z0-9_]+):\s*[A-Za-z0-9]+", lib)
    if not fields:
        return None
    fams = {}
    for f in fields:
        base = re.sub(r"_(s|ns)$", "", f)
        base = re.sub(r"\d+$", "", base)
        fams.setdefault(base, []).append(f)
    return fams, len(fields)


def block_usage(crate_dir, name, version):
    ex = sorted((crate_dir / "examples").glob("*.rs"), key=lambda p: p.stat().st_size)
    if not ex:
        return None
    body = "\n".join(l for l in ex[0].read_text().splitlines() if not l.startswith("//!")).strip()
    lib = (ROOT / "Cargo.toml").read_text()
    libname = re.search(r"\[lib\]\s*\nname\s*=\s*\"([^\"]+)\"", (crate_dir / "Cargo.toml").read_text())
    extra = ""
    name_note = ""
    if "cortex-m" in [d for d, _ in deps(crate_dir)] or "cortex-m" in (crate_dir / "Cargo.toml").read_text():
        extra = ('# the architecture crate supplies the critical-section implementation that the generated\n'
                 '# modify() needs; without this feature the build fails to link with\n'
                 '# "undefined symbol: _critical_section_1_0_acquire"\n'
                 'cortex-m = { version = "0.7", features = ["critical-section-single-core"] }\n'
                 'cortex-m-rt = "0.7"\n')
    if libname:
        name_note = (f"The package is `{name}`; the **library** it exports is `{libname.group(1)}`, so the `use` line\n"
                     f"below is not a typo.\n\n")
    # The template's own comments are for whoever edits the template, not for the crate page.
    tpl = "\n".join(l for l in TEMPLATE.read_text().splitlines() if not l.startswith("//")).lstrip()
    lib_name = libname.group(1) if libname else name.replace("-", "_")
    return (tpl.replace("{{package}}", name)
               .replace("{{version}}", version)
               .replace("{{extra_deps}}", extra)
               .replace("{{name_note}}", name_note)
               .replace("{{example_path}}", f"examples/{ex[0].name}")
               .replace("{{example_body}}", body)
               .replace("{{lib}}", lib_name))


def block_features(crate_dir):
    feats = features(crate_dir)
    if not feats:
        return None
    lines = ["## Features", "", "| feature | what it turns on | default |", "|---|---|---|"]
    default = feats.get("default", [])
    for f, on in feats.items():
        if f == "default":
            continue
        lines.append(f"| `{f}` | {', '.join('`'+o+'`' for o in on) or '—'} | {'yes' if f in default else 'no'} |")
    return "\n".join(lines) + "\n"


def block_deps(crate_dir):
    ds = deps(crate_dir)
    if not ds:
        return None
    lines = ["## Dependencies", "", "| crate | version |", "|---|---|"]
    for d, v in ds:
        lines.append(f"| `{d}` | `{v}` |")
    return "\n".join(lines) + "\n"


def block_peripherals(crate_dir):
    got = peripherals(crate_dir)
    if not got:
        return None
    fams, total = got
    lines = ["## Peripherals", "",
             f"{total} instances on `Peripherals`, in {len(fams)} families. Generated from `src/lib.rs`; every name here\n"
             "is a field you can call `.take()` on.", "",
             "| family | instances |", "|---|---|"]
    for fam in sorted(fams):
        inst = ", ".join(f"`{i}`" for i in sorted(fams[fam]))
        lines.append(f"| {fam} | {inst} |")
    return "\n".join(lines) + "\n"


BLOCKS = {"usage": block_usage, "features": block_features, "dependencies": block_deps,
          "peripherals": block_peripherals}


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    crate_dir = Path(sys.argv[1]).resolve()
    check = "--check" in sys.argv
    readme_path = crate_dir / "README.md"
    text = readme_path.read_text()
    original = text
    name = manifest_value(crate_dir, "name")
    version = manifest_value(crate_dir, "version", "0.0.0")
    stale = []

    for key, fn in BLOCKS.items():
        begin, end = f"<!-- gen:{key} -->", f"<!-- /gen:{key} -->"
        if begin not in text or end not in text:
            continue
        rendered = fn(crate_dir, name, version) if key == "usage" else fn(crate_dir)
        if rendered is None:
            continue
        new = re.sub(re.escape(begin) + r".*?" + re.escape(end),
                     begin + "\n" + rendered.rstrip() + "\n" + end, text, flags=re.S)
        if new != text:
            stale.append(key)
            text = new

    if check:
        if stale:
            print(f"FAIL {readme_path.name}: generated blocks out of date ({', '.join(stale)}) "
                  f"-- run scripts/gen-crate-blocks.py {sys.argv[1]}")
            return 1
        print(f"OK {readme_path.name}: generated blocks match the tree")
        return 0

    if text != original:
        readme_path.write_text(text)
        print(f"updated {readme_path}: {', '.join(stale)}")
    else:
        print(f"{readme_path}: already current")
    return 0


if __name__ == "__main__":
    sys.exit(main())
