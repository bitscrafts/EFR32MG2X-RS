"""Emit the Radio Configurator's BLE 39 MHz table as a Rust module.

Reads the SDK's own generated `sl_rail_ble_config_39MHz.c` (Zlib, Silicon Laboratories) and writes
`crates/efr32mg24-rail-sys/src/ble_config.rs`.

Why generated data instead of compiled C: `rail_types.h` includes chip-specific sources, so a translation unit of
ours that includes it re-defines symbols the RAIL blob already provides (`duplicate symbol: sli_bootloader_preHook`
and a pile of `RAILINT_*`). The numbers, though, are exactly what the firmware needs, and `xtalFrequencyHz = 39 MHz`
is what stops RAIL from asserting `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (code 66) — the assertion that keeps the
Bluetooth stack from starting.

Three details the generator has to get right, each of which cost a wrong first attempt:

- **Comments carry numbers.** `/* 4024 */` and `// 146.28571428571428` are annotations, not values; without stripping
  them the table grew by 28 entries and one frequency became 28571428571428.
- **Pointer casts hide in several forms**: `(uint32_t) &phyInfo_0`, `(uint32_t) irCalConfig_0`, `(uint32_t) NULL`.
  Rust refuses pointer-to-integer casts in constants, so those slots are emitted as 0 and patched at first use.
- **`RAIL_TX_POWER_MAX` is 0x7FFF**, from `rail_types.h`.
"""

import pathlib
import sys
import re

AQUI = pathlib.Path(__file__).resolve().parent.parent
SRC = AQUI / "crates/efr32mg24-rail-sys/vendor/ble_config/sl_rail_ble_1mbps_39MHz.c"
OUT = AQUI / "crates/efr32mg24-rail-sys/src/ble_config.rs"

text = SRC.read_text()
STRIP = re.compile(
    r"#ifdef[^\n]*\n|#ifndef[^\n]*\n|#else[^\n]*\n|#endif[^\n]*\n|/\*.*?\*/|//[^\n]*",
    re.S,
)


def block(name):
    m = re.search(r"^[^\n]*\b" + re.escape(name) + r"\s*(\[[^\]]*\])?\s*=\s*\{", text, re.M)
    if not m:
        raise SystemExit(f"definition not found: {name}")
    i = text.index("{", m.end() - 1)
    depth, j = 0, i
    while j < len(text):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    return STRIP.sub("", text[i + 1 : j])


def parse_array(name):
    """[(kind, value)] where kind is 'n' (number) or 'p' (pointer target, or None for NULL)."""
    body = block(name)
    out = []
    for tok in re.finditer(r"\(uint32_t\)\s*&?\s*(\w+)|(0x[0-9A-Fa-f]+)U?L?|(\d+)U?L?", body):
        if tok.group(1):
            target = tok.group(1)
            out.append(("p", None if target == "NULL" else target))
        elif tok.group(2):
            out.append(("n", int(tok.group(2), 16)))
        else:
            out.append(("n", int(tok.group(3))))
    return out


def emit_array(name, items, ty, mutable=False):
    size = len(items)
    head = f"static mut {name}: [{ty}; {size}] = [" if mutable else f"static {name}: [{ty}; {size}] = ["
    lines = [head]
    row = []
    for kind, value in items:
        row.append("0" if kind == "p" else str(value))
        if len(row) == 8:
            lines.append("    " + ", ".join(row) + ",")
            row = []
    if row:
        lines.append("    " + ", ".join(row) + ",")
    lines.append("];")
    return "\n".join(lines)


def patch_fn(name, items, source_map):
    """A function that writes the pointer slots of `name` at run time."""
    lines = [f"unsafe fn patch_{name.lower()}() {{"]
    writes = 0
    for i, (kind, value) in enumerate(items):
        if kind != "p":
            continue
        if value is None:
            lines.append(f"    {name}[{i}] = 0;")
        else:
            lines.append(f"    {name}[{i}] = core::ptr::addr_of!({source_map[value]}) as u32;")
        writes += 1
    lines.append("}")
    return "\n".join(lines), writes


modem_base = parse_array("sl_rail_ble_phy_1Mbps_viterbi_39MHz_modemConfigBase")
modem_0_37 = parse_array("sl_rail_ble_phy_1Mbps_viterbi_39MHz_0_37_modemConfig")
phy_info = parse_array("phyInfo_0")
ir_cal = parse_array("irCalConfig_0")
timing = parse_array("timingConfig_0")
hfxo = parse_array("hfxoRetimingConfigEntries")
stack_info = parse_array("stackInfo_0")

# C symbol -> Rust static whose address fills that slot.
source_map = {
    "phyInfo_0": "PHY_INFO",
    "irCalConfig_0": "IR_CAL_CONFIG",
    "timingConfig_0": "TIMING_CONFIG",
    "hfxoRetimingConfigEntries": "HFXO_RETIMING",
    "stackInfo_0": "STACK_INFO",
}

# Channel entries, parsed from the source so a typo cannot slip in.
entries_body = block("sl_rail_ble_phy_1Mbps_viterbi_39MHz_channels")
entry_texts = []
depth, start = 0, None
for i, ch in enumerate(entries_body):
    if ch == "{":
        if depth == 0:
            start = i
        depth += 1
    elif ch == "}":
        depth -= 1
        if depth == 0 and start is not None:
            entry_texts.append(entries_body[start + 1 : i])
            start = None

entries = []
for e in entry_texts:
    d = dict((k, " ".join(v.split())) for k, v in re.findall(r"\.(\w+)\s*=\s*([^,\n]+)", e))
    entries.append(d)

cfg_body = block("sl_rail_ble_phy_1Mbps_viterbi_39MHz_channelConfig")
cfg = dict((k, " ".join(v.split())) for k, v in re.findall(r"\.(\w+)\s*=\s*([^,\n]+)", cfg_body))

print(f"arrays: modem={len(modem_base)} 0_37={len(modem_0_37)} phy={len(phy_info)} "
      f"ircal={len(ir_cal)} timing={len(timing)} hfxo={len(hfxo)} stack={len(stack_info)}")
print(f"entries: {len(entries)}")

modem_patch, n_modem = patch_fn("MODEM_CONFIG_0_37", modem_0_37, source_map)
phy_patch, n_phy = patch_fn("PHY_INFO", phy_info, source_map)
print(f"pointer slots: modem_0_37={n_modem} phy_info={n_phy}")


def entry_literal(d, index):
    def num(key, default="0"):
        v = d.get(key, default).rstrip("UL")
        return v

    max_power = d.get("maxPower", "0").strip()
    if "RAIL_TX_POWER_MAX" in max_power:
        max_power_rust = "i16::MAX"
    else:
        max_power_rust = f"{max_power.rstrip('UL')}i16"
    return f"""    RAIL_ChannelConfigEntry_t {{
        phyConfigDeltaAdd: core::ptr::addr_of!(MODEM_CONFIG_0_37) as RAIL_RadioConfig_t,
        baseFrequency: {num('baseFrequency')},
        channelSpacing: {num('channelSpacing')},
        physicalChannelOffset: {num('physicalChannelOffset')}u16,
        channelNumberStart: {num('channelNumberStart')}u16,
        channelNumberEnd: {num('channelNumberEnd')}u16,
        maxPower: {max_power_rust},
        attr: core::ptr::addr_of_mut!(ATTR) as *mut RAIL_ChannelConfigEntryAttr_t,
        entryType: 0,
        reserved: [0; 3],
        stackInfo: core::ptr::addr_of!(STACK_INFO) as *const u8,
        alternatePhy: core::ptr::null_mut(),
    }}"""


literals = ",\n".join(entry_literal(d, i) for i, d in enumerate(entries))

module = f'''//! The Radio Configurator's BLE 1 Mbps Viterbi 39 MHz configuration, as Rust data.
//!
//! **Generated** by `scripts/emit-ble-config.py` from the Gecko SDK's own
//! `sl_rail_ble_config_39MHz.c` (Zlib, Silicon Laboratories). Regenerate rather than edit.
//!
//! Why this file exists: RAIL refuses to configure a radio whose crystal frequency it does not know, asserting
//! `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (code 66), and the default `RAILCb_AssertFailed` implementation is an
//! infinite loop -- so a Bluetooth stack that starts without this table looks like it hangs. The frequency lives in
//! `RAIL_ChannelConfig_t.xtalFrequencyHz`, and it arrives here as data rather than as compiled C because including
//! `rail_types.h` in our own translation unit duplicates symbols the RAIL blob already defines.
//!
//! The tables are `static mut` and filled on first use because ten of their slots hold **addresses as integers**
//! (`(uint32_t) &phyInfo_0`, `(uint32_t) irCalConfig_0`, ...), and Rust will not evaluate a pointer-to-integer cast
//! in a constant.

#![allow(non_upper_case_globals)]

use core::mem::MaybeUninit;
use core::sync::atomic::{{AtomicBool, Ordering}};

use crate::{{
    RAIL_ChannelConfigEntryAttr_t, RAIL_ChannelConfigEntry_t, RAIL_ChannelConfig_t, RAIL_RadioConfig_t,
}};

{emit_array("MODEM_CONFIG_BASE", modem_base, "u32")}

{emit_array("MODEM_CONFIG_0_37", modem_0_37, "u32", mutable=True)}

{emit_array("PHY_INFO", phy_info, "u32", mutable=True)}

{emit_array("IR_CAL_CONFIG", ir_cal, "u8")}

{emit_array("TIMING_CONFIG", timing, "i32")}

{emit_array("HFXO_RETIMING", hfxo, "u8")}

{emit_array("STACK_INFO", stack_info, "u8")}

static mut ATTR: MaybeUninit<RAIL_ChannelConfigEntryAttr_t> = MaybeUninit::uninit();
static mut ENTRIES: MaybeUninit<[RAIL_ChannelConfigEntry_t; {len(entries)}]> = MaybeUninit::uninit();
static mut CONFIG: MaybeUninit<RAIL_ChannelConfig_t> = MaybeUninit::uninit();
static READY: AtomicBool = AtomicBool::new(false);

// The pointer slots, written once. Each list is the set of indices in that array whose value is an address.
{modem_patch.replace("MODEM_CONFIG_0_37[", "MODEM_CONFIG_0_37[") if n_modem else "unsafe fn patch_modem_config_0_37() {}"}

{phy_patch if n_phy else "unsafe fn patch_phy_info() {}"}

/// The channel configuration, filled on first call and valid for the rest of the program.
pub fn config() -> &'static RAIL_ChannelConfig_t {{
    if !READY.swap(true, Ordering::AcqRel) {{
        // SAFETY: guarded by `READY`; the storage is `static`, so the reference handed out below outlives any use.
        unsafe {{
            patch_phy_info();
            patch_modem_config_0_37();

            // The attribute block is all-0xFF in the source; filled as bytes so this file does not depend on the
            // shape of `RAIL_RxIrCalValues_t`.
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!(ATTR) as *mut u8,
                0xFF,
                core::mem::size_of::<RAIL_ChannelConfigEntryAttr_t>(),
            );

            let entries = core::ptr::addr_of_mut!(ENTRIES) as *mut RAIL_ChannelConfigEntry_t;
            let table: [RAIL_ChannelConfigEntry_t; {len(entries)}] = [
{literals},
            ];
            for (i, e) in table.into_iter().enumerate() {{
                core::ptr::write(entries.add(i), e);
            }}

            core::ptr::write(
                core::ptr::addr_of_mut!(CONFIG) as *mut RAIL_ChannelConfig_t,
                RAIL_ChannelConfig_t {{
                    phyConfigBase: core::ptr::addr_of!(MODEM_CONFIG_BASE) as RAIL_RadioConfig_t,
                    phyConfigDeltaSubtract: core::ptr::null(),
                    configs: core::ptr::addr_of!(ENTRIES) as *const RAIL_ChannelConfigEntry_t,
                    length: {cfg.get("length", "1").rstrip("UL").strip()},
                    signature: {cfg.get("signature", "0").rstrip("UL").strip()},
                    xtalFrequencyHz: {cfg.get("xtalFrequencyHz", "39000000").rstrip("UL").strip()},
                }},
            );
        }}
    }}
    // SAFETY: written exactly once above, before any reference is handed out.
    unsafe {{ &*core::ptr::addr_of!(CONFIG).cast::<RAIL_ChannelConfig_t>() }}
}}
'''

# `--check`: regenerate into a temporary file, format it the same way the repository does, and compare. The gate runs
# this, so the committed table cannot drift from its generator without the build saying so -- which is what happened
# when the generator lived outside the repository (`B18`).
if "--check" in sys.argv:
    import subprocess
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        alvo = pathlib.Path(tmp) / "ble_config.rs"
        alvo.write_text(module)
        # rustfmt is what the repository applies after generating; without it the comparison fails on line wrapping
        # alone, which is not drift.
        try:
            subprocess.run(["rustfmt", "--edition", "2021", str(alvo)], check=True, capture_output=True)
        except (OSError, subprocess.CalledProcessError):
            pass  # no rustfmt here: compare the raw output
        esperado = alvo.read_text()
    atual = OUT.read_text()
    if atual == esperado:
        print(f"PASS  {OUT.name}: the generated table matches its generator")
        sys.exit(0)
    print(f"FAIL  {OUT.name} is out of date -- run: python3 scripts/emit-ble-config.py && cargo fmt --all")
    sys.exit(1)

OUT.write_text(module)
print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")
