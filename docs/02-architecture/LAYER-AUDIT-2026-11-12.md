# Layer audit — 2026-11-12

A detailed pass over every crate looking for the same class of defect: **a layer that hands the
layer above it something it should not have**, or **the same thing implemented twice**. Six were
found. All six compiled and every test passed, which is the point — the compiler cannot see a
layer boundary, so a boundary that is not a check is not a boundary.

## The rule, stated once

```text
efr32mg24-pac    registers, generated
     ↑ only the HAL may hold these
efr32mg24-hal    drivers + pin primitives; no board wiring
     ↑ only a BSP may hold these
xiao-*-bsp       which device, which pin, which bus, which power gate
     ↑ only an application may hold these
application      reads and writes; never sees an address or a pin
```

A layer calls the one below **internally** and its public API never names the layer below's types.
The tests of that statement are in `scripts/gate.sh`, under *layer boundaries*.

## What was found

### 1. The HAL's public API required the caller to own the PAC

Evidence: `Buzzer::new(dp: &pac::Peripherals)`, `Timer0::take(dp: &pac::Peripherals)`,
`Microphone::new(&pac::iadc0_s::RegisterBlock)`, `BitBangI2c::on(&pac::gpio_s::RegisterBlock)`,
`pin::Output::new(&pac::gpio_s::RegisterBlock)`, `board::gpio_static()`.

Consequence, measured: the microphone example could not be written without
`unsafe { efr32mg24_pac::Peripherals::steal() }` — a HAL that makes its own example steal the
singleton has failed to hand out what it already owns.

Fix: every one of those constructors became `pub(crate)`, and the two doors a caller is meant to
use were added:

```rust
let board = Board::take()?;                 // owns the PAC once, for the program's life
let mut led = board.pwm_on(Port::A, 7, 1_000)?;   // PWM, no PAC in sight
let mut mic = board.converter().microphone()?;    // converter is a handle, not a register block
let mut bus = board.i2c_on(Bus::new(Port::C, 1 << 4, 1 << 5));
```

`microphone::Converter` is the pattern: an opaque `Copy` handle with a constructor, so a driver can
be started from what the board hands out.

### 2. `xiao-mg24-bsp` exported the PAC, and two facades

Evidence: `pub use efr32mg24_pac as pac;` plus `pub ldma_s: pac::LdmaS` on a public struct, and
`&(*pac::Iadc0S::ptr())` inside `sensors/microphone.rs`. Alongside it, `Module` was the facade that
the carrier crate actually used — so every example proved one API and nothing exercised the other.

Fix: the older cluster (`board.rs`, `pins.rs`, `sensors/`, `radio/`) retired to
`_tmp/efr32mg24-rs/retired-bsp-cluster/` with a README; `lib.rs` now exports `Module` and
re-exports the HAL only. The PAC re-export is gone.

### 3. `xiao-expansion-base-bsp` re-exported the PAC

Same defect one level up: an application holding it can reconfigure what the board claimed to own.
Removed.

### 4. `Board::gpio()` and `board::gpio_static()` were public

Both handed out a register block. Now `pub(crate)`; the one public door for a facade that must own a
bus is `board::i2c_static(bus)`, which exists because a struct cannot borrow a bus from the board it
also owns.

### 5. Two GPIO implementations in the HAL

`pin.rs` (189 lines: `Port`, bit, polarity, `write_dout`, `read_din`, `set_mode`) and `gpio/`
(605 lines: type-state pins, `Pins`, `GpioExt`). The type-state layer's only consumer was the
retired BSP cluster, and it was never proven on this part — and its README documented the LED as
`PB2` active-high, when it is `PA7` active-low.

Fix: `gpio/` retired to `_tmp/efr32mg24-rs/retired-hal-gpio/`. `pin.rs` is the single pin
implementation, and the base examples that name pins use it.

### 6. Documentation that outlived the code

`xiao-mg24-bsp/src/lib.rs` claimed *78 MHz* (measured: 19 MHz SYSCLK), *256 KB RAM* (the linker
script reserves the mailbox, so 252K is usable), and a Quick Start calling `board.pins.d0`, an API
that no longer existed. Two READMEs in the HAL asserted the LED was `PB2` and the IMU bus was
`SCL=PB2, SDA=PB3` — the vendor's order, which this board reverses.

Fix: `lib.rs` rewritten; both READMEs corrected with their provenance; both modules retired.

### 7. Three examples stole a core peripheral

`cortex_m::Peripherals::steal().SYST` for a delay, and `steal()` plus two trace-register writes for
the cycle counter. Same defect one layer down: SysTick and DWT have an owner (the board), and an
example that steals them has no owner — two owners of SysTick is a delay that lies, and a cycle
counter that was never enabled reads as zero, which looks like a very fast loop.

Fix: `Board::take()` now takes `cortex_m::Peripherals` too and `board.delay()` hands out a `Delay`;
a new `hal::dwt` module (`start`, `cycles`, `since`, `cycles_to_us`) replaced the three hand-rolled
cycle-counter helpers. No example steals anything.

## What is now checked, every run

```text
PASS  examples outside the PAC crate name no PAC type
PASS  no BSP re-exports the PAC
PASS  the HAL exposes no register block publicly
PASS  no example steals a core peripheral
PASS  <crate>: N examples of its own level      (every crate, not only the BSPs)
PASS  motion-detection: no hardware dependency
```

Each of those failed at least once during this pass, in this workspace, while everything compiled.

## What is deliberately left alone

- **Peripheral-token constructors** (`Usart::new(pac::Usart0S, …)`, `Emu::new(pac::EmuS)`,
  `Sysrtc::new(pac::Sysrtc0Ns)`). Taking the singleton *token* is the idiom of every Rust HAL
  (`dp.USART0` in stm32, `peripherals.USART0` in rp2040): it is how exclusivity is proven at compile
  time. It is a different thing from handing out a *register block*, which is what rule 3 forbids.
  These modules are chip-level and no board uses them yet; they are unproven, and unproven is not
  the same as wrong.
- **The carrier crate entering through the HAL's `Board` rather than through `Module`.** The arrow
  base → module → hal is respected in the manifests, but `expansion_base.rs` constructs
  `efr32mg24_hal::board::Board` directly. It is a candidate for a later pass: `Module` would need to
  hand out the chip-level board, and the gain is one fewer name in the carrier.

## The lesson worth keeping

Every one of the six was invisible to `cargo build`, `cargo test` and `cargo clippy`. What found
them was reading the *public signatures* of each crate and asking, for each one, *what does this
force the caller to own?* That question is cheap; the answer is the architecture.
