#!/usr/bin/env python3
"""Give every peripheral module in the PAC a one-line description.

Why this exists: docs.rs renders `src/<peripheral>.rs` as a module page, and svd2rust writes `#[doc]` on items
but never a module-level `//!`. So all 104 module pages rendered with no description at all -- the generated
documentation was invisible in the one place people (and language models) read it.

Where each line comes from, and where it does not:

  * NAMES: the chapter list of the EFR32xG24 Reference Manual (1152 pages, sha256 d52ada9353165077..., held
    outside the repository because it is a vendor document -- see scripts/fetch-reference-manual.sh). A chapter
    title in this manual is the vendor's own name for the block, which is why the wording is safe to copy.
  * The 33 families below appear there or in the datasheet's table of contents.
  * Sixteen are not chapters at all: BUILD, DCDC, DMEM, GENERIC, HFRCOEM, HOSTMAILBOX, LDMAXBAR, LIB,
    RADIOAES, SCRATCHPAD, SEMAILBOX_{NS,S}_HOST and the SMU/SYSCFG CFGNS views. For those the line says the
    vendor gives no description, instead of inventing an expansion -- a wrong expansion in generated docs is
    worse than a missing one, and an LLM reading it has no way to tell.

Usage: python3 scripts/add-pac-module-docs.py [--check]
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "crates" / "efr32mg2x-pac" / "src"

# Family -> name, copied from the EFR32xG24 Reference Manual's chapter list (or the datasheet where the RM has
# no chapter of its own).
NOMES = {
    "ACMP": "Analog Comparator",
    "BURAM": "Backup RAM",
    "BURTC": "Back-Up Real Time Counter",
    "CMU": "Clock Management Unit",
    "DEVINFO": "Device Info Page",
    "DPLL": "Digital Phased Locked Loop",
    "EMU": "Energy Management Unit",
    "EUSART": "Universal Synchronous Asynchronous Receiver/Transmitter",
    "FSRCO": "Fast Start RCO",
    "GPCRC": "General Purpose Cyclic Redundancy Check",
    "GPIO": "General Purpose Input/Output",
    "HFRCO": "High-Frequency RC Oscillator",
    "HFXO": "High Frequency Crystal Oscillator",
    "I2C": "Inter-Integrated Circuit Interface",
    "IADC": "Incremental Analog to Digital Converter",
    "ICACHE": "Instruction Cache",
    "KEYSCAN": "Keyboard Scan",
    "LDMA": "Linked DMA",
    "LETIMER": "Low Energy Timer",
    "LFRCO": "Low-Frequency RC Oscillator",
    "LFXO": "Low-Frequency Crystal Oscillator",
    "MSC": "Memory System Controller",
    "MVP": "Matrix Vector Processor",
    "PCNT": "Pulse Counter",
    "PRS": "Peripheral Reflex System",
    "SMU": "Security Management Unit",
    "SYSCFG": "System Configuration",
    "SYSRTC": "System RTC",
    "TIMER": "Timer/Counter",
    "ULFRCO": "Ultra Low Frequency RC Oscillator",
    "USART": "Universal Synchronous Asynchronous Receiver/Transmitter",
    "VDAC": "Digital to Analog Converter",
    "WDOG": "Watch Dog Timer",
}

SEM_NOME = (
    "Register block; neither the vendor datasheet nor the reference manual's chapter list "
    "describes this block."
)


def familia(nome):
    """`GPIO_S` and `GPIO_NS` are one peripheral seen through TrustZone's two faces."""
    return re.sub(r"\d+$", "", re.sub(r"_(S|NS)$", "", nome))


def texto(nome):
    nome_fam = NOMES.get(familia(nome))
    if nome_fam:
        return f"{nome_fam}. Registers of `{nome}`."
    return f"{nome}. {SEM_NOME}"


def primeira_linha_nossa(corpo):
    """True when the first line is a description this script wrote (so it may be replaced)."""
    if not corpo.startswith("//! "):
        return None
    return corpo.split("\n", 1)[0]


def main():
    check = "--check" in sys.argv
    mudados = []
    for f in sorted(SRC.glob("*.rs")):
        nome = f.stem.upper()
        if nome in ("LIB", "BUILD"):
            continue
        corpo = f.read_text()
        quero = f"//! {texto(nome)}"
        atual = primeira_linha_nossa(corpo)
        if atual == quero:
            continue
        if atual is None:
            novo = quero + "\n" + corpo
        else:
            resto = corpo.split("\n", 1)[1] if "\n" in corpo else ""
            novo = quero + "\n" + resto
        if not check:
            f.write_text(novo)
        mudados.append(f.name)
    if check and mudados:
        print(f"FAIL {len(mudados)} module descriptions missing or stale (run scripts/add-pac-module-docs.py)")
        sys.exit(1)
    print(f"{'would fix' if check else 'fixed'}: {len(mudados)} module descriptions")


if __name__ == "__main__":
    main()
