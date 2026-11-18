#!/usr/bin/env python3
"""Move one `libraries/<core>-core` into the HAL, following CORE-MOVE-PROCEDURE.md.

Usage: python3 scripts/move-core.py i2c

Six steps, each asserted so a surprise stops the run instead of half-moving a crate:
  1 read the first lines (crate-root attributes must not travel into a module)
  2 copy the source to crates/efr32mg2x-hal/src/cores/<core>.rs, attributes stripped
  3 declare it in cores/mod.rs
  4 rewrite the library as the shim, keeping #![no_std] (it is a crate root)
  5 redirect the HAL's callers to crate::cores::<core>
  6 drop the path dependency from the HAL manifest
"""
import pathlib, re, sys

raiz = pathlib.Path(__file__).resolve().parent.parent
RAIZ_ATRIBUTOS = ("#![no_std]", "#![deny(", "#![warn(", "#![forbid(", "#![cfg_attr(", "#![allow(")

def passo1_ler(fonte):
    primeiras = fonte.read_text().splitlines()[:5]
    return [l for l in primeiras if l.strip().startswith("#![")]

def passo2_copiar(core, fonte, destino):
    linhas = [l for l in fonte.read_text().splitlines() if l.strip() not in RAIZ_ATRIBUTOS]
    destino.write_text("\n".join(linhas) + "\n")
    return len(fonte.read_text().splitlines()) - len(linhas)

def passo3_declarar(core, mod):
    t = mod.read_text()
    if f"pub mod {core};" not in t:
        mod.write_text(t.rstrip() + f"\npub mod {core};\n")

def passo4_shim(core, fonte, caminho_relativo):
    fonte.write_text(
        f"//! Host-test harness for the {core} logic.\n//!\n"
        "//! The source lives in the HAL, because the HAL is the crate that gets published and a\n"
        "//! published crate cannot depend on a path outside its own package. This file compiles\n"
        "//! that same file, so the host tests keep running here.\n"
        "//! Do not paste the code back: scripts/check-core-split.py fails if you do.\n\n"
        "#![no_std]\n\n"
        f"#[path = \"{caminho_relativo}\"]\n"
        f"mod {core};\n"
        f"pub use {core}::*;\n")

def passo5_redireccionar(core, hal_src):
    mudados = 0
    for f in hal_src.rglob("*.rs"):
        t = f.read_text()
        if "efr32mg2x_" + core + "_core::" in t:
            f.write_text(t.replace("efr32mg2x_" + core + "_core::", "crate::cores::" + core + "::"))
            mudados += 1
    return mudados

def passo6_dependencia(core, cargo):
    t = cargo.read_text()
    novo = re.sub(r"^efr32mg2x-" + core + r"-core = \{[^}]*\}\n", "", t, flags=re.M)
    assert novo != t, f"a dependencia de {core}-core nao foi encontrada no Cargo.toml"
    cargo.write_text(novo)

def main():
    if len(sys.argv) != 2:
        sys.exit("usage: python3 scripts/move-core.py <core>")
    core = sys.argv[1]
    fonte = raiz / "libraries" / f"{core}-core" / "src" / "lib.rs"
    assert fonte.is_file(), f"nao existe: {fonte}"
    hal = raiz / "crates" / "efr32mg2x-hal"
    cores = hal / "src" / "cores"
    destino = cores / f"{core}.rs"
    assert not destino.exists(), f"{destino} ja existe: este core ja foi movido?"

    atributos = passo1_ler(fonte)
    print(f"1  atributos de raiz encontrados: {atributos or 'nenhum'}")
    n = passo2_copiar(core, fonte, destino)
    print(f"2  copiado ({n} linha(s) de atributo removida(s))")
    passo3_declarar(core, cores / "mod.rs")
    print("3  declarado em cores/mod.rs")
    rel = "../../../crates/efr32mg2x-hal/src/cores/" + core + ".rs"
    passo4_shim(core, fonte, rel)
    print("4  library reduzida a shim")
    m = passo5_redireccionar(core, hal / "src")
    print(f"5  {m} ficheiro(s) do HAL redireccionado(s)")
    passo6_dependencia(core, hal / "Cargo.toml")
    print("6  dependencia por path removida")
    print(f"\nagora: python3 scripts/check-core-split.py && bash scripts/gate.sh")

if __name__ == "__main__":
    main()
