# The Bluetooth bring-up chain, measured

The scanner on the XIAO failed with `SL_STATUS_NOT_AVAILABLE` and, once the real entry points were
used, the linker asked for twenty symbols. This is the whole chain, in the order it was discovered,
with the source of each piece.

## 1. The wrong entry point, and the code that proves it

`Bluetooth::new()` called `sl_bt_system_start_bluetooth()`. On this board it returns **`0x000E`**:

```c
#define SL_STATUS_NOT_AVAILABLE ((sl_status_t)0x000E)  ///< Feature not available due to software configuration.
```

Measured on hardware, reported through the mailbox as `scan: boot failed bgapi14`. The command exists
in the blob (the blob is built from a superset) but the *feature* behind it — the "Bluetooth on-demand
start" component — is not part of this build. **The link succeeds and the runtime refuses:** a return
code is the only evidence.

## 2. The entry points this build does have

Exported by `libbluetooth_host` (verified with `arm-none-eabi-nm`), absent from the bindings because
the allowlist matches `sl_bt_*` and these are `sli_bt_*`:

```text
sli_bt_init_stack      initialise the host with a configuration   (3 arguments, config first)
sli_bt_start_stack     start it; the stack then emits system_boot
sli_bt_send_boot_event
```

`sli_bt_init_stack` reads `config->config_flags` first — the disassembly tests bit 8
(`SL_BT_CONFIG_FLAG_RTOS`) on the first word — so the struct must be laid out exactly as
`sl_bt_stack_config.h` declares it. That header *is* vendored, and the structure is reproduced in
`efr32mg24-bluetooth-sys/src/stack_init.rs`.

## 3. What the real path needs: 20 symbols in four groups

```text
nvm3 (5)      nvm3_defaultHandle   nvm3_defaultInit   nvm3_deleteObject
              nvm3_open            nvm3_readData
RAIL (5)      RAIL_BLE_Supports2MbpsNonViterbi   RAIL_BLE_Supports2MbpsViterbi
              RAIL_ConfigSleep     RAIL_SetTune     RAILCb_PaAutoModeDecision
emlib (2)     SYSTEM_ChipRevisionGet   SYSTEM_GetUnique
PSA (8+)      psa_crypto_init  psa_import_key  …  (the bonding/encryption path)
```

The finding that matters: **the Bluetooth controller is built on RAIL.** The linker discovered it —
`RAIL_ConfigAntenna`, referenced by `efr32xg2x.c` — and that in turn pulls RAIL's own dependencies.
This is why a Bluetooth-only project still needs the RAIL library and its chip layer.

### Where each group comes from

| Group | Source | Licence | Work |
|---|---|---|---|
| nvm3 | `platform/service/nvm3/src/*.c` in the SDK | Zlib | vendor ~10 C files, one build script |
| RAIL extras | partly the RAIL blob, partly the app | MSLA / ours | `RAILCb_PaAutoModeDecision` is an **application callback**: we write it |
| emlib | `platform/emlib/src/em_system.c` | Zlib | two functions, or vendor the file |
| PSA | `libpsa_crypto` + `libmbedtls` | MSLA | vendor, or ship a documented no-security build |

## 4. Why this changes the earlier estimate

`BLE-GATT-GAP.md` counted twelve symbols for a connectable build. That count was taken while the code
was still calling the **wrong** init command: the linker then only needed what the advertising path
referenced. With the correct entry point, the advertised path itself needs nvm3 (bonding keys, and the
GATT database's persistence) and RAIL. The honest number for "advertising works" is **12 of these 20**,
and for "connectable with security" all 20 plus PSA.

## 5. The order to do it

1. **nvm3** — Zlib source, self-contained, and it unblocks the stack's init independent of everything
   else. It also needs a flash area reserved, which is a linker-script decision.
2. **`RAILCb_PaAutoModeDecision`** — ours; a constant answer is valid for a board with no PA switch.
3. **emlib's two functions** — `SYSTEM_GetUnique` reads the factory DI page; `SYSTEM_ChipRevisionGet`
   reads the chip revision. Both are documented in the reference manual.
4. **PSA** — last, and deliberately: it is the only step that needs a vendor binary that is not in this
   repository.

## 6. What is proven, and what is not

Proven on hardware: the wrong command returns `0x000E`; the correct entry points exist in the blob and
their first argument is a `sl_bt_configuration_t`; the controller needs RAIL.

Not proven: that the stack boots once nvm3 is present. The next measurable step is nvm3 plus the PA
callback, then the same mailbox read — `scan: after new` followed by `scan: listening` would be the
first real sign of life.
