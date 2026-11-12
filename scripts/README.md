# scripts/

Every script in this directory, what it does, what it touches, and whether it
still works on the current dev machine (the Linux container on `lnx`).

Status legend:

- **current** — works here, part of the active workflow.
- **macOS only** — depends on the bundled Mach-O `openocd` binary or on
  `~/Library/Arduino15`; it cannot run in the container and does not run on lnx.
- **safety** — a guard; it never writes to the board on its own.

| script | kind | status |
|---|---|---|
| `build-pac-examples.sh` | build matrix | current |
| `build-container.sh` | container build | current (podman) |
| `flash-lnx.sh` | flash (guarded) | current |
| `gate.sh` | gate (no hardware) | current |
| `read-fall-log.sh` | read / decode (SWD) | current |
| `elf-load-range.py` | safety / inspection | current |
| `elf-sections.py` | safety / inspection | current |
| `inventory-pac-examples.py` | safety / inspection | current |
| `build-crates.sh` | build matrix (workspace) | current |
| `openocd-mg24-mini.cfg` | SWD read path (no flash driver) | current |
| `read-pin-lnx.sh` | verification: pin state over SWD | current |
| `read-mailbox.sh` | verification: RAM mailbox over SWD | current |
| `dump-flash-lnx.sh` | backup: flash dump over SWD | current |
| `repo-tidy-pac-artifacts.sh` | repo hygiene: move artefacts out | current |
| `repo-tidy-pac-examples.sh` | repo hygiene: retire untested examples | current |
| `erase_and_flash.sh` | flash | macOS only (legacy) |
| `flash-app-only.sh` | flash | macOS only |
| `flash-arduino-only.sh` | flash | macOS only |
| `flash-bootloader-arduino.sh` | flash | macOS only |
| `flash-bootloader-test.sh` | flash | macOS only |
| `flash-mass-erase.sh` | flash (destructive) | macOS only |
| `restructure-crates.sh` | repo maintenance | done once, idempotent |
| `rewrite-crate-paths.py` | repo maintenance | done once |
| `README.md` | this file | — |

---

## Build

### `build-pac-examples.sh [--features rt]`

Builds **every** example in `crates/efr32mg24-pac/examples/` and prints one
`OK`/`FAIL` line per example plus a total. Per-example logs land in
`_tmp/build-logs/pac-<name>.log`; the exit code is non-zero if any example
fails, so it can gate CI.

Why: the PAC examples are the first rung of the ladder and they rot silently
when the PAC API changes. `blink_pac` has been failing since the svd2rust 0.37
`bits()` API change and nothing noticed.

### `build-crates.sh`

Builds each workspace crate once and prints `OK`/`FAIL` per crate, so a broken
crate is visible instead of being discovered at dispatch time. As of
2026-11-10 `efr32mg24-bluetooth-sys` fails (bindgen union codegen) and
`xiao-mg24-bsp --features ble` with it; see
`docs/03-development/BUILD-STATUS-2026-11-10.md`. Logs in `_tmp/build-logs/`.

### `build-container.sh [--example NAME] [--release] [--clean]`

Builds the workspace inside the podman image described by `Containerfile`
(pinned Rust 1.75 + `thumbv8m.main-none-eabihf` + `gcc-arm-none-eabi` +
`cargo-binutils`). Use it when the host toolchain drifts or when reproducing a
build on another machine. Not needed for day-to-day work in the container,
which already has the toolchain.

Note: `Containerfile` pins Rust 1.75, the host toolchain is newer. A green
container build is therefore *weaker* evidence than a green host build, not
stronger.

---

## Flash

### `flash-lnx.sh <elf> --confirm [--run]`

The only supported way to write an image to the board. Builds happen here,
flashing happens on `lnx` (host `10.10.10.217`, user `mvcorrea`) because that
is where the board is plugged in.

Guards, all fail-safe (each one refuses rather than proceeding):

1. without `--confirm` it is a **dry run** — it prints the image's address
   range, checks for the probe, and stops;
2. the image must parse as 32-bit little-endian ELF;
3. `load_min >= 0x08006000` — anything lower would overwrite the Gecko
   bootloader at `0x08000000-0x08005FFF`;
4. `load_max < 0x08180000` — the end of the 1512 KiB application region;
5. exactly **one** CMSIS-DAP probe must be visible on lnx;
6. after writing, `probe-rs verify` reads the flash back and compares — a
   write is only reported as done when the comparison succeeds.

It does **not** reset the core unless `--run` is passed, so the operator
decides when the new image starts.

```bash
cargo build --release -p efr32mg24-pac --example blink_xiao --features rt
scripts/flash-lnx.sh \
  /projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/blink_xiao   # dry run
scripts/flash-lnx.sh \
  /projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/blink_xiao --confirm
```

Verified 2026-11-10 both ways: the real image passes all guards; a control
image whose `p_paddr` was patched to `0x08000000` is refused with exit 3.

### `erase_and_flash.sh` (macOS only)

Legacy. Hardcodes `~/Library/Arduino15/packages/SiliconLabs/tools/openocd/...`
and programs `blink_bare_patched.hex` from the repository root after erasing
`0x08006000 + 0x178000`. Cannot run here (no Arduino15, no macOS openocd).

### `flash-app-only.sh`, `flash-arduino-only.sh`, `flash-bootloader-arduino.sh`, `flash-bootloader-test.sh` (macOS only)

All four drive `crates/efr32mg24-bootloader/openocd/bin/openocd`, which is a
**Mach-O x86_64 macOS binary** — it cannot execute in this container or on
lnx. They program one or both of `bootloader.hex` and `test-app.hex` /
`arduino_blink.hex` through `interface/cmsis-dap.cfg` +
`target/efm32s2_g23.cfg`, then `reset run`.

Kept as the record of how the Gecko bootloader was installed. Do not use them
as-is on Linux; install a Linux openocd or use `flash-lnx.sh`.

### `flash-mass-erase.sh` (macOS only, destructive)

Erases the whole 1.5 MiB at `0x08000000` and then rewrites bootloader + test
app. This is the recovery path for a board whose application is broken, and
the last thing to run on a healthy board: it removes the bootloader, so a
failure between erase and program leaves a blank chip.

---

## Safety and inspection

### `elf-load-range.py <elf>`

Prints `load_min`, `load_max`, total size and one line per `PT_LOAD`. This is
the check `flash-lnx.sh` runs before touching the board. Exit codes: `0` ok,
`2` not a supported ELF, `3` no loadable segments.

This script is why the 2026-11-10 defect was found: every image built before
`--nmagic` was added to `.cargo/config.toml` carried a 372-byte `PT_LOAD` at
`0x08000000` — the ELF header, page-aligned down into the bootloader region.
`probe-rs` silently skips that segment (verified by reading the board's flash
at `0x08000000`: it still holds the bootloader's vector table,
`08 10 00 20 71 01 00 08`), but openocd and Simplicity Commander are not known
to, and an image that writes it destroys the bootloader.

### `elf-sections.py <elf>`

Lists section headers with address, size and flags. Use it when a segment
looks wrong: it shows which section actually lands at which address, and it is
what proved no section — only the header — lived at `0x08000000`.

### `inventory-pac-examples.py`

Classifies every PAC example as `TYPED` (PAC typed accessors, SVD-derived
offsets) or `RAW` (`write_volatile` to a hardcoded address), and flags the ones
that touch clock-source selection (`sysclkctrl`, `oscencmd`, `hfxo`, `clksel`,
`emu`, `wdog`, `lock`) — the class of write that bricked the board on
2026-06-29. Run it before trusting any example on hardware.

Current output: `blink_minimal` and `blink_xiao` are the only two that match
the clock keywords, and in `blink_xiao` the match is the comment block that
documents the incident, not a register write.

---

### `read-pin-lnx.sh [reg] [bit] [samples]`

Samples a GPIO port DOUT register over SWD (`halt`, `mdw`, `resume` per
sample) and reports one bit per sample plus a verdict. This is how "the LED
blinks" became data on 2026-11-10: eight samples of `0x5003C040` bit 7 gave
`0 0 1 1 0 1 1 0`, i.e. PA7 toggles, i.e. the PAC-only image is executing its
loop. Read-only: nothing is written to the target.

### `read-mailbox.sh`

Reads the 216-byte diagnostic record at `0x2003_F000` written by examples that
include `crates/efr32mg24-pac/examples/common/mailbox.rs`, and prints the
sequence number, status and message. Read-only.

### `openocd-mg24-mini.cfg`

Minimal OpenOCD target (Cortex-M33 via `swj_newdap` + `dap create`) with **no
flash driver**, so it can read memory but cannot program the part. Exists
because probe-rs 0.32 can wedge its own CMSIS-DAP session (`Command ID in
response (0x14) does not match sent command ID`) while OpenOCD still talks to
the same probe. This is the read path that survives that wedge.

### `dump-flash-lnx.sh <out.bin> [chunk] [base] [size]`

Chunked flash dump for the original-image backup, with per-chunk retry: a
single 1.5 MiB `probe-rs read` wedges this CMSIS-DAP, hence the chunking.

### `repo-tidy-pac-artifacts.sh`, `repo-tidy-pac-examples.sh`

Repository hygiene, both dry-run by default and both *moving* files out to
`/projects/_tmp/efr32mg24-rs/pac-artifacts/` rather than deleting them:

- the first moved 53 loose artefacts out of the PAC crate (31 hex files, three
  2.5 MiB images, the 21 MiB vendor pack, a 92 MiB Silabs C project, a 22 MiB
  source backup, nine ad-hoc scripts) plus 40 unused SVD variants — the
  repository went from 470 MiB to 59 MiB;
- the second retired 14 blink variants that are not in the verification
  ladder, leaving six examples that are flashed and measured (or about to be).

## Repository maintenance

### `restructure-crates.sh`

Moved all seven crate directories into `crates/` (2026-11-10). Idempotent-ish:
it refuses if `crates/<name>` already exists. Every intra-workspace path
dependency is a sibling reference, so the move needed no `Cargo.toml` edits.

### `rewrite-crate-paths.py`

The companion pass: rewrites `efr32mg24-pac/…` → `crates/efr32mg24-pac/…` in
living docs, scripts and CI files, deliberately skipping `CHANGELOG.md`,
`docs/03-development/LOG.md`, `docs/99-archive/`, `book*/` and `.archive/`,
because those record where files were at the time and rewriting them would
falsify the history.

### `gate.sh`

Everything that can be checked without the board, in one run: `cargo fmt --all --check`,
clippy at **zero warnings** for the four crates that build, every example, and the host
tests of `motion-detection`. It prints a `PASS`/`FAIL` row per check and a final
`VERDICT:`. It names the two crates it does **not** gate (the Bluetooth and RAIL sys
crates, whose bindgen output does not compile) instead of skipping them silently — a gate
that hides a broken crate is worse than no gate.

The example-in-a-directory check is worth knowing about: an `examples/foo/main.rs` is only
built by `--examples` if the manifest declares it, so the gate checks the **declaration**
and prints the name it is built under (`fall_detector/` is declared as `base_fall_detector`).
A directory nobody declared compiles never and fails silently.

### `read-fall-log.sh`

Dumps and decodes the fall detector's state over SWD: the event log, the counters
(`FALLS`, `IMPACTS`, `AIRBORNE`), the waveform ring and the per-sample intervals. Every
address is read from the ELF it is given, because these statics move whenever one is added
and a baked-in address is a silent wrong answer. It analyses only the samples the board has
actually written: the ring's unwritten slots read as zero, and a zero sample looks exactly
like free fall.

Measured with it on 2026-11-11: mean interval 4249 µs (235 Hz), display stalls of 63.75 ms,
and 0 events on a board at rest.
