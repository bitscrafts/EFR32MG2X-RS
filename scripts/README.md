# Scripts

Everything here is a bench tool, a guard, or an instrument. Nothing is generated, and nothing is dead: if a script
existed only to perform a one-off migration, it has been retired (see the end of this file) rather than kept as
archaeology.

Three conventions hold throughout:

- **Host access is not baked in.** A script that needs the machine with the debug probe takes it from `FLASH_HOST`
  (and `FLASH_USER`, defaulting to your own user). Nothing in this repository knows the name or address of anybody's
  bench.
- **Read-only means read-only.** The inspection scripts halt the core only long enough to read, and resume it. Nothing
  writes to the target unless its name says so.
- **A check must be able to fail.** A script that reports PASS must derive that word from a measurement, not from the
  absence of an error message.

## The gate and the build

| script | what it does | needs the board |
|---|---|---|
| `gate.sh` | The definition of healthy: build every crate, clippy at zero warnings, host tests for the domain crates, every example, message-buffer shapes, layer boundaries. Prints one `VERDICT:` line and exits non-zero on failure. This is what CI runs. | no |
| `check-examples-build.sh` | Builds **every** example with the features it declares. `cargo build --examples` silently skips any example that has `required-features` — which is all the Bluetooth and BSP ones — so this script names each one. | no |
| `check-copy-len.py` | Finds `copy_from_slice` calls whose length is not the destination's. Three defects in this project came from hand-counted lengths. | no |
| `check-example-class.sh` | Builds a BLE example and reports whether a given BGAPI class actually reached the image, i.e. whether the linker pulled the archive member. | no |
| `build-crates.sh` | Builds every workspace crate once, printing OK/FAIL per crate. A coarser view than the gate, useful when only linking is in doubt. | no |
| `cargo-clean-all.sh` | `cargo clean` across the workspace, with the reclaimed size measured before and after. | no |

## Flashing — one path only

| script | what it does | needs the board |
|---|---|---|
| `flash.sh` | **The only way to write flash.** Dry run by default; `--confirm` to write; refuses a load range that would touch the bootloader below `0x08006000`; verifies by reading the image back; `--run` resets into it. | yes |
| `elf-load-range.py` | Prints the load addresses, sizes and per-segment ranges of an ELF. Run it before flashing anything you did not just build. | no |
| `elf-sections.py` | Section sizes of an ELF, largest first — where the flash went. | no |
| `dump-flash.sh` | Reads the whole 1.5 MiB flash to a file, in chunks. Read-only, and the first thing to do before anything risky. | yes |
| `uf2-range.py` | Validates a UF2 file's target addresses before the bootloader sees it. | no |

## Reading the board over SWD (read-only)

| script | what it does | needs the board |
|---|---|---|
| `read-mailbox.sh` | Reads the diagnostic mailbox at `0x2003_F000`: the firmware's own account of what it did, with the fields that explain a failure. Retries a torn record instead of reporting "no data". | yes |
| `clear-mailbox.sh` | Invalidates the mailbox before flashing a new image. **A mailbox record survives a reset**: without this, the record you read belongs to the previous firmware. | yes |
| `read-memory.sh` | Reads a range of target memory. For a snapshot only: the firmware keeps allocating between two reads, so a hand-walked structure desynchronises. | yes |
| `where.sh` | Halts the core and prints `pc`, `lr` and `sp`, then resumes. This is how a panic is located — `addr2line` on the `pc` beats re-reading the source. | yes |
| `measure-blink-rate.sh` | Measures the LED's toggle rate over SWD, so "the LED blinks" is a number and not an opinion. | yes |
| `read-led.sh` | Reads PA7's mode, output latch and pad input across several halts, and says whether the pin is actually being driven. | yes |
| `read-pin.sh` | Samples one bit of a GPIO port's `DOUT` over many halts — a slow logic analyser for a pin you cannot reach. | yes |
| `read-rate.sh` | Measures how fast a counter grows, in wall-clock seconds. | yes |
| `read-fall-log.sh` | Reads and decodes the fall detector's state. | yes |
| `device-identifiers.sh` | Reads every identifier of every device on the bench from the devices themselves (probe serials, BD addresses, USB ids) — the numbers a report needs when two boards are on the desk. | yes |
| `openocd-mg24-mini.cfg` | The OpenOCD config for this target. Used by every SWD script above; `probe-rs` can wedge its own CMSIS-DAP session, OpenOCD does not. | — |

## Bluetooth: the board under test

| script | what it does | needs the board |
|---|---|---|
| `ble-matrix-all.sh` | **The matrix.** For every BLE example: build, flash, clear the mailbox, read the record, capture the air from a separate host, print one table row. The row is only a pass if the air agrees with the firmware. | yes |
| `ble-scan-bluez.sh` | Asks whether BlueZ's own stack accepts the board's advertisement, or only a raw HCI capture does. | yes |
| `ble-scan-raw.sh` | Scans at 100% duty cycle through raw HCI and reports every advertiser — the widest window this adapter can give. | yes |

## Bluetooth: the host as observer

The host is the witness. Its own limits are part of every measurement: the adapter captures roughly 8% of what is on
the air, and a device learned in one scan is *temporary* — connecting from a new process answers "not available" for a
board that is transmitting perfectly.

| script | what it does | needs the board |
|---|---|---|
| `host-env-verify.py` | **The deliverable check.** One Bluetooth session: scan, connect, read the three Environmental Sensing characteristics, subscribe, and decode what arrives into °C, %, hPa. Prints a `VERDICT:`. | yes |
| `host-gatt-probe.py` | The GATT echo test: writes a characteristic, watches for the firmware echoing those bytes in its notifications. A read-back cannot prove this; an echo can. | yes |
| `host-pair-probe.py` | Drives a pairing attempt over D-Bus, because `bluetoothctl pair` does nothing at all, silently, when the device is not in BlueZ's cache. | yes |
| `host-hold.sh` | Connects and holds the link, subscribing only if asked — the instrument that separated "the connection breaks the heap" from "notifications break the heap". | yes |
| `host-advertise.sh` | Makes the host's adapter transmit an advertisement we control byte by byte, as a known-good transmitter. | no |
| `host-aircheck.sh` | Counts how often the board appears on air in a fixed window. | yes |
| `host-payload.sh` | Prints the distinct frames the board emits during a scan, so a payload can be read rather than counted. | yes |
| `host-rawscan.sh` | Counts the board's reports through raw HCI, without BlueZ's discovery. `bluetoothd` can stop reporting entirely while the radio is fine; this tells those two cases apart. | yes |

## Retired, and why

These lived in this directory and were moved out rather than deleted. Each one either performed a one-off migration, was
superseded by a script above, or bypassed a guard that exists for a reason.

- `ble-example-matrix.sh`, `ble-examples-status.sh`, `ble-run-and-report.sh`, `ble-examples-required-features.sh` —
  superseded by `ble-matrix-all.sh`, which does all of it in one pass and cannot report a row it did not run.
- `host-env-probe.py`, `host-env-read.sh`, `host-env-session.sh`, `host-env-bluetoothctl.sh` — the attempts that led to
  `host-env-verify.py`. They kept failing in ways that taught the rules in the table above; the rules survive them.
- `flash-mass-erase.sh`, `flash-bootloader-test.sh`, `erase_and_flash.sh`, `flash-app-only.sh`,
  `flash-bootloader-arduino.sh`, `flash-arduino-only.sh` — the bootloader-era helpers. They could write below
  `0x08006000`; `flash.sh` refuses to. The bootloader work is finished and recorded in
  `docs/03-development/bootloader-session.md`.
- `repo-tidy-*.sh`, `restructure-crates.sh`, `rewrite-crate-paths.py`, `inventory-pac-examples.py` — one-off migrations,
  already applied.
- `build-container.sh`, `build-pac-examples.sh` — subsets of `gate.sh`.
- `ralph.sh`, `run-claude-finish.sh` — agent dispatch harnesses belonging to an orchestrator, not to this crate.
