#!/usr/bin/env python3
"""Render a crate README from templates/readme-<kind>.tmpl, so no line of it can lie about the tree.

Why the whole file and not just the Usage block: measured, the PAC README had a hand-written peripheral list with 17
names that do not exist and 37 that were missing, a dependency section that invented two crates and a version, a module
structure claiming "2,634 files" when the tree has 2,633, and an example that did not compile. Every one of those was a
value the tree already knew. The prose stays in the template, where a human writes it once; the numbers come from the
manifest, the source, and the measured hardware results.

    python3 scripts/gen-crate-readme.py crates/efr32mg2x-pac            # render
    python3 scripts/gen-crate-readme.py crates/efr32mg2x-pac --check    # fail if out of date
"""
import csv
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HW = Path("/projects/_tmp/efr32mg24-rs/PAC-HARDWARE.tsv")


def field(crate, name, default="?"):
    text = (crate / "Cargo.toml").read_text()
    m = re.search(rf"^{name}\s*=\s*\"([^\"]*)\"", text, re.M)
    if m:
        return m.group(1)
    if re.search(rf"^{name}\.workspace\s*=\s*true", text, re.M):
        ws = (ROOT / "Cargo.toml").read_text().split("[workspace.package]")[-1]
        m2 = re.search(rf"^{name}\s*=\s*\"([^\"]*)\"", ws, re.M)
        if m2:
            return m2.group(1)
    return default


def block_generated(crate):
    files = list((crate / "src").rglob("*.rs"))
    fams, total = peripheral_families(crate)
    svd = sorted(p.name for p in (crate / "svd").glob("*.svd"))[0] if (crate / "svd").exists() else "the vendor SVD"
    return (f"**Generated**: {len(files):,} files across {len({p.parent for p in files})} directories, from "
            f"`{svd}`. {total} peripheral instances in {len(fams)} families.\n")


def block_usage(crate, name, version):
    # An example that reaches a helper with `#[path = "common/x.rs"] mod x;` is correct inside this crate and useless in
    # a README: the reader has no `common/` directory, and that dangling `mod` is exactly the error this block shipped.
    # So the pair is replaced by the helper's own source, recursively, until the block is a program somebody can paste.
    def inline(text, base, depth=0):
        lines = text.splitlines()
        out, i = [], 0
        while i < len(lines):
            m = re.match(r'#\[path = "([^"]+)"\]', lines[i].strip())
            if m and i + 1 < len(lines) and re.match(r"mod [A-Za-z0-9_]+;", lines[i + 1].strip()):
                helper = base / m.group(1)
                if helper.exists() and depth < 4:
                    body = "\n".join(l for l in helper.read_text().splitlines() if not l.startswith("//!"))
                    out.append(f"// ---- inlined from {m.group(1)}, which ships in examples/ ----")
                    out.append(inline(body, base, depth + 1).strip())
                    i += 2
                    continue
            out.append(lines[i])
            i += 1
        return "\n".join(out)

    # Prefer the example written to be self-contained: nothing to inline, nothing the reader has to fetch.
    self_contained = crate / "examples" / "pac_minimal.rs"
    if self_contained.exists():
        ex = self_contained
        body = "\n".join(l for l in ex.read_text().splitlines() if not l.startswith("//!")).strip()
    else:
        ex = sorted((crate / "examples").glob("*.rs"), key=lambda p: p.stat().st_size)[0]
        body = inline("\n".join(l for l in ex.read_text().splitlines() if not l.startswith("//!")), ex.parent).strip()
    lib = re.search(r"\[lib\]\s*\nname\s*=\s*\"([^\"]+)\"", (crate / "Cargo.toml").read_text())
    extra = ""
    if "cortex-m" in (crate / "Cargo.toml").read_text():
        extra = ('# the architecture crate supplies the critical-section implementation that the generated\n'
                 '# modify() needs; without this feature the build fails to link with\n'
                 '# "undefined symbol: _critical_section_1_0_acquire"\n'
                 'cortex-m = { version = "0.7", features = ["critical-section-single-core"] }\n'
                 'cortex-m-rt = "0.7"\n')
    note = ""
    if lib:
        note = f"The package is `{name}`; the **library** it exports is `{lib.group(1)}`, so the `use` line is not a typo.\n\n"
    tpl = "\n".join(l for l in (ROOT / "templates/crate-usage.tmpl").read_text().splitlines()
                    if not l.startswith("//")).lstrip()
    return (tpl.replace("{{package}}", name).replace("{{version}}", version)
               .replace("{{extra_deps}}", extra).replace("{{name_note}}", note)
               .replace("{{lib}}", lib.group(1) if lib else name.replace("-", "_"))
               .replace("{{example_path}}", f"examples/{ex.name}").replace("{{example_body}}", body))


def block_features(crate):
    text = (crate / "Cargo.toml").read_text()
    m = re.search(r"^\[features\]\n(.*?)(?=^\[|\Z)", text, re.M | re.S)
    if not m:
        return ""
    feats, default = {}, []
    for line in m.group(1).splitlines():
        mm = re.match(r"^([A-Za-z0-9_-]+)\s*=\s*\[(.*)\]", line.strip())
        if not mm:
            continue
        items = [x.strip().strip('"') for x in mm.group(2).split(",") if x.strip()]
        if mm.group(1) == "default":
            default = items
        else:
            feats[mm.group(1)] = items
    out = ["## Features", "", "| feature | what it turns on | default |", "|---|---|---|"]
    for f, on in feats.items():
        what = ', '.join('`' + o + '`' for o in on)
        if not what and re.fullmatch(r'mg2[0-9]', f):
            what = 'the device this crate is generated for'
        out.append(f"| `{f}` | {what or '—'} | {'yes' if f in default else 'no'} |")
    return "\n".join(out) + "\n"


def peripheral_families(crate):
    lib = (crate / "src/lib.rs").read_text()
    fields = re.findall(r"pub ([a-z0-9_]+):\s*[A-Za-z0-9]+", lib)
    fams = {}
    for f in fields:
        base = re.sub(r"_(s|ns)$", "", f)
        base = re.sub(r"\d+$", "", base)
        fams.setdefault(base, []).append(f)
    return fams, len(fields)


def block_peripherals(crate):
    fams, total = peripheral_families(crate)
    out = ["## Peripherals", "",
           f"{total} instances on `Peripherals`, in {len(fams)} families. Generated from `src/lib.rs`: every name below is "
           "a field you can reach through `Peripherals`.", "",
           "| family | instances |", "|---|---|"]
    for fam in sorted(fams):
        out.append(f"| {fam} | {', '.join('`'+i+'`' for i in sorted(fams[fam]))} |")
    return "\n".join(out) + "\n"


def block_memory_map(crate):
    mem = (crate / "memory.x").read_text()
    flash = re.search(r"FLASH\s*:\s*ORIGIN\s*=\s*(0x[0-9a-fA-F]+),\s*LENGTH\s*=\s*(\d+)K", mem)
    ram = re.search(r"RAM\s*:\s*ORIGIN\s*=\s*(0x[0-9a-fA-F]+),\s*LENGTH\s*=\s*(\d+)K", mem)
    fo, fl = int(flash.group(1), 16), int(flash.group(2))
    ro, rl = int(ram.group(1), 16), int(ram.group(2))
    return "\n".join([
        "## Memory Map", "",
        f"Taken from `memory.x`, which is what the linker actually uses.", "",
        "| region | start | end | size |", "|---|---|---|---|",
        f"| Flash (application) | `0x{fo:08X}` | `0x{fo + fl*1024 - 1:08X}` | {fl} KB |",
        f"| RAM | `0x{ro:08X}` | `0x{ro + rl*1024 - 1:08X}` | {rl} KB |",
        f"| Gecko bootloader (never written) | `0x{fo - 0x6000:08X}` | `0x{fo - 1:08X}` | 24 KB |", "",
        "Peripheral space: secure `0x4000_0000-0x4FFF_FFFF`, non-secure `0x5000_0000-0x5FFF_FFFF`; every peripheral with "
        "a TrustZone face appears twice, as `<name>_s` and `<name>_ns`.", ""])


def block_module_structure(crate):
    files = list((crate / "src").rglob("*.rs"))
    dirs = {p.parent for p in files}
    fams, total = peripheral_families(crate)
    gpio = sorted(((f, len(list((crate / "src" / f).glob("*.rs")))) for f in
                   [p.name for p in (crate / "src").iterdir() if p.is_dir()]), key=lambda x: -x[1])[:3]
    lib = (crate / "src/lib.rs")
    out = ["## Module Structure", "",
           f"Generated by `svd2rust` and split one register per file. Counted in this tree by the generator that "
           f"produced this section:", "",
           f"- **{len(files):,} files** under `src/`, in **{len(dirs)}** directories",
           f"- **{total}** peripheral instances in **{len(fams)}** families; each with a secure and a non-secure face "
           f"where TrustZone applies (`<name>_s`, `<name>_ns`)",
           f"- the largest register blocks: " + ", ".join(f"`{n}/` {c} modules" for n, c in gpio),
           f"- `lib.rs` is {len(lib.read_text().splitlines()):,} lines ({lib.stat().st_size:,} B): the `Peripherals` "
           f"struct and the base addresses", "",
           "**There is no `mod.rs` inside a peripheral directory.** `svd2rust` writes the register block as "
           "`<periph>.rs` and puts the register modules in the directory beside it; that file declares them. An earlier "
           "version of this page showed a `mod.rs` that does not exist.", ""]
    return "\n".join(out)


def workspace_versions():
    ws = (ROOT / "Cargo.toml").read_text().split("[workspace.dependencies]")[-1]
    out = {}
    for line in ws.splitlines():
        m = re.match(r"^([A-Za-z0-9_-]+)\s*=\s*(?:\"([^\"]+)\"|\{([^}]*)})", line.strip())
        if m:
            out[m.group(1)] = m.group(2) or (re.search(r'version\s*=\s*"([^"]+)"', m.group(3) or "") or [None, "?"])[1]
    return out


def block_linker_script(crate):
    mem = (crate / "memory.x").read_text()
    body = "\n".join(l for l in mem.splitlines() if l.strip() and not l.startswith("/*") and not l.startswith(" *"))
    return "\n".join([
        "## Linker Script", "",
        "Shipped with the crate as `memory.x`; `build.rs` copies it to `OUT_DIR` so `-Tlink.x` can include it. These are "
        "the numbers in the file, not a plan:", "",
        "```ld", body.strip(), "```", "",
        "The first 24 KiB (`0x08000000`-`0x08005FFF`) belong to the Gecko bootloader; an image there bricks the board. "
        "Build with `-C link-arg=--nmagic` as well, or the linker emits a segment at `0x08000000` regardless of this "
        "script.", ""])


def block_dependencies(crate):
    """Every dependency in the manifest, including the ones written as [dependencies.x] tables.

    The table used to read only the inline [dependencies] block, so an optional dependency declared as its own table --
    critical-section, which is on by default -- was missing from a section that claims to list dependencies.
    """
    text = (crate / "Cargo.toml").read_text()
    ws = workspace_versions()
    rows = []
    m = re.search(r"^\[dependencies\]\n(.*?)(?=^\[|\Z)", text, re.M | re.S)
    if m:
        for line in m.group(1).splitlines():
            line = line.strip()
            mm = re.match(r"^([A-Za-z0-9_-]+)\s*=\s*(?:\"([^\"]+)\"|\{(.*)\})?", line)
            if not mm:
                continue
            inline = mm.group(3) or ""
            ver = mm.group(2) or (re.search(r'version\s*=\s*"([^"]+)"', inline) or [None, None])[1] \
                or ws.get(mm.group(1), "workspace")
            rows.append((mm.group(1), ver, bool(re.search(r'optional\s*=\s*true', inline))))
    for mm in re.finditer(r"^\[dependencies\.([A-Za-z0-9_-]+)\]\n(.*?)(?=^\[|\Z)", text, re.M | re.S):
        body = mm.group(2)
        ver = (re.search(r'version\s*=\s*"([^"]+)"', body) or [None, None])[1] or ws.get(mm.group(1), "workspace")
        rows.append((mm.group(1), ver, bool(re.search(r'optional\s*=\s*true', body))))
    out = ["## Dependencies", "", "| crate | version |", "|---|---|"]
    for name, ver, opt in sorted(rows):
        out.append(f"| `{name}` | `{ver}`{' (optional)' if opt else ''} |")
    return "\n".join(out) + "\n"


def block_related(crate):
    ws = (ROOT / "Cargo.toml").read_text()
    members = re.findall(r"^name\s*=\s*\"(efr32mg2x-[a-z0-9-]+)\"", "\n".join(
        (ROOT / "crates" / d / "Cargo.toml").read_text() for d in
        [p.name for p in (ROOT / "crates").iterdir() if (p / "Cargo.toml").exists()]), re.M)
    self_name = re.search(r'^name\s*=\s*"([^"]+)"', (crate / "Cargo.toml").read_text(), re.M).group(1)
    out = ["## Related Crates", "", "| crate | what it adds |", "|---|---|"]
    for m in sorted(set(members)):
        if m == self_name:
            continue
        man = next((p / "Cargo.toml" for p in (ROOT / "crates").iterdir() if (p / "Cargo.toml").exists()
                    and re.search(r'^name\s*=\s*"' + re.escape(m) + r'"', (p / "Cargo.toml").read_text(), re.M)), None)
        desc = ""
        if man:
            d = re.search(r'^description\s*=\s*"([^"]+)"', man.read_text(), re.M)
            desc = d.group(1) if d else ""
        out.append(f"| `{m}` | {desc or 'in this workspace'} |")
    out += ["", "Runnable examples for all of them: <https://github.com/bitscrafts/efr32mg2x-examples>.", ""]
    return "\n".join(out)


BLOCKS = {"generated": block_generated, "usage": None, "features": block_features, "peripherals": block_peripherals,
          "memory_map": block_memory_map, "module_structure": block_module_structure,
          "dependencies": block_dependencies,
          "linker_script": block_linker_script, "related_crates": block_related}


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    crate = Path(sys.argv[1]).resolve()
    check = "--check" in sys.argv
    name, version = field(crate, "name"), field(crate, "version")
    kind = "pac" if "pac" in name else "hal"
    tpl = (ROOT / f"templates/readme-{kind}.tmpl").read_text()

    for key, fn in BLOCKS.items():
        token = "{{" + key + "}}"
        if token not in tpl:
            continue
        rendered = block_usage(crate, name, version) if key == "usage" else fn(crate)
        tpl = tpl.replace(token, rendered.rstrip() + "\n")

    tpl = tpl.replace("{{package}}", name).replace("{{crate_dir}}", crate.name)

    readme = crate / "README.md"
    if check:
        if readme.read_text() != tpl:
            print(f"FAIL {readme.relative_to(ROOT)} is out of date -- run scripts/gen-crate-readme.py {sys.argv[1]}")
            return 1
        print(f"OK {readme.relative_to(ROOT)} matches the template and the tree")
        return 0

    readme.write_text(tpl)
    print(f"wrote {readme.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
