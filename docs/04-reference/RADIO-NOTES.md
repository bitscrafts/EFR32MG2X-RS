# Radio notes — 2026-11-12

What it took to make the two vendor stacks compile and link from this repository, what is proven,
and what is not. Written for the next person who assumes "the blob is there, so it works".

## What exists now

```text
crates/cmsis-stubs              the CORE_* helpers both blobs reference — compiled once
crates/efr32mg24-bluetooth-sys  bindings (bindgen) + 3 MSLA blobs   -> COMPILES
crates/efr32mg24-rail-sys       bindings (bindgen) + 1 MSLA blob    -> COMPILES
crates/efr32mg24-radio          safe layer + examples               -> ble_beacon LINKS
```

| Example | State | What it proves |
|---|---|---|
| `ble_beacon` | links, not flashed | header/blob/linker agree; the stack can be entered |
| `ble_gatt` | **does not link** | needs PSA crypto blobs (8 symbols, listed below) |
| `rail_version` | links, not flashed | RAIL header and blob are the same version |

Load ranges, measured with `scripts/elf-load-range.py`: `ble_beacon` starts at **0x08006000**
(5856 bytes), `rail_version` at **0x08006000** (1316 bytes). Both sit above the bootloader, which is
the only condition the flash guard cares about.

## The five defects that made this look impossible

### 1. bindgen emulated the anonymous unions, and the wrapper read them as structs

17 × `E0609: no field X on type &__BindgenUnionField<T>`. `sl_bt_msg_t` declares an **anonymous**
union; bindgen represents those with `__BindgenUnionField<T>` regardless of the requested Rust
target (measured: adding `.rust_target(Stable_1_71)` left the output byte-identical). The fix is not
a bindgen flag — it is reading the payload through `as_ref()` in `blocking/events.rs`, which is
`unsafe` and therefore says out loud that the union's active member is the caller's responsibility.

### 2. A flexible array member cannot be indexed

`data.value.data[i]` on a `uint8array` fails: bindgen's `__IncompleteArrayField<u8>` has no `Index`.
The byte is read through `as_ptr().add(i).read()`, with the length taken from the header's own `len`.

### 3. One field renamed between two events of the same SDK

`sl_bt_evt_gatt_server_attribute_value_t` has `attribute`; `sl_bt_evt_gatt_server_user_read_request_t`
has `characteristic`. Same 16-bit handle, two names. Converted in one place, with a comment, because
a silent rename is exactly what a "port the examples" session gets wrong.

### 4. `build.rs` was not watching its own C file

`cmsis_stubs.c` was compiled by `cc` but absent from `rerun-if-changed`, so an edit to it was **not
compiled** — the link kept failing on symbols that were visibly present in the source. Same class of
defect as a book that compiles a stale tree: the build is green, the artefact is old.

### 5. The `-sys` crate had four examples that never compiled

They called `bt::sl_bt_*` (bindings live in `bt::ffi::`) and used `sl_bt_system_start_bluetooth`, a
command this SDK configuration does not declare (it belongs to the "on-demand start" component).
Retired to `_tmp/efr32mg24-rs/retired-ble-examples/`. Examples belong in `efr32mg24-radio`, against
the safe API.

## Two stacks, two different gaps

**Bluetooth is close.** `ble_beacon` links; the only thing left is a board and a phone. `ble_gatt`
fails at link with PSA crypto symbols, because a connectable stack pulls the bonding/encryption
path:

```text
psa_crypto_init      psa_destroy_key    psa_import_key     psa_mac_abort
psa_mac_sign_finish  psa_mac_sign_setup psa_mac_update     psa_reset_key_attributes
```

Those live in Silicon Labs' `libpsa_crypto` + `libmbedtls` builds, which are not vendored here and
are not in the archived copy of this project either. Until they are, `ble_gatt` is a staged file, not a
working example — it must not be counted as one.

**RAIL is further away, and for a different reason.** The blob and headers are vendored (from
`/podman/PROJECTS/_archive/.../efr32mg24-rail-sys`), and `RAIL_GetVersion` links and will run. But
`RAIL_Init` pulls `rfhal_radio.o`, which calls **emlib C sources**, not blob code:

```text
CMU_ClockEnable                 em_cmu.c
EMU_DCDCSetPFMXTimeoutMaxCtrl   em_emu.c
SYSTEM_ChipRevisionGet          em_system.c
SystemSYSCLKGet                 em_system.c
```

Writing those four by hand from headers nobody has would be guessing register bits — the exact way a
radio ends up configured wrong in a way that still transmits. The path is to vendor `platform/emlib`
(the sources are Zlib and are on GitHub) plus the real device headers, and drop the `em_device.h`
shim: `em_cmu.c` alone is 363 KB and needs the complete `em_device.h`, not our 52-line stub.

**And before any of that:** a working RAIL transmission needs a *radio configuration* — channel
definitions, modem settings, PA tables — which the Radio Configurator in Simplicity Studio generates
as `rail_config.c`. That file is board-specific and is the difference between "the library
initialised" and "a packet left the antenna".

## Decisions taken

1. **The CMSIS helpers became their own crate** (`cmsis-stubs`) instead of a copy per `-sys` crate.
   Two copies of `CORE_EnterCritical` would either collide at link or, worse, disagree:
   `CORE_EnterCritical` masks with `BASEPRI`, `CORE_EnterAtomic` masks with `PRIMASK`, and getting
   that wrong corrupts a buffer only under interrupt load.
2. **`#[link(name = "cmsis_stubs", kind = "static")]` in both `-sys` crates.** Nothing in Rust calls
   those symbols, so without an explicit link directive the archive is dropped and every symbol comes
   back undefined.
3. **`--features rail` is a `compile_error!`** in `efr32mg24-radio` until the emlib and radio config
   work is done. A feature that cannot link is a promise, and this file is the evidence it is not
   kept yet.
4. **`ble_gatt` is not in the gate's example count** until PSA is vendored.

## Next steps, in order

1. Vendor `libpsa_crypto` + `libmbedtls` for xg24 → `ble_gatt` links → flash both BLE examples and
   confirm with a phone scanner.
2. Vendor `platform/emlib` sources + real device headers → `RAIL_Init` links.
3. Obtain a `rail_config.c` for this board (Simplicity Studio, Radio Configurator) → first RAIL
   packet.
4. Then, and only then, `--features rail` stops being a `compile_error!`.
