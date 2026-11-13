<!--
Provenance: independent read-only review by claude-opus-5-5 (Claude Code 2.1.300, plan mode, effort high,
39 turns, US$2.54) of the bring-up brief at /projects/_tmp/efr32mg24-rs/BLE-BRINGUP-BRIEF.md, against HEAD 94158fe
on 2026-11-13. Session afd7330f-0878-498e-8f84-0e8f882043ca (resumable). Nothing here is authorised work: each
step of its Part E needs its own gate.
-->

# Review of BLE-BRINGUP-BRIEF: `0x040C` is the stack refusing randomness, not the radio

## Context

`sli_bt_system_start_bluetooth()` returns `0x040C`, and the brief ranks H1 (TX FIFO) as the most likely cause. The
evidence below says the cause is outside the radio. **`psa_generate_random` in this tree refuses on purpose
(`crates/efr32mg24-bluetooth-sys/src/psa.rs:137-141`). The controller asks it for 32 bytes during `BTLE_LL_Init`, and
the refusal comes back as `0x040C`.** The chain below is proven from the disassembly. One flash cycle has to confirm
that it is the *first* failure.

### The chain (all offsets from `/projects/_tmp/efr32mg24-rs/aes-sigs/*.disasm`)

1. `BTLE_LL_Init+0x54` calls `ll_radioInit`. A non-zero return goes straight back (`+0x58 cbnz r0`).
2. If `ll_radioInit` returns 0, `BTLE_LL_Init+0x70` calls `ll_randomInit`, and `+0x74 cmp r0,#0 / beq` returns
   **`ll_randomInit`'s value** when it is non-zero.
3. `ll_randomInit+0x8` calls `bg_random_get_data(32, sp)` and returns its value unchanged.
4. `bg_random_get_data` (bgcommon) runs `bg_random_init` (→ `psa_crypto_init`, which our stub passes), then
   `psa_generate_random`. Any non-zero result becomes **`0x0407` = `SL_STATUS_BT_CRYPTO`** (`+0x14..0x1c`).
5. `bg_radio_init+0x5e` computes `ands r5, r0, #0xff`, which gives `0x07`, non-zero, so it returns `0x040C`.

---

## Part A: Diagnosis

| H | Verdict | Evidence | Consequence |
|---|---|---|---|
| **H6 (new): `ll_randomInit` fails because PSA RNG refuses** | **Supported by code. Certain to fail if it is reached.** | Chain above. `psa.rs:140` returns `-134` unconditionally. | This is a blocker whether or not A or B also fail. It must be fixed in every scenario. |
| H1: `RAIL_SetTxFifo != 512` (path A, returns 7) | Unlikely | `rail.h` (GSDK 4.4.5, `RAIL_SetTxFifo` doc): the result is "the closest allowed size ≤ size… 0 if an error occurs". It depends only on hardware FIFO sizes and alignment. Channel or PHY configuration is never mentioned. Args are `(h, ll_radioTxBuffer, initLength=0, size=512)` (`ll_radioInit+0x74..0x7e`). | No `RAIL_ConfigChannels` is needed first. |
| H2: `ll_radioInitPA` non-zero (path B) | Unlikely, still open | `ll_radioInitPA` returns `1` only if `RAIL_SupportsTxPowerModeAlt` rejects **every** mode from the configured one (`SL_RAIL_UTIL_PA_SELECTION_2P4GHZ = HIGHEST = 4`) up to 13 (`+0x2c..0x50`). It never touches `RAILCb_PaAutoModeDecision`. | Q4: the PA auto-mode callback is irrelevant to init. Calling `sl_rail_util_pa_init()` changes nothing here. |
| H3: an earlier `BTLE_LL_Init` step fails | **Ruled out as stated** | `BTLE_LL_Init` ignores the returns of `ll_hciInit`, `ll_initSetLocalFeatures`, `BTLE_LL_SetFlags`, `bgbuf_add_callback`, `ll_execTimingTableInit`, `ll_channelmapInit` and `ll_addrInit`. Only `ll_radioInit` and `ll_randomInit` can fail it. | Drop `ll_execTimingTableInit` as a suspect. |
| H4: LL config empty because `linklayer_config = 0` | Ruled out for this failure | `bg_radio_init` builds the LL config from config offsets `+8, +12, +30, +32` and the PA mode byte (`sli_bt_get_pa_mode_config()` returns 0). `BTLE_LL_Init+0x2a..0x3e` replaces a NULL priority table with `ll_priorityTableDefault`. No return value on this path depends on it. | Q5 is not on the critical path. I did not verify the SDK's generated value. |
| H5: controller `RAIL_Init` returns NULL | Ruled out | A NULL handle faults at `ll_radioInit+0x62`, and the fault record is empty. The brief's own reasoning holds. | — |

## Part B: The discriminating experiment (one build plus one flash, about 2 min)

**Why the brief's plan can't work:** path A returns `7` and the RNG path returns `0x0407`, so both have low byte
`0x07`. The low byte cannot tell them apart. Use the counter the firmware already keeps, plus two controller
globals. `nm` on the blobs shows `B ll_radio` (64 B) and `B ll_radio_pa` (24 B), both global.

Source change, as text only, in `stack_init::start()` around `stack_init.rs:259`. Trace slots 1–3 are free after the
existing `trace(2,0)`/`trace(3,0)`:

```rust
extern "C" { static ll_radio: [u32; 16]; static ll_radio_pa: [u16; 12]; }
let before = crate::psa::stats().2;
let started = sli_bt_system_start_bluetooth();
trace(7, started);
trace(1, crate::psa::stats().2 - before);                                   // RNG requests during start
trace(2, core::ptr::read_volatile(core::ptr::addr_of!(ll_radio[10])));      // +0x28 = RAIL handle
trace(3, core::ptr::read_volatile(core::ptr::addr_of!(ll_radio_pa[10])) as u32); // +20 = 0xFFFF only if PA init succeeded
```

Then `scripts/gate.sh`, `scripts/flash.sh <probe ELF> --confirm --run`, and read the trace words at `0x2003_F600`
(`scripts/read-mailbox.sh`). **Use the trace words, not the text record.** The record is cut at `len=64`, mid-field
(`tick=1/`), so anything appended to it is lost.

| `[1]` RNG delta | `[2]` handle | `[3]` | Meaning |
|---|---|---|---|
| ≥ 1 | ≠ 0 | `0xFFFF` | **Expected.** `ll_radioInit` passed (A and B ruled out). The failure is the RNG refusal. Go to Part E step 2. |
| 0 | ≠ 0 | `0xFFFF` | Path A (FIFO). Next, wrap `RAIL_SetTxFifo` and record its return. |
| 0 | ≠ 0 | ≠ `0xFFFF` | Path B (PA). Next, record `RAIL_SupportsTxPowerModeAlt` per mode. |
| 0 | 0 | — | `ll_radioInit` was never reached, or `RAIL_Init` returned NULL. Re-examine H5. |

The definitive alternative, also debugger-free: `BTLE_LL_Init` is `U` in the host archive, so
`cargo:rustc-link-arg=--wrap=BTLE_LL_Init` plus a Rust `__wrap_BTLE_LL_Init` that records the **full 32-bit** return
works. Expect `0x00000407`. It costs more plumbing than the counter, so use it only if the table above is ambiguous.

## Part C: Ranked findings

| # | Finding | Evidence | Why it matters | Closes it |
|---|---|---|---|---|
| 1 | PSA RNG refusal fails `BTLE_LL_Init` | chain above | The current blocker | Part E step 2 |
| 2 | "No TRNG on this part" (`psa.rs:9-14`) is false | The device header defines `SEMAILBOX_PRESENT` (`emlib-sys/vendor/device/*.h:443`). `sl_se_manager_entropy.c:48-80`: `sl_se_get_random` → `SLI_SE_COMMAND_TRNG_GET_RANDOM` (`0x07000000`). GSDK `sl_entropy_hardware.c` counts `SEMAILBOX_PRESENT` as having a TRNG. | The refusal rests on a wrong premise. A real hardware TRNG exists behind the SE mailbox. | Real `psa_generate_random` via the SE |
| 3 | Low byte `0x07` is ambiguous (path A = 7, RNG = 0x0407) | `ll_radioInit+0xfa`, `bg_random_get_data+0x16` | Q2's "recover the low byte" would not discriminate | Part B |
| 4 | Bare metal is supported. No RTOS shim is needed. | `protocol/bluetooth/src/sl_bluetooth.c.jinja`: without a kernel it enables `PendSV_IRQn`, defines `PendSV_Handler(){ sl_bt_priority_handle(); }`, calls `sli_bt_system_start_bluetooth()` in `sl_bt_init`, and `sl_bt_step()` = `sl_bt_run()` + `sl_bt_event_pending_len()` + `sl_bt_pop_event()` | Q6: drop the CMSIS-RTOS2 idea | Part E step 3 |
| 5 | `pump()` never calls `sl_bt_run()` | `stack_init.rs:132-136`. `sl_bt_run` is `T` in the host blob. | Even after a good start, the host's main processing never runs, so `system_boot` may never be queued | Part E step 3 |
| 6 | `SL_STATUS_NOT_SUPPORTED = 0x0E` in `support.rs:81` | `sl_status.h:101` defines `0x000F`, and `0x000E` is `NOT_AVAILABLE` | The §6.6 sweep missed this one. A wrong status code again. | One-line fix |
| 7 | Stale comments in `stack_init.rs:212-221` and `:226-228` say the app initialises RAIL and that "the crystal travels inside the chip-specific configuration" | They contradict §6.4, and the second is unverified | Misleads the next reader | Rewrite with the start fix |
| 8 | The text record truncates at 64 bytes | §4.1 record ends at `tick=1/` | §3's `heap=3664/0/0` cannot come from the current record | Enlarge or reorder, or rely on trace words |
| 9 | BLE PHY and crystal: open, next risk | The controller imports `RAIL_BLE_ConfigPhy1MbpsViterbi/2Mbps/Coded/Simulscan`. These pass library-internal configs (`ble_efr32xg24_configurator_out.o` in `librail`) and are called at PHY-switch time, not in `ll_radioInit`. | Q8: there is no supported way to inject the app's table into the controller's instance, and none is needed. Whether the built-in config matches the XIAO's 39 MHz crystal shows only at the first advertisement (RAIL assert 66 would reveal a mismatch). | Part E step 4 |

## Part D: Corrections to premises

- **§4.2 "error path C: return 0xFF if config is NULL"** is wrong. At `ll_radioInit+0xa6`, `cbz r5` goes to `+0xf6`
  (`r0=0xFF`), then **back to `+0xaa`** (`bl ll_radioInitPA`). A NULL config means "PA mode 0xFF", not an error. The
  pointer is also never NULL here (`bg_radio_init` passes `sp+4`).
- **§4.2 "normal exit returns the config pointer"** is wrong. `+0xae mov r5, r0` overwrites it with
  `ll_radioInitPA`'s result, so success returns 0. The brief also omits `RAIL_BLE_Init` (`+0xb4`),
  `ll_radioPaSetAccurateTxPowerLimits` and the feature setters.
- **§4.2 list of earlier `BTLE_LL_Init` failures** is wrong. Only `ll_radioInit` and `ll_randomInit` can propagate.
  `ll_randomInit` comes **after** `ll_radioInit`, not before.
- **§5 / §10 Q6 "blobs built for the RTOS configuration"** is wrong. Importing `RAIL_Init` and exporting
  `sl_bt_pop_event` is equally true of the SDK's bare-metal build (`sl_bluetooth.c.jinja`). The Arduino core is one
  RTOS consumer, not evidence of how the library was built.
- **§6.5's conclusion is confounded.** It was measured while start was failing, so "no events" never isolated the
  delivery mechanism. With `scheduler_callback` NULL, the host's `ll_task_default_callback` writes `PENDSVSET`
  (`0x10000000` → `0xE000ED04`), which is the bare-metal design.
- **§9 H1 premise "FIFO size is a property of the radio configuration"** is contradicted by `rail.h`'s
  `RAIL_SetTxFifo` documentation.
- **§3 "no assertion, no fault"** is sound (the slots are cleared since §6.7). **§3 heap figures**: see finding 8.

## Part E: Shortest path to a working example

1. **Run the Part B experiment** (≈2 min: build 30 s, flash 40 s, read). This confirms the RNG is the first blocker.
2. **Real `psa_generate_random` from the SE TRNG** (≈half a day plus 2–3 flash cycles).
   - Compile the vendor `sl_se_manager.c` and `sl_se_manager_entropy.c` (already extracted at
     `/projects/_tmp/efr32mg24-rs/gecko-src/platform/security/sl_component/se_manager/src/`) with the bare-metal OSAL
     (`sli_se_manager_osal_baremetal.h`), plus emlib `em_se.c`.
   - Call `sl_se_init()` once before `sl_bt_stack_init`, then `sl_se_get_random()` inside `psa_generate_random`.
   - Check the `SEMAILBOX` clock enable bit in `CMU_CLKEN1` (the PAC has it in `cmu_s/clken1.rs`).
   - Keep the module's honesty rule. On any SE error, return `PSA_ERROR_HARDWARE_FAILURE` (`-147`) and write
     nothing. **Never fall back to a counter or `RAIL_GetRadioEntropy`.**
   - Update the `psa.rs` module doc, whose premise was wrong.
   - Expect `boot` to become `0` or a *different* code from later in `sli_bt_start_stack`.
3. **Bare-metal loop exactly as `sl_bluetooth.c.jinja`** (≈1 h plus 1 flash).
   - Add a cortex-m-rt `#[exception] fn PendSV() { sl_bt_priority_handle() }`.
   - Leave `scheduler_callback`/`stack_schedule_callback` NULL, or keep the flags but then also handle PendSV.
   - Make `pump()` do `sl_bt_run()`, then `if sl_bt_event_pending_len() > 0 { sl_bt_pop_event() }`.
   - Expect `evt=n/1`, meaning `system_boot` was seen.
4. **Legacy advertiser example** (≈1 h plus 2 flashes). Watch the RAIL assert slot `0x2003_F500` for `66` (the
   crystal mismatch from finding 9), and confirm with `btmon` on the host.
5. **Cleanup** (≈30 min, gate only): fix `support.rs:81` to `0x0F`, remove the stale comments and the unused
   `RAIL_CONFIGURATION`, and fix record truncation.

## Verification

- Step 1: the trace-word table in Part B, read through `scripts/read-mailbox.sh`.
- Step 2: `[1]` RNG delta ≥ 1 **and** `boot != 0x040C`, plus a host-side unit check that two `psa_generate_random`
  calls return different 32-byte buffers. **That check would fail if the SE were not wired**, because the stub writes
  nothing.
- Step 3: `evt` boot count = 1. Step 4: `btmon` shows `ADV_IND` from the stack's address, not `C0:DE:00:00:00:01`.
- Each step: `scripts/gate.sh` → `VERDICT: PASS`, and the flash guard confirms `load_min ≥ 0x08006000`.
