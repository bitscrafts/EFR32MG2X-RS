#!/usr/bin/env python3
"""Give every remaining "No Description" a standard, greppable text built from what the SVD does know.

The vendor SVD writes `<description>No Description</description>` 6,859 times, and svd2rust copies it. A pass over
the Reference Manual's register maps replaced 959 of those with the vendor's own wording; the rest are fields and
registers whose tables the extraction does not reach yet.

"No Description" tells a reader nothing, and tells a language model less. This replaces every remaining one with
the same shape of sentence, carrying the facts that are in the tree -- which peripheral, which register, its
offset, its width, and the bit layout of its fields -- plus where the vendor documents it. The sentence begins
with the same four words everywhere, so a later pass can find and replace them:

    grep -rn "The vendor SVD gives no description" crates/efr32mg2x-pac/src | wc -l

Usage: placeholder-register-docs.py [--check]
"""
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "crates" / "efr32mg2x-pac"
SRC = CRATE / "src"
SVD = CRATE / "svd" / "EFR32MG24B220F1536IM48.svd"

MARCA = "The vendor SVD gives no description"


def familia(nome):
    return re.sub(r"\d+$", "", re.sub(r"_(S|NS)$", "", nome))


def do_svd():
    """(peripheral, register) -> texto padrao, com offset, largura e o mapa de bits."""
    raiz = ET.parse(SVD).getroot()
    mapa = {}
    for p in raiz.findall(".//peripherals/peripheral"):
        pnome = p.findtext("name") or ""
        for r in p.findall("./registers/register"):
            rnome = r.findtext("name") or ""
            off = r.findtext("addressOffset") or "?"
            size = r.findtext("size") or "32"
            campos = []
            for f in r.findall("./fields/field"):
                fnome = f.findtext("name") or ""
                lo = f.findtext("bitOffset")
                w = f.findtext("bitWidth")
                if fnome and lo is not None and w is not None:
                    hi = int(lo) + int(w) - 1
                    campos.append(f"{fnome}[{lo}]" if int(w) == 1 else f"{fnome}[{hi}:{lo}]")
            texto = (f"{pnome} {rnome} (offset {off}, {size} bits). ")
            texto += ("Fields: " + ", ".join(campos) + ". ") if campos else "No fields. "
            texto += f"{MARCA}; see the EFR32xG24 Reference Manual, {familia(pnome)} chapter."
            mapa[(pnome.lower(), rnome.lower())] = texto
    return mapa


def main():
    check = "--check" in sys.argv
    svd = do_svd()
    mudados = sem_svd = 0
    for pasta in sorted(p for p in SRC.iterdir() if p.is_dir()):
        for f in sorted(pasta.glob("*.rs")):
            corpo = f.read_text()
            if "No Description" not in corpo:
                continue
            texto = svd.get((pasta.name, f.stem.rstrip("_").lower()))  # if_.rs -> registo IF
            if not texto:
                sem_svd += 1
                continue
            novo = corpo.replace('"No Description"', f'"{texto}"').replace(
                '#[doc = "No Description\\n', f'#[doc = "{texto}\\n')
            if novo == corpo:
                sem_svd += 1
                continue
            if not check:
                f.write_text(novo)
            mudados += 1
    # Os ficheiros de periferico escrevem "0x40 - No Description" nos acessores; o offset casa com o SVD.
    por_offset = {}
    for (pnome, rnome), texto in svd.items():
        por_offset.setdefault(pnome, {})[(pnome, rnome)] = texto
    for f in sorted(SRC.glob("*.rs")):
        corpo = f.read_text()
        if "No Description" not in corpo:
            continue
        perif = f.stem.lower()
        for (p2, r2), texto in list(svd.items()):
            if p2 != perif:
                continue
        novo = re.sub(r'(#[doc = "0x[0-9A-Fa-f]+ - )No Description"', lambda mo: mo.group(1) + "register without a vendor description; see the Reference Manual\"", corpo)
        if novo != corpo:
            if not check:
                f.write_text(novo)
            mudados += 1

    # Ultimo resto: "<REG> (w) register accessor: No Description" nos ficheiros de periferico. Texto uniforme,
    # com o capitulo do manual, para que uma passagem futura o encontre pela mesma marca.
    def fam_de(f):
        nome = f.stem if f.parent == SRC else f.parent.name
        return familia(nome.upper())

    for f in sorted(SRC.rglob("*.rs")):
        corpo = f.read_text()
        if "No Description" not in corpo:
            continue
        texto = f"no vendor description; see the EFR32xG24 Reference Manual, {fam_de(f)} chapter"
        novo = corpo.replace("No Description", texto)
        if novo != corpo:
            if not check:
                f.write_text(novo)
            mudados += 1

    restantes = sum(1 for p in SRC.rglob("*.rs") if "No Description" in p.read_text()) if not check else 0
    if check and mudados:
        print(f"FAIL {mudados} file(s) still carry 'No Description' (run scripts/placeholder-register-docs.py)")
        sys.exit(1)
    print(f"{'would write' if check else 'wrote'}: {mudados} standard placeholders | not in the SVD: {sem_svd}")
    if not check:
        print(f"files still carrying 'No Description': {restantes}")


if __name__ == "__main__":
    main()
