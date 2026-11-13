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

## The link is closed

Measured 2026-11-13: the example builds and flashes with **zero undefined symbols** (129 KB, `load_min=0x08006000`).
Everything below is now about booting, not linking.

### Where the boot stops, and what each answer means

| Heap | Result | Reading |
|---|---|---|
| 4 KiB | `sli_bt_init_stack` returns `0x001A` | `SL_STATUS_NO_MORE_RESOURCE`: an allocation failed. The stack's object allocations come through `sl_malloc`, which is separate from `max_buffer_memory`. |
| 16 KiB | no return; the mailbox is never written, PC sits inside the firmware | The init gets further and then waits. A blocking init cannot serve a **polled** timer, and this project's timer is polled because the SYSRTC compare channel does not set `GRP0_IF`. |

So the next defect to fix is the compare channel, and it is the same one `SYSRTC-FOR-THREE-PROTOCOLS.md` records:
the counter runs at 32 968 Hz, the flag stays clear when the counter passes a programmed deadline. Until it fires,
any stack path that needs a timer callback during initialisation will stall, and raising the heap only moves the
stall later.

The 16 KiB arena stays: it is what showed the difference. `heap::stats()` reports `(in_use, failures, violations)`
and the boot example prints all three plus the largest free block, because "out of memory" and "the arena was
overwritten" are different faults and the status code cannot tell them apart.

## Remaining, and what each really costs

| Symbol | What it is | Cost / risk |
|---|---|---|
| ~~PSA (10 symbols)~~ | **done, with one deliberate refusal** | Key slots and CMAC in `psa.rs`, on `aes-modes::cmac` (RFC 4493). `psa_generate_random` returns `PSA_ERROR_NOT_SUPPORTED`: no TRNG is declared for this part, and a counter is not entropy. |
| ~~`sli_aes_crypt_ecb_radio`~~ | **done** | Signature taken from `sli_protocol_crypto.h` (not inferred): `(encrypt, key, keybits, input, output)`. Decryption is refused with `SL_STATUS_NOT_SUPPORTED` because nothing here needs it and a silent wrong answer would be worse. |
| ~~`sli_ccm_encrypt_and_tag_ble`~~ | **done** | `(data, length, key, iv, header, tag)` from the same header. The CCM* under it is checked against RFC 3610 packet vector 1, ciphertext and tag. |
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
