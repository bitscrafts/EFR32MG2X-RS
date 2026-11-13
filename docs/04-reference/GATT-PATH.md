# The path to GATT, after advertising works

Measured 2026-11-13. Advertising is done and proven at three levels; this file is what separates it from a
connectable GATT server, with the evidence for each step.

## What is proven

```
controller      btmon decodes our ADV_IND: Address type Random (Static), Flags 0x06, Name "xiao-mg24", -73 dBm
stack           BlueZ keeps it:      Device C0:DE:00:00:00:01 xiao-mg24, alias set, RSSI -73
link            bluetoothctl connect reaches the board and the connection is established,
                then dropped by the local side: le-connection-abort-by-local
```

That last line is the boundary. The advertisement is connectable, the controller completes the connection
procedure, and the firmware drops it because there is no link layer running to answer. **Advertising is a
transmitter. A connection is a link layer. GATT is a host stack.** Three different pieces of work.

## The four symbols between here and a GATT server

Linking the vendor Bluetooth stack now fails on exactly these (down from twenty; the sixteen others were the
RAIL link order and the PA plugin):

| Symbol | What the blobs do with it | Where a real implementation lives | Status |
|---|---|---|---|
| `sli_psa_context_get_size` | `r0 = 6`, then `bl bg_zalloc` — it returns the **allocation size** for a PSA context type | Silicon Labs prebuilt PSA library | **not obtainable from any public source** (see below) |
| `sli_aes_crypt_ctr_radio` | `(dst, 128, src, key, arg5, arg6)` — six arguments, from `ll_randomAesCtrEncrypt` | Silicon Labs prebuilt PSA library | **not obtainable from any public source** |
| `sl_sleeptimer_restart_timer` | the stack's timers | `platform/service/sleeptimer` — **vendored here** (`crates/efr32mg24-emlib-sys/vendor/sleeptimer`) | source available; needs the SYSRTC interrupt wired |
| `nvm3_open` | persistence for the bonding database and stack keys | `platform/emdrv/nvm3` — **vendored here** (`.../vendor/nvm3`) | source available; needs a reserved flash region and `nvm3_default_common_linker.c` |

The signatures above were read out of the blobs' relocations and the argument registers loaded before each
call, not guessed:

```
$ bash /projects/_tmp/efr32mg24-rs/extract-signatures.sh
   sli_psa_context_get_size:  movs r0, #6 ; bl <sli_psa_context_get_size> ; bl <bg_zalloc>
   sli_aes_crypt_ctr_radio:   mov r0, sl ; movs r1, #128 ; mov r2, r7 ; ldr r3, [pc,#40] ;
                              strd r9, r8, [sp] ; bl <sli_aes_crypt_ctr_radio>
```

## The wall, stated plainly

These two are **not** in any source tree this project can reach, and that was checked rather than assumed:

- `SiliconLabs/simplicity_sdk` has no implementation of either; its `platform/security/sl_component` holds
  drivers (`sl_psa_driver`, `sl_mbedtls_support`, `sli_crypto`) but not these two functions.
- `SiliconLabs/mbedtls` is upstream mbedtls: 1279 files, and zero `sli*` files.
- The public SDK ships **no prebuilt libraries** (`platform/emdrv/nvm3/lib`, `platform/security/lib`: 404).

They live in the prebuilt PSA archive that only a Simplicity Studio / Gecko SDK *installation* carries.

Writing them from inference is the option this project rejects, and the reason is in the first row of the
table: `sli_psa_context_get_size` returns a size that is immediately passed to an allocator. A wrong number
does not fail — it corrupts the heap later, in code nobody is looking at. A zero `sli_aes_crypt_ctr_radio`
would make every connection's random numbers predictable, which is worse than not connecting.

## Three ways forward, cheapest first

1. **Obtain the prebuilt PSA archive.** Any Simplicity Studio install (or the Gecko SDK package) for
   `efr32mg24` carries `libpsa_crypto.a` (+ `libmbedtls.a`) with both symbols. Vendoring them is the same
   mechanical step as the RAIL and Bluetooth blobs, and it closes the table outright. **This is the
   recommended path** and the only one that needs no invention.
2. **Vendor `nvm3` + `sleeptimer` first anyway** (two of four, sources in hand): it does not make GATT link,
   but it removes two unknowns and is required in every variant. Needs a flash region (the part has 1536 KB;
   the app currently ends far below `0x0801_8000`) and the SYSRTC interrupt.
3. **Write a minimal host stack over RAIL.** Advertising and connection supervision we would own, then
   ATT/GATT on top. It is the largest option by an order of magnitude, and it trades a licence artefact for
   several thousand lines of protocol code that the vendor already ships.

## What to measure when the archive arrives

1. `arm-none-eabi-nm libpsa_crypto.a | grep -E "sli_psa_context_get_size|sli_aes_crypt_ctr_radio"` — both
   present, both `T`.
2. Link `ble_gatt`; the undefined list must be empty.
3. Flash; the mailbox must show the stack's boot event (`system_boot`) with a Bluetooth address, which is the
   first thing the stack reports and the first thing this project has never seen.
4. `bluetoothctl connect` must stay connected; `bluetoothctl gatt` (or `nRF Connect`) must list the service.
