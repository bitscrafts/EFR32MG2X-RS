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
| `elf-load-range.py` | safety / inspection | current |
| `elf-sections.py` | safety / inspection | current |
| `inventory-pac-examples.py` | safety / inspection | current |
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
