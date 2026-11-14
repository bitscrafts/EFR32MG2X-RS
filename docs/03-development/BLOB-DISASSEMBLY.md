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

What this produced for SYSRTC, expressed as the finding rather than as the vendor's bytes:

The handler for the module's sequencer interrupt takes **one** base pointer from a literal pool and, through it,
reads the group status at **+0x60** and the group interrupt enable at **+0x64**, ANDs the two, and writes the result
back at the **same offsets through the +0x2000 page** — the write-1-to-clear alias. Three facts follow, and none of
them is in a header: the group block sits at +0x60 (every header says +0x40), the clear alias is a page offset
rather than a register, and the handler clears only the channels that are enabled.

and, in `RTCCSYNC_GetRtccCount`, a plain read of `CNT` at **+0x18**, which is the offset we use and confirms it.

## Level 2 — a decompiler, for pseudocode and cross-references

`objdump` gives instructions; the question is usually about control flow, so a decompiler earns its place: it
recovers the function boundaries, the arguments of every call, and which constants reach which register. That is
how the twelve functions behind the `0x040C` (`SL_STATUS_BT_RADIO`) failure were read.

The decompiler is a tool of the bench, not of the repository. Its installation, versions and invocation are not
documented here on purpose: what matters to a reader is the *finding* and how it was checked against the silicon,
and both live in `docs/04-reference/BLE-BLOB-FINDINGS.md`. A tool pinned in a project document ages badly and gets
copied into places it does not belong.

Two rules survive from those sessions and they are not about the tool:

- **Rename everything before you believe anything.** A decompiler's names (`uVar12`, `param_2`) carry no meaning;
  the meaning comes from the call sites and from the header that declares the function.
- **A decompiled function is a hypothesis until the silicon agrees.** Where the pseudocode and the measured
  behaviour disagree, the measurement wins, and the disagreement is the most informative thing in the session.

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

So the workflow is: run the decompiler against the ELF, read the pseudocode, and write **what it establishes** into a
document like this one. This document was itself in breach of that rule until 2026-11-13 — it carried a short
objdump excerpt and a decompiled function body while thirty lines below it said not to. Both were replaced by the
finding in prose, and the version with the vendor's bytes was moved to
`/projects/_tmp/efr32mg24-rs/private-docs/BLOB-DISASSEMBLY.with-vendor-output.md`, outside the tree. A rule that is
written in the middle of a document does not stop the top of the same document from breaking it. The raw output stays outside the tree — `/projects/_tmp/efr32mg24-rs/` while it is
being read, and nothing of it afterwards. What this file keeps is the conclusion, which is the part that is
ours and the part the next person needs.
