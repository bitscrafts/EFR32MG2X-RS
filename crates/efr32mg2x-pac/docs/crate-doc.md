# EFR32MG24 register access, from the SVD

Typed, zero-cost access to every peripheral of the **EFR32MG24B220F1536IM48** — the part on the Seeed XIAO MG24
Sense. The `Peripherals` struct is a singleton: taking it proves at compile time that no two pieces of code own the
same registers.

```rust
use efr32mg2x_pac::Peripherals;

let p = Peripherals::take().unwrap();               // None on a second call: the token is the proof
p.cmu_s.clken0().modify(|_, w| w.gpio().set_bit());  // gate the GPIO clock
p.gpio_s.porta_dout().write(|w| unsafe { w.bits(0x80) }); // PA7 high
```

The fields are snake_case (`cmu_s`, not `CMU_S`) because `svd2rust` lowercases the peripheral names it reads from the
SVD; a page that shows the upper-case spelling sends the reader to a compile error.

## Where this came from, exactly

Generated with **svd2rust 0.37.1** from the Silicon Labs SVD for this part. Everything below `src/` is that output:
2,633 files, one module per peripheral, with register and field types derived from the SVD. Nothing in those files is
hand-edited, and regenerating must not be either — the hand-written documentation you are reading lives in
`docs/crate-doc.md` and is included from `lib.rs`, which is the one line a regeneration has to preserve.

## The features

| feature | effect |
|---|---|
| `mg24` | selects the device: EFR32MG24B220F1536IM48, the part this crate is generated for. On by default, and turning it off without naming another device is a `compile_error!`. |
| `critical-section` | on by default; pulls the `critical-section` crate so `modify()` can exclude interrupts. The architecture crate supplies the implementation (`cortex-m/critical-section-single-core`). |
| `rt` | pulls `cortex-m-rt` and gives you the interrupt vector table. Without it the crate is pure register definitions, which is what a bootloader or a test harness wants. |

## What the SVD does not tell you

Four things this project measured on the die, which the headers and the SVD disagree about. They are recorded here
because a register access crate that hides them sends the reader to a wrong datasheet page:

- **There is no `DOUTTGL` register on this part.** Read-modify-write `DOUT` instead. A toggle-by-write compiles
  against other EFR32 families and does nothing here.
- **`MODEH` exists only for PORTA and PORTC.** Port B and D have no high-mode register; touching one writes into the
  neighbouring register.
- **`I2C.IF` is only cleared through the `IF_CLR` alias** (the window at `+0x2000`), and `I2C0`'s non-secure view reads
  zero for it. Writing 1 to the plain register clears nothing, and a flag left set -- a NACK, say -- changes what the
  next transaction does before it touches the bus. Measured, with the recovery sequence, in
  `docs/04-reference/I2C-RECOVERY-STATE.md`.
- **The delay is 2.3x short** if you count cycles from `CMU`'s default clock. The clock is what is wrong, not the
  counter (`docs/04-reference/DELAY-CLOCK-DEFECT.md`).

## Using it through the layers above

Most code should not touch this crate directly. `efr32mg2x-hal` wraps these registers in drivers that cannot be
misused (a pin that knows its port, an I2C transfer that owns the bus), and `efr32mg2x-xiao-bsp` /
`efr32mg2x-expansion-base-bsp` wrap the boards. This crate is the floor, and it is here so that the layers above can
be read and audited against the silicon.

The examples under `examples/` cover the LED ladder, the button, the I2C scan, the BME680, the OLED, the microphone, the
IMU, the RTC, the watchdog and the mailbox that lets a board report what it did with no debugger attached. The list and
the date each one was watched on the board are generated from the files themselves in `docs/examples.md` — no count is
written here, because a count in prose is a number that goes stale silently.

An earlier version of this page said all of them were "verified on hardware" when one was. The rule that came out of it:
a date in the file, or a `--` in the generated table, and nothing else counts as watched.

## Licence

MIT OR Apache-2.0. The SVD is Silicon Labs'.

## Staying out of the bootloader

The first 24 KiB of flash (`0x08000000`-`0x08005FFF`) belong to the Gecko bootloader. An image that writes there stops
the board from booting and only a debugger brings it back. Three things have to be true, and the crate can only give
you the first two:

**1. The linker script.** One is shipped with this crate; use it, or copy its numbers:

```
FLASH : ORIGIN = 0x08006000, LENGTH = 1512K
```

**2. The flags.** In `.cargo/config.toml`, for the target:

```toml
[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "link-arg=-Tlink.x", "-C", "link-arg=--nmagic"]
```

`--nmagic` is not cosmetic. Without it LLD aligns the first segment down to the 64 KiB page boundary and emits a
372-byte `PT_LOAD` at `0x08000000` — inside the bootloader. `probe-rs` happens to skip that segment; `openocd`,
Simplicity Commander and any naive loader may not.

**3. Looking before writing.** This part is yours, because no crate can police your flash command. Read the image's
lowest load address first and refuse anything below `0x08006000`:

```bash
readelf -lW image.elf | awk '/LOAD/ {print $3}'      # every paddr must be >= 0x08006000
```

A script that does this for you, and refuses, is in the examples repository:
<https://github.com/bitscrafts/efr32mg2x-examples> (`elf-load-range.py`).
