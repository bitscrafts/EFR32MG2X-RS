# Build status after the migration — 2026-11-10

Measured in the container, `cargo build --release` unless noted, Rust 1.99.1,
target `thumbv8m.main-none-eabihf`, `gcc-arm-none-eabi` 14.2 installed.

| crate | build | note |
|---|---|---|
| `efr32mg24-pac` | OK | |
| `efr32mg24-hal` (`--features rt`) | OK | |
| `efr32mg24-rail-sys` | OK | |
| `xiao-mg24-bsp` (lib) | OK | |
| `efr32mg24-bluetooth-sys` | **FAIL** | 17 × `E0609 no field ... on type &__BindgenUnionField<...>` |
| `xiao-mg24-bsp --features ble` | **FAIL** | inherits the bluetooth-sys failure |

Reproduce: `scripts/build-crates.sh`.

## bluetooth-sys: bindgen union codegen drift

The crate's Rust code reads event payloads as `evt.data.connection`,
`evt.data.value`, `evt.data.handle` … but the bindings now generate

```rust
pub struct sl_bt_evt_connection_opened_s__bindgen_ty_1 {
    pub connection: __BindgenUnionField<u8>,
    ...
}
```

i.e. bindgen no longer flattens the C union that the event structs carry, so
every field access through `data` fails. The code was written against a
different bindgen output shape; the crate does not pin bindgen.

Fix directions, cheapest first:

1. pin the `bindgen` version in `build.rs` / `Cargo.toml` to the one the code
   was written against;
2. or keep the current bindgen and read through the union accessor
   (`unsafe { data.connection.connection }`), which is mechanical but touches
   every BLE event site;
3. or generate with `--wrap-unsafe-ops`/`--merge-extern-blocks` variants and see
   whether the layout matches the code.

This is BLE-phase work (D2/D3), not part of the PAC ladder, so it is recorded
rather than fixed here.

The build also needs the MSLA-licensed Gecko headers, which are gitignored
(`**/vendor/**/*.h`). They were restored from the archived pre-migration tree
into `crates/efr32mg24-bluetooth-sys/vendor/bluetooth/` (local only, never
committed). Without them bindgen emits an empty module and the crate fails with
`cannot find type sl_bt_msg_t`.

## BSP examples: API drift

Two of the BSP examples no longer match `Board::split()`, which returns
`(Pins, FrozenClocks, cortex_m::Peripherals, BoardPeripherals)`:

- `00_blink.rs` destructures three values;
- `00_minimal_blink.rs` uses `board.led` / `board.delay`, fields that do not
  exist on `Board`.

Both are one-line fixes but they belong to the BSP phase, after the PAC
examples run on hardware.

## PAC examples: 13 of 14 build

`blink_pac` fails on the svd2rust 0.37 API change (`w.bits()` now takes an
argument). `scripts/build-pac-examples.sh` lists the rest as `OK` and keeps the
logs under `_tmp/build-logs/`.
