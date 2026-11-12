# Where does which code live? — how four Rust embedded ecosystems split PAC / HAL / BSP

Researched 2026-11-12, from the repositories themselves, to decide how this workspace should
separate the **XIAO MG24 module** from the **XIAO Expansion Base**, and where domain code
belongs. Every claim below was read from GitHub (repository listings, `Cargo.toml` files and
crate documentation) rather than recalled; the source is named for each.

## The question

This workspace had grown into one story: `efr32mg24-hal` contained a `board` module with the
Expansion Base's pins, `xiao-mg24-bsp` contained a facade for the same base, and examples
that need only the module sat beside examples that need both boards. Three specific questions
came out of that:

1. Should the Expansion Base have its own crate, or be examples?
2. Is the module's **Sense** variant a crate, a feature, or nothing?
3. Where does domain code with no hardware in it (`motion-detection`) belong?

## What the ecosystems do

### Raspberry Pi Pico — `rp-rs/rp-hal` + `rp-rs/rp-hal-boards`

```text
rp-hal/            rp2040-hal, rp235x-hal, rp-hal-common, *-macros,
                   rp2040-hal-examples, rp235x-hal-examples   (examples as their own crates)
rp-hal-boards/     boards/<board>/           one crate per board:
                   adafruit-feather-rp2040, adafruit-kb2040, arduino_nano_connect,
                   pimoroni-pico-explorer, ... (13 boards listed)
```

`boards/rp-pico/` holds `Cargo.toml`, `build.rs`, `src/`, `examples/` — a real crate. Its own
documentation says it is *"a Hardware Abstraction Layer for the Raspberry Pi Pico … re-exports
the rp2040_hal crate"*, and its example imports `rp_pico::entry` and `rp_pico::hal`.

- **Separation:** HAL per chip, **BSP per board**, BSPs in a repository of their own.
- **Examples:** each BSP carries its own; the HAL's examples live in separate crates.
- **Wiring:** inside the BSP crate, once.

### nRF — `nrf-rs/nrf-hal`

```text
nrf52840-hal, nrf52832-hal, nrf52805-hal … nrf9160-hal,
nrf5340-app-hal, nrf5340-net-hal, nrf-hal-common, examples/, xtask/
```

One HAL crate **per chip**, plus a shared `nrf-hal-common`. There are **no BSP crates**: the
examples at the repository root select a board through cargo features, and the repository
ships a `Cargo.example.toml` template whose comment explains why the pieces cannot all live in
one workspace:

> *"Since Cargo resolves all features for all crates in the workspace together, it is not
> possible to have a workspace with all the crates together, since they enable incompatible
> features. Instead, we provide this template so you can enable only the crates you're going to
> work on."*

- **Separation:** chip granularity only; the board is a set of features.
- **Examples:** in the HAL repository, one directory per demo, `exclude = ["examples"]`.
- **Wiring:** in the examples, selected by `cfg`.

### STM32 — `stm32-rs/stm32f4xx-hal`

```text
stm32f4xx-hal/     src/{adc,can,crc32,dac,dma,dsi,dwt,flash,fmc,fmpi2c,gpio,i2c,i2s,ltdc,…}.rs
                   examples/  memory.x  build.rs
```

One HAL **per family**, one module per peripheral, examples inside the crate, and **no board
features at all** in `Cargo.toml` — the grep for `board`/`disc` returns nothing. The board is
whatever the example says it is: pin numbers appear inline.

- **Separation:** none below the family; there is no BSP layer.
- **Examples:** in the HAL crate, each one wiring its own pins.
- **Wiring:** per example. Fine when the point is to teach the peripheral.

### ESP32 — `esp-rs/esp-hal`

```text
esp-hal, esp-alloc, esp-backtrace, esp-config, esp-hal-procmacros, esp-radio,
esp-storage, esp-rtos, esp-println, esp-phy, esp-lp-hal, esp-rom-sys, esp-sync,
esp-bootloader-direct-boot, esp-bootloader-esp-idf, xtensa-lx-rt, esp-riscv-rt,
hil-test, compile-tests, examples/, xtask/, template/
```

A monorepo of roughly thirty crates. The chip is chosen by feature, not by crate name. There
are **no BSP crates**: `examples/` is grouped by capability (`async`, `ble`, `esp-now`,
`peripheral`, `qa`, `selfie_cam`), and board wiring appears inside the examples.

- **Separation:** HAL and its supporting infrastructure, chip selected by feature.
- **Examples:** all in one place, grouped by what they demonstrate.
- **Wiring:** per example.

### Counterpoint — `embassy-rs/embassy`

`embassy-nrf`, `embassy-rp`, `embassy-stm32`: HAL only, chip by feature, no BSP crates,
examples grouped per chip/board directory. Same shape as `esp-hal`.

## The comparison

| | chip selection | BSP crates | where examples live | where the wiring is written |
|---|---|---|---|---|
| **rp-hal** | one HAL per chip | **yes, one per board** (`boards/`, separate repo) | in each BSP, plus separate example crates | in the BSP, once |
| **nrf-hal** | one HAL per chip | no | in the HAL repo, board by feature | in the examples |
| **stm32f4xx-hal** | one HAL per family | no | in the HAL crate | in the examples |
| **esp-hal** | one crate, chip by feature | no | one `examples/` grouped by capability | in the examples |
| **this workspace (decided)** | one HAL (one chip exists) | **yes, two**: module, and Expansion Base | per crate, by the minimum hardware required | in the BSP, once |

Two ecosystem facts are worth carrying over beyond the table:

- **`rp-hal` puts BSPs in a different repository from the HAL.** That is stronger than a
  folder: the HAL's CI cannot depend on a board, because the board is not there.
- **`nrf-hal`'s workspace note** is the reason not to solve this with features alone when the
  parts are *alternative* rather than *additive*: cargo unifies features across a workspace,
  so "the module without the base" and "the module with the base" in one crate can silently
  become the second when anything builds the first.

## What this workspace needs that none of them have

The board family here is one chip, two physical boards that **stack**, and one variant that is
the *same PCB with more parts fitted*:

```text
XIAO MG24                 LED (PA7)                                   both variants
XIAO MG24 Sense           + LSM6DS3TR-C at 0x6A on PB2/PB3, gate PD5
                          + analog MEMS microphone, signal PC9, enable PC8, IADC0
XIAO Expansion Base       OLED 0x3C, BME68x 0x76, RTC PCF8563 0x51,
   (stacks under it)     buzzer PC03 on TIMER0 CC0, user button PC01,
                          Grove UART/A0/I2C, microSD
```

None of the four ecosystems has that shape, so the mapping is by analogy, not by copy:

- **Expansion Base → its own crate**, as `rp-hal-boards` does per board. It is an optional
  piece of hardware that a user may not own, and its API must not exist for someone who does
  not.
- **Sense → a feature of the module crate**, not a crate. `rp-hal` has no such case because
  every board in its list is a different PCB. A crate would duplicate the LED, the board init
  and the clock setup to add two sensors; a feature says exactly what is true — same board,
  more parts.
- **Wire removal from the HAL** goes further than any of the four (all of them keep some
  board knowledge in examples). The reason is specific and written down elsewhere: on this
  board the IMU bus is `SDA = PB2`, `SCL = PB3`, the opposite of what the vendor's pin map
  declares, and the on-chip I2C0 cannot complete a read. Both facts cost hours to establish.
  A HAL that knows them would make every new board a fork of the HAL; an example that
  re-derives them is how the same investigation gets repeated. They belong in a BSP, once.

## The decision

```text
crates/                              the hardware stack: one dependency direction
  efr32mg24-pac/                     registers
  efr32mg24-hal/                     drivers + a bit-banged I2C that takes pins; no wiring
  xiao-mg24-bsp/                     the module: LED; feature "sense": IMU + microphone
  xiao-expansion-base-bsp/           the base: OLED, BME68x, RTC, buzzer, button, Grove, SD
  efr32mg24-bluetooth-sys/           bindings
  efr32mg24-rail-sys/
  (bootloader crates, excluded from the workspace)

libraries/                           domain policy: no hardware, host-testable
  motion-detection/                  [i32;3] + dt_us -> Event
```

Rules that the gate enforces rather than states (`scripts/gate.sh`):

1. **Nothing in `libraries/` may depend on the hardware stack** — `cargo tree` must contain
   no `efr32mg24-*`. One HAL dependency turns `cargo test` into a cross-compilation, and the
   tests stop being written.
2. **Every crate with examples must declare the example in its manifest**, and the example
   directory must be the one the manifest names (`examples/fall_detector/` is built as
   `fall-detector`).
3. **A crate that exposes hardware features and has zero examples fails the gate**: a crate
   whose API nothing builds is a crate whose API is not tested.

And a rule for the examples themselves, which no ecosystem above needed because none of them
ships a board that stacks:

> An example lives in the crate of the **largest** hardware requirement it has, and states
> that requirement in its first comment. `xiao-mg24-bsp/examples/` is code that runs with the
> module on the desk; `xiao-expansion-base-bsp/examples/` is code that needs the base too.

## What this costs

- Two BSP crates instead of one: more manifests, and a user with the base must depend on both
  (one-way: base → module).
- `motion-detection` moved out of `crates/`, so its path changed in the HAL's dev-dependency
  and in three documents.
- Some duplication is now *structurally* impossible rather than merely discouraged — at the
  price of the module crate having to grow a `sense` feature instead of just adding methods.

## Sources

- `github.com/rp-rs/rp-hal` — top-level crates; `github.com/rp-rs/rp-hal-boards` — `boards/`,
  `boards/rp-pico/{Cargo.toml,src/lib.rs,examples/}`.
- `github.com/nrf-rs/nrf-hal` — per-chip HAL crates, `examples/`, `Cargo.example.toml`.
- `github.com/stm32-rs/stm32f4xx-hal` — `src/` module list, `Cargo.toml` without board features,
  `examples/`.
- `github.com/esp-rs/esp-hal` — monorepo crate list, `examples/` grouped by capability.
- `github.com/embassy-rs/embassy` — `embassy-nrf`, `embassy-rp`, `embassy-stm32`.
