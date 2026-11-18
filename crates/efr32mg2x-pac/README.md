# efr32mg2x-pac

[![crates.io](https://img.shields.io/crates/v/efr32mg2x-pac.svg)](https://crates.io/crates/efr32mg2x-pac)
[![docs.rs](https://docs.rs/efr32mg2x-pac/badge.svg)](https://docs.rs/efr32mg2x-pac)
[![CI](https://github.com/bitscrafts/EFR32MG2X-RS/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/bitscrafts/EFR32MG2X-RS/actions/workflows/build-and-test.yml)
[![license](https://img.shields.io/crates/l/efr32mg2x-pac.svg)](#license)

Peripheral Access Crate (PAC) for the Silicon Labs EFR32MG24, generated from the vendor SVD.

## Overview

Low-level register access to every peripheral of one part, the **EFR32MG24B220F1536IM48** — the chip on the XIAO MG24
Sense. Generated from Silicon Labs' CMSIS-SVD for that part with svd2rust; the register map, the field types and the doc
comments come from the SVD, not from a header file somebody transcribed.

**Generated**: 2,633 files across 104 directories, from `EFR32MG24B220F1536IM48.svd`. 103 peripheral instances in 46 families.


**Runnable examples**: <https://github.com/bitscrafts/efr32mg2x-examples> — a clone, a build and a guarded flash, one
crate per layer (PAC, HAL, radio, board, Expansion Base). They are not published to crates.io: an examples-only crate has
no binary, so it installs only with `--example`, and its crate page would render nothing.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
efr32mg2x-pac = "0.1.0"
# the architecture crate supplies the critical-section implementation that the generated
# modify() needs; without this feature the build fails to link with
# "undefined symbol: _critical_section_1_0_acquire"
cortex-m = { version = "0.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7"
```

The smallest program that runs on the board — copied from `examples/pac_minimal.rs`, which the gate compiles:

```rust,ignore
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg2x_pac as pac;
use panic_halt as _;

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7, 0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],           // 8-byte header (02 01 02 02 02 03 02 04 from Arduino)
    magic: [u8; 16],           // Magic signature (Word 13 points HERE, not to header)
    struct_version: u32,       // Version of this struct
    signature_type: u32,       // Type of signature (0 = none)
    signature_location: u32,   // Pointer to signature (0xFFFFFFFF = none)
    app_info: ApplicationData, // Application metadata
    cert: u32,                 // Pointer to certificate (NULL = 0)
    long_token_section_address: u32, // Pointer to long token section (NULL = 0)
    decrypt_key: [u8; 16],     // Decryption key (zeros if not encrypted)
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32, // Arduino uses 0x22 (34)
    version: u32,
    capabilities: u32,
    product_id: [u8; 16],
}

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    header: [0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04],
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0201,
    signature_type: 0,
    signature_location: 0xFFFF_FFFF,
    app_info: ApplicationData {
        app_type: 0x22,
        version: 0x0000_0001,
        capabilities: 0,
        product_id: [0u8; 16],
    },
    cert: 0,
    long_token_section_address: 0,
    decrypt_key: [0u8; 16],
};

#[entry]
fn main() -> ! {
    // One line, no module, no dependency: the diagnostic slot belongs to the previous binary until something says
    // otherwise, and a reader would take its record as this program's own. Measured on 2026-11-15: without this,
    // `pac_minimal` was reported as "mic rst=ok rms=504" -- the microphone example that had run before it.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let dp = pac::Peripherals::take().unwrap();

    // 1. The GPIO clock, gated by bit 26 of CLKEN0 in the secure CMU.
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());

    // 2. PA7 is the LED (active low). MODEL holds the pins 0-7 mode fields; mode7() is this one.
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());

    // 3. Toggle it. No DOUTTGL on this part: XOR the bit in software.
    loop {
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });

        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}
```


## Features

| feature | what it turns on | default |
|---|---|---|
| `mg24` | the device this crate is generated for | yes |
| `rt` | `cortex-m-rt/device` | no |
| `critical-section` | `dep:critical-section` | yes |


## Peripherals

103 instances on `Peripherals`, in 46 families. Generated from `src/lib.rs`: every name below is a field you can reach through `Peripherals`.

| family | instances |
|---|---|
| acmp | `acmp0_ns`, `acmp0_s`, `acmp1_ns`, `acmp1_s` |
| buram | `buram_ns`, `buram_s` |
| burtc | `burtc_ns`, `burtc_s` |
| cmu | `cmu_ns`, `cmu_s` |
| dcdc | `dcdc_ns`, `dcdc_s` |
| devinfo | `devinfo` |
| dmem | `dmem_ns`, `dmem_s` |
| dpll | `dpll0_ns`, `dpll0_s` |
| emu | `emu_ns`, `emu_s` |
| eusart | `eusart0_ns`, `eusart0_s`, `eusart1_ns`, `eusart1_s` |
| fsrco | `fsrco_ns`, `fsrco_s` |
| gpcrc | `gpcrc_ns`, `gpcrc_s` |
| gpio | `gpio_ns`, `gpio_s` |
| hfrco | `hfrco0_ns`, `hfrco0_s` |
| hfrcoem | `hfrcoem23_ns`, `hfrcoem23_s` |
| hfxo | `hfxo0_ns`, `hfxo0_s` |
| hostmailbox | `hostmailbox_ns`, `hostmailbox_s` |
| i2c | `i2c0_ns`, `i2c0_s`, `i2c1_ns`, `i2c1_s` |
| iadc | `iadc0_ns`, `iadc0_s` |
| icache | `icache0_ns`, `icache0_s` |
| keyscan | `keyscan_ns`, `keyscan_s` |
| ldma | `ldma_ns`, `ldma_s` |
| ldmaxbar | `ldmaxbar_ns`, `ldmaxbar_s` |
| letimer | `letimer0_ns`, `letimer0_s` |
| lfrco | `lfrco_ns`, `lfrco_s` |
| lfxo | `lfxo_ns`, `lfxo_s` |
| msc | `msc_ns`, `msc_s` |
| mvp | `mvp_ns`, `mvp_s` |
| pcnt | `pcnt0_ns`, `pcnt0_s` |
| prs | `prs_ns`, `prs_s` |
| radioaes | `radioaes_ns`, `radioaes_s` |
| scratchpad | `scratchpad_ns`, `scratchpad_s` |
| semailbox_ns_host | `semailbox_ns_host` |
| semailbox_s_host | `semailbox_s_host` |
| smu | `smu_ns`, `smu_s` |
| smu_ns_cfgns | `smu_ns_cfgns` |
| smu_s_cfgns | `smu_s_cfgns` |
| syscfg | `syscfg_ns`, `syscfg_s` |
| syscfg_ns_cfgns | `syscfg_ns_cfgns` |
| syscfg_s_cfgns | `syscfg_s_cfgns` |
| sysrtc | `sysrtc0_ns`, `sysrtc0_s` |
| timer | `timer0_ns`, `timer0_s`, `timer1_ns`, `timer1_s`, `timer2_ns`, `timer2_s`, `timer3_ns`, `timer3_s`, `timer4_ns`, `timer4_s` |
| ulfrco | `ulfrco_ns`, `ulfrco_s` |
| usart | `usart0_ns`, `usart0_s` |
| vdac | `vdac0_ns`, `vdac0_s`, `vdac1_ns`, `vdac1_s` |
| wdog | `wdog0_ns`, `wdog0_s`, `wdog1_ns`, `wdog1_s` |


## Memory Map

Taken from `memory.x`, which is what the linker actually uses.

| region | start | end | size |
|---|---|---|---|
| Flash (application) | `0x08006000` | `0x0817FFFF` | 1512 KB |
| RAM | `0x20000000` | `0x2003EFFF` | 252 KB |
| Gecko bootloader (never written) | `0x08000000` | `0x08005FFF` | 24 KB |

Peripheral space: secure `0x4000_0000-0x4FFF_FFFF`, non-secure `0x5000_0000-0x5FFF_FFFF`; every peripheral with a TrustZone face appears twice, as `<name>_s` and `<name>_ns`.


## Module Structure

Generated by `svd2rust` and split one register per file. Counted in this tree by the generator that produced this section:

- **2,633 files** under `src/`, in **104** directories
- **103** peripheral instances in **46** families; each with a secure and a non-secure face where TrustZone applies (`<name>_s`, `<name>_ns`)
- the largest register blocks: `gpio_ns/` 173 modules, `gpio_s/` 173 modules, `prs_ns/` 115 modules
- `lib.rs` is 1,756 lines (53,990 B): the `Peripherals` struct and the base addresses

**There is no `mod.rs` inside a peripheral directory.** `svd2rust` writes the register block as `<periph>.rs` and puts the register modules in the directory beside it; that file declares them. An earlier version of this page showed a `mod.rs` that does not exist.


## Generation Process

```bash
# 1. device crate from the SVD
svd2rust -i svd/EFR32MG24B220F1536IM48.svd --target cortex-m --atomics

# 2. split the monolithic lib.rs into one module per register
form -i lib.rs -o src/

# 3. format and build
cargo fmt
cargo build --release --target thumbv8m.main-none-eabihf
```

### This crate is one part

`efr32mg2x-pac` is generated from `svd/EFR32MG24B220F1536IM48.svd` and exposes that part's registers. Another part means
another device feature and its own SVD, not a differently-named package:

```toml
[features]
mg24 = []                                  # EFR32MG24B220F1536IM48, the part this crate ships
default = ["mg24", "critical-section"]     # so the ordinary dependency line needs no ceremony
```

Turning default features off without naming a device fails to compile, deliberately: a register map without a device is
not a register map.

## Dependencies

| crate | version |
|---|---|
| `cortex-m` | `0.7.7` |
| `cortex-m-rt` | `0.7.3` (optional) |
| `critical-section` | `1.2.0` (optional) |
| `portable-atomic` | `1.10.0` |
| `vcell` | `0.1.3` |


## Target Architecture

- **Target triple**: `thumbv8m.main-none-eabihf`
- **CPU**: ARM Cortex-M33
- **FPU**: single precision, hardware
- **DSP**: enabled
- **TrustZone**: supported; peripherals appear twice, as `<name>_s` and `<name>_ns`

## Linker Script

Shipped with the crate as `memory.x`; `build.rs` copies it to `OUT_DIR` so `-Tlink.x` can include it. These are the numbers in the file, not a plan:

```ld
MEMORY
{
  /* Application flash - starts after 24 KB Gecko bootloader */
  /* Bootloader occupies 0x08000000 - 0x08005FFF (first 24 KB) */
  /* MUST match Arduino's bootloader layout for compatibility */
  FLASH : ORIGIN = 0x08006000, LENGTH = 1512K
  /* SRAM - 256 KB total, but the last 4 KB page is reserved as the SWD mailbox
   * (0x2003_F000), so the linker gets 252 KB. */
  RAM : ORIGIN = 0x20000000, LENGTH = 252K
}
_stack_start = 0x2003F000;
```

The first 24 KiB (`0x08000000`-`0x08005FFF`) belong to the Gecko bootloader; an image there bricks the board. Build with `-C link-arg=--nmagic` as well, or the linker emits a segment at `0x08000000` regardless of this script.


## Safety

This crate provides `unsafe` access to hardware registers. The caller is responsible for:

- no data races — use `critical-section` or an RTOS
- configuring a peripheral before touching it
- enabling the clock or gate that the peripheral needs, in the CMU that owns it (secure or non-secure)
- respecting the TrustZone boundary: a write through the wrong alias faults at run time

## Development

### Building from source

```bash
git clone https://github.com/bitscrafts/EFR32MG2X-RS
cd EFR32MG2X-RS/crates/efr32mg2x-pac
rustup target add thumbv8m.main-none-eabihf
cargo build --release --target thumbv8m.main-none-eabihf
```

### Testing

The examples are the test: each one that carries an instrument writes what it measured into the mailbox and reports it
there, and the gate builds all of them. `examples/pac_minimal.rs` is the exception and deliberately so — it is the
smallest program that boots, with no helper and no mailbox. `docs/examples.md` is generated from the files themselves
and carries the date each was watched on the board, or `--` when it has only been compiled.

## Related Crates

| crate | what it adds |
|---|---|
| `efr32mg2x-bluetooth-sys` | Low-level FFI bindings to Silicon Labs Bluetooth stack (BGAPI) for EFR32MG24 |
| `efr32mg2x-bootloader` | in this workspace |
| `efr32mg2x-bootloader-test-app` | in this workspace |
| `efr32mg2x-cmsis-stubs` | CMSIS core helpers the Silicon Labs radio blobs expect the application to provide |
| `efr32mg2x-emlib-sys` | The three emlib units the EFR32MG24 radio blobs link against (Silicon Labs, Zlib) |
| `efr32mg2x-expansion-base-bsp` | Board support for the Seeed XIAO Expansion Base when it carries a XIAO MG24 module |
| `efr32mg2x-hal` | Hardware Abstraction Layer for EFR32MG24 microcontrollers |
| `efr32mg2x-radio` | Safe radio layer for EFR32MG24: Bluetooth (BGAPI) and RAIL, with the buffers and state machines the vendor API assumes |
| `efr32mg2x-rail-sys` | Low-level FFI bindings to Silicon Labs RAIL (Radio Abstraction Interface Layer) for EFR32MG24 |
| `efr32mg2x-xiao-bsp` | Board Support Package for Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) |

Runnable examples for all of them: <https://github.com/bitscrafts/efr32mg2x-examples>.


## Resources

- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/)
- [SVD Specification](https://www.keil.com/pack/doc/CMSIS/SVD/html/)
- [svd2rust documentation](https://docs.rs/svd2rust/)

## Staying out of the bootloader

The first 24 KiB of flash (`0x08000000`-`0x08005FFF`) hold the Gecko bootloader. An image that lands there bricks the
board. The linker script in this crate starts at **0x08006000**, and the flags are not optional either:

```toml
[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "link-arg=-Tlink.x", "-C", "link-arg=--nmagic"]
```

Without `--nmagic` the linker emits a small segment at `0x08000000` and some flash tools will happily write it. Read the
lowest load address of the image before flashing and refuse anything below `0x08006000`; `elf-load-range.py` in the
examples repository does exactly that and returns non-zero.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://www.apache.org/licenses/LICENSE-2.0.txt))
- MIT license ([LICENSE-MIT](https://opensource.org/license/mit))

at your option.
