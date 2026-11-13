# Reading the vendor blobs

How to find out what a closed binary actually does with this silicon, and why it is worth the trouble. The
short version: **the blob is the only code that has run on the part.** Headers, PACs and reference manuals
describe what the *vendor intended*; the blob describes what was *compiled and shipped*, and when the two
disagree the blob is right until the silicon says otherwise.

Written 2026-11-13 after this method found two register offsets that four other sources had wrong.

## Level 1 — `objdump`, which needs nothing installed

Fast, and enough for most questions.

```bash
# the archive that corresponds to our part
B=crates/efr32mg24-rail-sys/blobs/efr32mg24b220/librail_efr32xg24_gcc_release.a

arm-none-eabi-nm -A "$B" | grep -i sysrtc     # which objects mention it, and are they U or T?
arm-none-eabi-objdump -d "$B" > /tmp/rail.disasm
grep -nE "0x500a8|#576|#4100" /tmp/rail.disasm  # the peripheral base, and register offsets
```

Two habits make this productive:

- **Search for the peripheral base address in literal pools.** ARM code loads a peripheral base with
  `ldr rN, [pc, #k]` from a constant pool, and the pool line is printed as `.word 0x500a8000`. Finding those
  lines finds the functions that talk to the peripheral, without knowing any symbol name.
- **Read 30 lines before a hit, not the hit alone.** A single `str r2, [r3, #96]` means nothing; the
  surrounding `ldr`s show which registers are read together, which is how the layout was established.

What this produced for SYSRTC (and what nothing else had told us):

```
Disassembly of section .text.SYSRTC_SEQ_IRQHandler:
   0: ldr r3, [pc, #12]      @ 0x500a8000
   2: ldr r2, [r3, #96]      @ +0x60  GRP0_IF
   4: ldr r1, [r3, #100]     @ +0x64  GRP0_IEN
   6: add.w r3, r3, #8192    @ +0x2000 = the CLR alias page
   a: ands r2, r1
   c: str r2, [r3, #96]      @ GRP0_IF_CLR
   e: bx lr
```

and, in `RTCCSYNC_GetRtccCount`, `ldr r0, [r3, #24]` — a plain read of `CNT` at **+0x18**, which is the offset
we use and confirms it.

## Level 2 — REA + Ghidra, for pseudocode and cross-references

REA ("reverse engineer anything") is an MCP server and CLI that wraps Ghidra, Rizin and others. The CLI is
what is used here; the MCP form is for an agent to call interactively.

### Installing it on this container (the exact, complete recipe)

Three components, and the versions are not negotiable — REA rejects the rest:

| Component | Version | Why that one |
|---|---|---|
| Node | 22.23.4+ / 24.22.1+ / **26.12.1+** | 26.7.0 (what the image ships) is *rejected*: `rea-agents` requires `^22.23.4 \|\| ^24.22.1 \|\| >=26.12.1` |
| JDK | **25** | Ghidra 12.2 refuses anything older: `Select a 64-bit full JDK 25 or newer` |
| Ghidra | **12.2 PUBLIC** | REA's bridge is verified against 12.1.4 and accepts 12.x >= 12.1 |

```bash
# 1. JDK 25 (Debian 13's apt has it; 21 is not enough for Ghidra 12.2)
apt-get install -y openjdk-25-jdk-headless

# 2. Ghidra 12.2, extracted with the executables still executable
mkdir -p /projects/_tmp/tools && cd /projects/_tmp/tools
curl -sL -o ghidra.zip \
  https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_12.2_build/ghidra_12.2_PUBLIC_20261016.zip
python3 -m zipfile -e ghidra.zip .          # NOT a Python zipfile.extractall: see the trap below
chmod +x ghidra_12.2_PUBLIC/support/analyzeHeadless \
         ghidra_12.2_PUBLIC/Ghidra/Features/Decompiler/os/linux_x86_64/*

# 3. REA itself
npm i -g rea-agents@latest

# 4. Environment (every one of these is required, and they must be set in the shell that runs REA)
export JAVA_HOME=/usr/lib/jvm/java-25-openjdk-amd64
export PATH=/root/.local/bin:$JAVA_HOME/bin:$PATH
export GHIDRA_INSTALL_DIR=/projects/_tmp/tools/ghidra_12.2_PUBLIC

# 5. Prove it before using it
node /root/.local/lib/node_modules/rea-agents/scripts/rea.mjs capabilities \
  | grep -A3 "id: ghidra" | grep status        # must print: status: available
```

`scripts/setup-rea.sh` in this repository does exactly the above and is what should be called; this section
exists so that the script's choices are explained rather than magic.

### The traps, each of which cost a run

- **`zipfile.extractall` drops the executable bit.** Ghidra's `analyzeHeadless` and the native `decompile`
  binary then exist but are not executable, and REA reports `executable_missing` — pointing at a file that is
  right there. `python3 -m zipfile -e` preserves the mode; a bare `chmod +x` on the two paths is the fix.
- **JDK 21 is not enough.** The failure is `unsupported_version` with the reason naming JDK 25, after the
  Ghidra path was already accepted. Install 25 *and* export `JAVA_HOME`; a `java` on PATH from another
  version wins otherwise.
- **A background shell does not inherit the interactive environment.** The first run of the decompile script
  in the background exited 127 (`npx: command not found`). Scripts must set `PATH`/`JAVA_HOME`/
  `GHIDRA_INSTALL_DIR` themselves and use absolute paths for `node`.
- **REA is an MCP server first.** `rea analyze` / `rea decompile` work as a CLI, but the CLI is a thin front
  end over the same providers, so the capability check above is the thing to run before trusting any output.

### The commands that matter

```bash
REA=/root/.local/lib/node_modules/rea-agents/scripts/rea.mjs
ELF=/projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/rail_ble_tx

# what is this binary (arch, format, digest, providers)
node "$REA" analyze "$ELF" --format md

# pseudocode for one function, by symbol name
node "$REA" decompile "$ELF" --procedure SYSRTC_SEQ_IRQHandler --format md
```

Ghidra analyses the whole image on first use (a few minutes for a 70 KB ARM ELF); afterwards the session
serves queries. The output is a markdown envelope: `evidence_id`, `subject.digest`, `provider`, then the
pseudocode under `normalized_result`, with `authority: shipped-artifact` and `confidence: observed`.

What it returned for the same function the objdump showed, and why this level is worth installing:

```c
void SYSRTC_SEQ_IRQHandler(void)
{
  _DAT_500aa060 = _DAT_500a8060 & _DAT_500a8064;   /* GRP0_IF_CLR = GRP0_IF & GRP0_IEN */
  return;
}
```

The decompiler names the registers itself: `0x500a8060` and `0x500a8064`. That is the group interrupt block at
**+0x60/+0x64**, while every header in the SDK (both the gecko_sdk v4.4.5 and the newer simplicity_sdk) puts
`GRP0_IF` at **+0x40** and `GRP0_IEN` at **+0x44**.

## The offsets, and the rule

| Register | PAC | Vendor header | **Blob** | Silicon |
|---|---|---|---|---|
| `CMU.SYSRTC0CLKCTRL` | 0x1DC | 0x240 | – | 0x240 works, 0x1DC reads 0 |
| `SYSRTC.EN_SET` | – | 0x1004 | – | 0x1004 works, 0x1000 leaves `EN=0` |
| `SYSRTC.CNT` | 0x18 | 0x18 | **0x18** | read stalls (open) |
| `SYSRTC.GRP0_IF` | 0x40 | 0x40 | **0x60** | writing 0x60 changes behaviour, 0x40 does not |
| `SYSRTC.GRP0_IEN` | 0x44 | 0x44 | **0x64** | – |
| Alias pages | – | +0x1000/+0x2000/+0x3000 | `0x500a9000`, `0x500aa000` in literal pools | consistent |

**The rule:** when a register access misbehaves, read the blob before believing a header — and when the blob
and the silicon still disagree, believe the silicon and write down both numbers. Two of these three
corrections came from disassembly, and one of them (the group block) explained why a compare channel that
looked configured had never been configured at all.

## Where the output goes: not in this repository

**Decompiled output is never committed** (user directive, 2026-11-13: *"nao quer o codigo descompilado no
repo!"*). Two reasons, and the first one is reason enough:

- the blobs are MSLA-licensed binaries; a repo containing their reconstructed code is a licensing problem
  whatever its intent, and this repository is public;
- a decompiler's output is not a source of truth to be copied around — it is evidence to be read once and
  then expressed as a *finding* (an offset, a sequence, a contract) in prose or in our own Rust.

So the workflow is: run REA against the ELF, read the pseudocode, and write **what it establishes** into a
document like this one. The raw output stays outside the tree — `/projects/_tmp/efr32mg24-rs/` while it is
being read, and nothing of it afterwards. What this file keeps is the conclusion, which is the part that is
ours and the part the next person needs.
