# efr32mg2x-pac

Peripheral Access Crate (PAC) for Silicon Labs EFR32MG24 microcontrollers.

## Overview

Low-level register access to every peripheral of one part, the **EFR32MG24B220F1536IM48** — the chip on the XIAO MG24
Sense. Generated from Silicon Labs' CMSIS-SVD for that part with svd2rust; the register map, the field types and the
doc comments come from the SVD, not from a header file somebody transcribed.

**Generated**: 2,633 files across 104 directories, from `EFR32MG24B220F1536IM48.svd`. One part, one crate: an
earlier version of this file advertised 20 device variants that do not exist.

**Runnable examples**: <https://github.com/bitscrafts/efr32mg2x-examples> — a clone, a build and a guarded
flash, one crate per layer (PAC, HAL, radio, board, Expansion Base). Not published to crates.io: an examples-only
crate has no binary, so it installs only with `--example`, and its crate page would render nothing.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
efr32mg2x-pac = "0.1.3"
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
use efr32mg24_pac as pac;
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
| `rt` | `cortex-m-rt/device` | no |
| `critical-section` | `dep:critical-section` | yes |
<!-- /gen:features -->

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



## Peripherals

103 instances on `Peripherals`, in 46 families. Generated from `src/lib.rs`; every name here
is a field you can call `.take()` on.

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
<!-- /gen:peripherals -->

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
- `lib.rs` is 1,752 lines (53,678 B): the `Peripherals` struct and the base addresses

**There is no `mod.rs` inside a peripheral directory.** `svd2rust` writes the register block as `<periph>.rs` and puts the register modules in the directory beside it; that file declares them. An earlier version of this page showed a `mod.rs` that does not exist.



## Generation Process

The PAC in this tree was generated once, before this repository's current
history, with `svd2rust` 0.37.1 (`--locked`) from
`svd/EFR32MG24B220F1536IM48.svd`, then split one register per file with
`form`. The commands below are the shape of that recipe as recorded at the
time; they were **not** re-run on 2026-11-10, and the flag spelling should be
checked against the installed `svd2rust` version before anyone relies on them:

```bash
cd crates/efr32mg24-pac

# 1. device crate from the SVD
svd2rust -i svd/EFR32MG24B220F1536IM48.svd --target cortex-m --atomics

# 2. split the monolithic lib.rs into one module per register
form -i lib.rs -o src/

# 3. format and build
cargo fmt
cargo build --release --target thumbv8m.main-none-eabihf
```

`build.rs` then copies `memory.x` and `device.x` into `OUT_DIR` and adds that
directory to the linker search path, which is what makes `-Tlink.x` resolve the
memory layout at link time.

### This crate is one part

`efr32mg2x-pac` is generated from `svd/EFR32MG24B220F1536IM48.svd` and exposes that part's registers. It is not a multi-variant crate:
an earlier version of this page showed a `--features` list of 20 device variants and an example regeneration command
for `EFR32MG24A020F1536GM48`, neither of which exists here.

Another part means another crate, generated the same way:

```bash
svd2rust -i svd/<part>.svd --output-dir src/ --atomics --target cortex-m --generic-mod
cargo fmt
```

The rest of this document describes the layout such a crate gets.

## Target Architecture

- **Target Triple**: `thumbv8m.main-none-eabihf`
- **CPU**: ARM Cortex-M33
- **FPU**: Single-precision (hardware)
- **DSP**: Enabled
- **TrustZone**: Supported

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

This crate provides `unsafe` access to hardware registers. Users must ensure:
- No data races (use `critical-section` or RTIC)
- Correct peripheral configuration
- Proper clock/power management before peripheral access
- Respect TrustZone security boundaries

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/bitscrafts/EFR32MG2X-RS
cd EFR32MG2X-RS/crates/efr32mg24-pac

# Install target
rustup target add thumbv8m.main-none-eabihf

# Build
cargo build --target thumbv8m.main-none-eabihf
```

### Testing

PAC testing is limited without hardware:
- Compile tests verify code generation
- Hardware-in-loop tests require dev kit
- Register access tests can use QEMU (if supported)

## Related Crates

| crate | what it adds |
|---|---|
| `efr32mg2x-bluetooth-sys` | in this workspace |
| `efr32mg2x-bootloader` | in this workspace |
| `efr32mg2x-bootloader-test-app` | in this workspace |
| `efr32mg2x-cmsis-stubs` | in this workspace |
| `efr32mg2x-emlib-sys` | in this workspace |
| `efr32mg2x-expansion-base-bsp` | in this workspace |
| `efr32mg2x-hal` | in this workspace |
| `efr32mg2x-pac` | in this workspace |
| `efr32mg2x-radio` | in this workspace |
| `efr32mg2x-rail-sys` | in this workspace |
| `efr32mg2x-xiao-bsp` | in this workspace |

Runnable examples for all of them: <https://github.com/bitscrafts/efr32mg2x-examples>.



## Resources

- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/)
- [SVD Specification](https://www.keil.com/pack/doc/CMSIS/SVD/html/)
- [svd2rust Documentation](https://docs.rs/svd2rust/)

## Staying out of the bootloader

The first 24 KiB of flash (0x08000000-0x08005FFF) hold the Gecko bootloader. An image that lands there bricks the
board. The linker script in this crate starts at **0x08006000**, and the flags are not optional either:

```toml
[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "link-arg=-Tlink.x", "-C", "link-arg=--nmagic"]
```

Without `--nmagic` the linker emits a small segment at 0x08000000 and some flash tools will happily write it. Read the
lowest load address of the image before flashing and refuse anything below 0x08006000.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://www.apache.org/licenses/LICENSE-2.0.txt))
- MIT license ([LICENSE-MIT](https://opensource.org/license/mit))

at your option.

---

**Status**: ✅ Generated - Modular Structure Complete
**Version**: 0.1.3
**Device**: EFR32MG24B220F1536IM48 (XIAO MG24 Sense)
**Generated With**: svd2rust 0.37.1 + form
**Files**: 2,633 Rust files across 104 modules
**Last Updated**: December 4, 2025
