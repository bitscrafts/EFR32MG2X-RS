# Building here, flashing where the probe is

**Date**: 2026-11-10
**Scope**: how firmware gets built and written to the XIAO MG24 from this
workspace, and what is and is not verified about it.

---

## Machine split

| where | what |
|---|---|
| this container (`/projects/efr32mg24-rs`) | source, cargo build, tests, ELF inspection |
| the flash host (`$FLASH_HOST`, `$FLASH_USER`) | the board is plugged in here; `probe-rs` lives here |
| board | XIAO MG24 Sense, EFR32MG24B220F1536IM48-B, CMSIS-DAP on the onboard SAMD11 |

Tooling measured on the flash host on 2026-11-10:

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

Rust and cargo are **not** installed on that host, so building there is not an option:
images are built in this container and copied over.

---

## The workflow

```bash
# 1. build (in the container)
cargo build --release -p efr32mg24-pac --example blink_xiao --features rt

# 2. dry run: address range + probe check, nothing written
scripts/flash.sh \
  /projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/blink_xiao

# 3. write, with the guards satisfied
scripts/flash.sh \
  /projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/blink_xiao --confirm

# 4. start the image only when you decide to
scripts/flash.sh <same elf> --confirm --run
```

`scripts/flash.sh` refuses to write when the image would touch
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
`flash.sh` passes it, and refuses a control image whose `p_paddr` was
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

### 2026-11-10: the blink was verified without eyes

`scripts/read-pin.sh 0x5003C040 7 8` samples PORTA_DOUT bit 7 over SWD,
halting and resuming the core between samples. Result:

```
sample 1: dout=0x00000000 bit7=0
sample 2: dout=0x00000000 bit7=0
sample 3: dout=0x00000080 bit7=1
sample 4: dout=0x00000080 bit7=1
sample 5: dout=0x00000000 bit7=0
sample 6: dout=0x00000080 bit7=1
sample 7: dout=0x00000080 bit7=1
sample 8: dout=0x00000000 bit7=0
VERDICT: bit 7 toggles -> firmware is executing the toggle loop
```

Supporting reads from the same session: `0x08006000` holds
`20001208 08006159 0800623d 0800624f` (the app vector table: SP 0x20001208,
reset 0x08006158), and two halts taken ~300 ms apart report `pc=0x0800060c`
(inside the bootloader) then `pc=0x08006212` (inside the app's `.text`). The
image on the board is `blink_xiao`, PAC only, no HAL.

Reads use OpenOCD (`scripts/openocd-mg24-mini.cfg`), not probe-rs: probe-rs
0.32 wedged its own CMSIS-DAP session at 03:05 during an attempt to read the
whole 1.5 MiB flash in one transfer, and afterwards every probe-rs command
fails with `Command ID in response (0x14) does not match sent command ID`
while OpenOCD talks to the same probe normally. A USB port reset
(`usbreset`), an `authorized` toggle and an unbind/bind did not clear it; a
physical replug will. Nothing was written to the board by any of this.

**Also not verified**: the LED itself, only the pin. PA7 being the user LED
comes from the Silicon Labs Arduino core (`variants/xiao_mg24/pins_arduino.h`:
`LED_BUILTIN 20`, `LED_BUILTIN_ACTIVE LOW`) and is consistent with the
project's 2026-06-29 hardware note.

No image may be flashed while the open question below is open.

---

## 2026-11-10: the guarded flash path, exercised end to end

The first image written under this procedure was `pac_blink_double` (two short
pulses, then a pause). Every step below is a measurement.

```
guards        load_min=0x08006000 load_max=0x08006517 size=1304, 3 segments
probe         exactly 1 CMSIS-DAP on the flash host
write         probe-rs download   Finished in 0.49s
read-back     probe-rs verify     Verification successful
```

Bootloader region, read over SWD before and after the write:

```
before  0x08000000: 20001008 08000171 0800265d 08002201
after   0x08000000: 20001008 08000171 0800265d 08002201   (identical)
```

That is the proof the procedure protects the bootloader: the same four words,
before and after, read from the chip rather than asserted by the tool.

The application actually runs:

```
0x08006000: 20001208 08006159 080063ed 080064c3    (vector table of the new image)
pc = 0x08006340                                    (inside its .text)
PORTA_MODEL mode7 = 4 (push-pull), DOUT bit7 toggles, DIN follows in 6/6 samples
```

And the LED was **seen blinking by the operator**, in the requested shape
(two quick blinks, then a pause). That is the one piece of evidence SWD cannot
produce, and it now agrees with the register-level evidence.

### What the measurement says about the clock (and why the next step is a timer)

`measure-blink-rate.sh 8` counted 56 toggles in 9.10 s → 6.16 toggles/s. With
`pulses=2, pulse_iters=500_000, gap_iters=2_000_000` a group is 4 toggles and
4 M iterations, so a group lasts ~0.65 s, i.e. ~160 ms per pulse phase and
~640 ms of pause. That implies roughly 19 MHz of SYSCLK — FSRCO, exactly what
the core runs on when `probe-rs reset` starts the image directly instead of
letting the bootloader set the clock tree.

Consequence: **a NOP-loop blink is not portable**. Flash the same image after a
power cycle (bootloader path, HFXO/HFRCODPLL) and it runs ~2–4× faster. Timing
that must be true regardless of how the board started has to come from a
peripheral timer, not from a loop count — which is the next example.

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
