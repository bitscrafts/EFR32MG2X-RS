# Building here, flashing on lnx

**Date**: 2026-11-10
**Scope**: how firmware gets built and written to the XIAO MG24 from this
workspace, and what is and is not verified about it.

---

## Machine split

| where | what |
|---|---|
| this container (`/projects/efr32mg24-rs`) | source, cargo build, tests, ELF inspection |
| `lnx` (10.10.10.217, user `mvcorrea`) | the board is plugged in here; `probe-rs` lives here |
| board | XIAO MG24 Sense, EFR32MG24B220F1536IM48-B, CMSIS-DAP on the onboard SAMD11 |

Tooling measured on `lnx` on 2026-11-10:

```
probe-rs 0.32.0 (git commit: 48f5e4d)      /usr/local/bin/probe-rs
USB 2886:0062  Seeed Studio XIAO MG24 (Sense) CMSIS-DAP
/dev/ttyACM0   present, user in group dialout
openocd        NOT installed
commander      NOT installed
rustc/cargo    NOT installed
```

The probe node is `root:root 0664`, so `probe-rs` must run under `sudo -n`;
without it the device is listed as `(inaccessible)`. The chip name to pass is
`EFR32MG24B220F1536IM48` — `probe-rs 0.32` lists it under `EFR32MG24 Series`.

Rust and cargo are **not** installed on lnx, so a build there is not an option:
images are built in this container and copied over.

---

## The workflow

```bash
# 1. build (in the container)
cargo build --release -p efr32mg24-pac --example blink_xiao --features rt

# 2. dry run: address range + probe check, nothing written
scripts/flash-lnx.sh \
  /projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/blink_xiao

# 3. write, with the guards satisfied
scripts/flash-lnx.sh \
  /projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/blink_xiao --confirm

# 4. start the image only when you decide to
scripts/flash-lnx.sh <same elf> --confirm --run
```

`scripts/flash-lnx.sh` refuses to write when the image would touch
`0x08000000-0x08005FFF` (the 24 KiB Gecko bootloader), when more than one probe
is visible, or when the ELF cannot be parsed. After writing it runs
`probe-rs verify`, which reads the flash back — the write is only reported as
done when the comparison succeeds.

---

## Defect found on 2026-11-10: images had a PT_LOAD inside the bootloader

Every image built before this date carried four `PT_LOAD` segments, the first
of which was **372 bytes at `0x08000000`**:

```
#1 type=1 off=0x0 vaddr=0x08000000 paddr=0x08000000 filesz=372 align=65536
#2 ... 0x08006000 344   (.vector_table)
#3 ... 0x08006158 252   (.text)
#5 ... 0x08006260  88   (.app_properties)
```

No section lived at `0x08000000` (`scripts/elf-sections.py` shows sections only
from `0x08006000` up). Segment #1 was the ELF header itself, page-aligned down
to the 64 KiB boundary — the classic `ld`/LLD page-alignment artefact, harmless
on a hosted OS and dangerous on a part where `0x08000000` is a bootloader.

Consequences, measured rather than assumed:

- `probe-rs 0.32` **skips** that segment. Proof: reading the board's flash at
  `0x08000000` after flashing returns the bootloader's own vector table
  (`08 10 00 20 | 71 01 00 08` = SP `0x20001008`, PC `0x08000171`), not the ELF
  header (`7f 45 4c 46 …`). The claim in `QUICK_FLASH_GUIDE.md` that
  `probe-rs download` leaves the bootloader alone held — for `probe-rs`.
- openocd and Simplicity Commander were not tested and are not known to do the
  same. A tool that honours every `PT_LOAD` would write 372 bytes over the
  bootloader's vector table.

Fix: `--nmagic` in `.cargo/config.toml` (LLD stops page-aligning). Rebuilt
`blink_xiao` now reports:

```
load_min=0x08006000 load_max=0x080062B7 size=696 segments=3
```

Three segments, all inside the application region. The guard in
`flash-lnx.sh` passes it, and refuses a control image whose `p_paddr` was
patched back to `0x08000000` (exit 3).

Any `.hex` file produced elsewhere is unaffected by this change and still needs
its own address check (a hex record carries its own address).

---

## What is verified, and what is not

Verified on the board today:

- the probe enumerates and the chip is reachable over SWD
  (`sudo -n probe-rs list`; `probe-rs read` at `0x08000000` returns data);
- the bootloader's vector table at `0x08000000` is intact;
- the application region holds a `blink_xiao` image whose bytes match the ELF
  (`probe-rs verify` → `Verification successful`).

**Not** verified: that the LED actually blinks with the current image, and that
any HAL or BSP example runs. The last hardware-verified fact in the project is
the 2026-06-29 note in `blink_xiao.rs` (LED on PA7 blinking after a
`probe-rs run`), and the bricking incident that followed it came from the HAL
clock path, not from the PAC path.

No image may be flashed while the open question below is open.

---

## Open question before the next flash

`Clocks::new` no longer switches SYSCLK to HFXO — the sequence that bricked the
board is gone, and the function now only *records* the requested frequency.
But it records what was asked for, not what is running:

```rust
let hfclk = if let Some(hfxo_config) = config.hfxo {
    hfxo_config.frequency          // 39 MHz "assumed"
} else {
    Hertz(Self::HFRCO_FREQ)
};
```

If the caller asks for HFXO and the core is actually on FSRCO (~19 MHz), every
delay computed from `pclk()` is wrong by ~2x, and any baud rate derived from it
is wrong too. Before the HAL is used on hardware, the running clock has to be
read from `CMU_S.STATUS`/`SYSCLKCTRL` rather than assumed, or the HAL has to
implement the enable → wait-for-ready → switch sequence properly.
