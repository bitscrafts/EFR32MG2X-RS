# Bluetooth integration: the symbol ledger

Working document, 2026-11-13 (overnight). It exists because "how far is the stack from booting" has one honest
answer: **count the undefined symbols, and name what each one needs.** Two days of work reduced the list from
twenty-odd to thirteen; this file keeps the arithmetic visible and says which of the remaining ones are
mechanical and which are dangerous.

Measure it with:

```bash
cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg24-radio \
  --example ble_beacon --features stack 2>&1 | grep -oE "undefined symbol: [A-Za-z0-9_]+" | sort -u
```

## Resolved, with how

| Group | Symbols | How it was resolved |
|---|---|---|
| `sl_sleeptimer_*` | 10 | Rust, polled — `crates/efr32mg24-bluetooth-sys/src/sleeptimer.rs`. The compare interrupt does not fire yet, so deadlines are polled from the application loop. |
| `sl_malloc` / `sl_free` | 2 | Rust — `libraries/bt-heap` (first fit, coalescing, 6 host tests) exposed by `bluetooth-sys/src/heap.rs` over a 4 KiB static arena. |
| `SYSTEM_GetUnique`, `SYSTEM_ChipRevisionGet` and seven more emlib names | 9 | **Version skew, not missing code**: the blobs are Gecko SDK v4.4.5 and call `CMU_ClockEnable`, `CMU_HFXOCTuneSet`, `EMU_DCDCSetPFMXTimeoutMaxCtrl`, `GPIO_PinModeSet`, which the newer simplicity_sdk generation renamed or does not ship. Vendored the **v4.4.5 emlib** (`crates/efr32mg24-emlib-sys/vendor445/`) next to the existing tree and pointed the build at it. |
| `sl_rail_util_pa_get_tx_power_config_2p4ghz` | 1 | Already compiled into `librail_pa.a`; it was on the link line **before** the Bluetooth blobs, so the archive never satisfied the reference. Now re-linked after them (`DEP_EFR32MG24_RAIL_LIB_DIR`). |
| `nvm3_*` | 5 | Rust surface in `bluetooth-sys/src/vendor_support.rs`, **no persistence yet** (see below). |
| `sli_bt_init_controller_features` | 1 | Rust transcription of the SDK dispatcher: `sl_btctrl_init_adv`, `_scan`, `_conn`, `_phy` — all four verified present in the controller blob before being called. |
| `sli_power_manager_update_*` | 2 | Rust: counted and not acted on, because this firmware never sleeps. The counter is in `vendor_support::stats()`. |

## Remaining, and what each really costs

| Symbol | What it is | Cost / risk |
|---|---|---|
| `psa_crypto_init`, `psa_generate_random`, `psa_import_key`, `psa_destroy_key`, `psa_reset_key_attributes`, `psa_set_key_domain_parameters`, `psa_mac_sign_setup`/`update`/`finish`/`abort` | PSA Crypto: the API the stack's security manager uses to import its keys and compute CMAC | **Real work, and honest stubs are not acceptable**: a stub that returns success without storing a key makes the stack believe it has security it does not have. Needs a key-slot table plus AES-CMAC, both testable against RFC 4493 on the host. `psa_generate_random` must come from the TRNG, not a counter. |
| `sli_aes_crypt_ecb_radio` | single-block AES for the link layer | Implementable with `libraries/aes-ctr-radio`'s block cipher. **The signature is inferred from the call site, not documented**: `(r0=1, r1=pointer, r2=128, r3=pointer, [sp]=pointer)`. A wrong argument order is memory corruption, so this must be validated against a BLE test vector rather than trusted. |
| `sli_ccm_encrypt_and_tag_ble` | BLE's CCM* for encrypted packets | Same shape of problem, six arguments at the call site. CCM* is CTR + CBC-MAC with a 13-byte nonce; the algorithm is public, the ABI is not. Validate against RFC 3610 vectors. |
| ~~`sl_power_manager_subscribe_em_transition_event`~~ | **done** | Recorded and never called: no transition happens in this firmware, so nothing downstream can depend on one. `vendor_support::subscriptions()` reports the count. |

## What the package does and does not contain

Measured by extracting all 1748 `.a` files from `gecko-sdk-v4.4.5` and running `nm` over them:

- **`nvm3_open` is available**, in `libnvm3_CM33_gcc.a`. So real persistence is a vendoring job, not a
  reimplementation: that archive plus `nvm3_default.c`, `nvm3_hal_flash.c`, `nvm3_lock.c` (all present as
  source) and a reserved flash region. Until then the Rust surface answers with "key not found", which is what
  an empty store looks like — bonding will not survive a reset, and the document says so.
- **`psa_*`, `sli_aes_crypt_ecb_radio` and `sli_ccm_encrypt_and_tag_ble` are in none of them.** They ship with
  Simplicity Studio's prebuilt libraries, which is the same missing artifact that `PURE-RUST-PATH.md` records
  for the two PSA symbols. So for these four groups the choice is: implement in Rust and validate against
  published vectors, or obtain the tool.

## Order of work, with the reasoning

1. ~~`sl_power_manager_subscribe_em_transition_event`~~ — done.
2. **AES-ECB + CCM\*** — with RFC 3610 vectors as the contract. Doing them together is right: CCM* uses ECB, and
   a wrong ECB order shows up as a wrong tag rather than as a subtle failure later.
3. **PSA** — key slots + CMAC + TRNG. This is the one that must not be faked, because the stack's security
   state depends on it being real.
4. **nvm3 from the prebuilt archive** — persistence, plus the flash region and the linker work that goes with
   it (a flash write path deserves the same care as `scripts/flash.sh` gives the bootloader region).
5. **Then** the stack boots with a real timer, a real heap, real storage and real crypto, and the examples can
   be run per level.

## A note on the timer, since it is what boots first

The stack schedules its event timer through `sl_sleeptimer_restart_timer`. That works today, polled. The
compare channel does not fire yet (measured: the counter passes a programmed deadline and `GRP0_IF` stays
clear), so **the application loop must call `efr32mg24_bluetooth_sys::sleeptimer::poll()`** — an example that
forgets it will look like a stack that never advances, which is exactly the class of bug this project keeps
paying for. The examples that boot the stack call it in their loop, and that call is the first thing to check
when a Bluetooth example does not progress.
