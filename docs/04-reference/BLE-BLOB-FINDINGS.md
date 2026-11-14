# What the vendor's blobs actually do — findings from decompilation

This file records what reading the shipped binaries established about the Bluetooth bring-up, **in our own words
and with no vendor code in it**. The decompiled output itself lives outside the tree, because the blobs are MSLA-licensed and this repository is
public. The
rule and its reasoning are in `BLOB-DISASSEMBLY.md`.

Written 2026-11-13 after that dossier found the cause of a `0x040C` (`SL_STATUS_BT_RADIO`) failure that had resisted
weeks of work: twelve functions were decompiled — `sli_bt_system_start_bluetooth`, `sli_bt_start_stack`,
`bg_radio_init`, `BTLE_LL_Init`, `ll_radioInit`, `ll_radioInitPA`, `ll_randomInit`, `bg_random_get_data`, `sl_bt_run`,
`sl_bt_priority_handle`, `sl_bt_pop_event`, `sl_bt_event_pending_len`.

## The bring-up chain, as the code orders it

1. **`sli_bt_system_start_bluetooth`** is the entry point the application calls. It runs `sli_bt_start_stack()`,
   and if that succeeds it installs the BGAPI device with `bgapi_init_device()`, and only then calls the function
   that **queues `system_boot`**. Two consequences: the boot event is produced *inside* the start call (not by the
   main loop), and any failure in the start means the boot event is never queued at all.
2. **`sli_bt_start_stack`** clears the out-of-memory observers, initialises two buffer pools from the heap, then
   opens NVM3. A failed `nvm3_open` is **fatal**: the error is converted and returned before anything else runs. That
   is why a stubbed NVM3 stopped the start at `0x0483` and why a stub that lied about success crashed the stack
   later, reading a handle that had never been filled in.
3. **`bg_radio_init`** builds a link-layer configuration on its own stack from the RF part of the configuration, calls
   `BTLE_LL_Init`, and — this is the detail that cost the most time — **masks the result with `0xFF`**, returning the
   fixed `0x040C` if any low byte is non-zero. The real error code is destroyed there.
4. **`BTLE_LL_Init`** ignores the return values of its early steps (HCI init, local features, flags, buffer callback,
   timing table, channel map, address, random). Only two calls can fail it: **`ll_radioInit`** and, *after* it,
   **`ll_randomInit`**.
5. **`ll_radioInit`** fills the controller's own RAIL configuration (including its event handler), calls `RAIL_Init`
   itself, stores the handle, configures events and RX options, sets the TX FIFO, sets state timing, initialises the
   PA, then brings up the BLE PHY, timing and multi-timer. Its failure paths are: the FIFO call not returning 512
   (returns 7), the PA stage returning non-zero, or `RAIL_ConfigChannels` never being needed at all.
6. **`ll_randomInit`** asks for **32 bytes** from `bg_random_get_data`, feeds them into its own AES-CTR DRBG and
   propagates any failure. **`bg_random_get_data`** calls `psa_generate_random` and maps any non-zero result to
   `0x0407` (`SL_STATUS_BT_CRYPTO`) — whose low byte is **also 7**, which is why the masked `0x040C` could not
   distinguish "TX FIFO" from "no entropy".

That last sentence is the whole diagnosis: the two candidate failures share a low byte, so the status the
application receives is ambiguous by construction.

## Why the stack needs the application at all

- **RAIL belongs to the controller.** It creates the instance, so an application that calls `RAIL_Init` first makes
  the controller's call return NULL, and the next RAIL call faults.
- **The NVM3 default instance belongs to the application** — the controller only opens it.
- **Event delivery belongs to the application.** Without a kernel the host asks for link-layer work by setting
  `PENDSVSET` on the `PendSV` exception; `sl_bt_priority_handle()` is the service routine. There is no other
  mechanism, and a firmware with no `PendSV` handler stops in `DefaultHandler` with `ICSR` reporting vector 14 — no
  fault, no status, indistinguishable from a deadlock.
- **The host has its own processing step.** `sl_bt_run()` advances GAP/GATT state and the stack's software timers;
  `sl_bt_event_pending_len()` and `sl_bt_pop_event()` are how the application collects what it produced. Polling
  `sl_bt_pop_event` without `sl_bt_run` answers "no event" for ever while the stack is healthy. That is the shape of
  a `blocking` API that waits for a command response: it must run the host while it waits.

## The entropy source, and a false premise this project held

`psa_generate_random` refused, on the stated grounds that this part declares no TRNG. The device header says
otherwise: the Secure Engine is present, and the vendor's own random-number routine issues a TRNG command through
the SE mailbox on this exact part. The mailbox protocol is small — clock gate, a four-word FIFO exchange, a
descriptor in RAM, a bounded wait, a response code in the header — and is implemented in Rust in `se_trng`.

Measured after that change: two calls, both status 0, a pre-filled sentinel overwritten (the SE really writes to
memory), and the two buffers different.

## What this method is worth, stated as a rule

Every one of the four facts above is absent from every header in the SDK, and three of them were actively
contradicted by this project's own earlier conclusions. The cost of getting them by decompiling was a few hours; the
cost of *not* having them was weeks of work spent on the radio, the crystal, the channel table and the PA, all of
which were innocent.
