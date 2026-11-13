# BLE — the state of the bring-up, measured

Updated 2026-11-13. Every number below came from the board, not from a datasheet or from hope.

## One line

The radio works, the host initialises and **answers BGAPI commands** (`sl_bt_system_get_version` → 7.2), and the
event queue stays empty, so no example advertises through the stack yet. The radio has been proven separately by a
pure-RAIL advertisement that three independent receivers decoded.

## Update 2026-11-13 (round 2): the link layer's advertising is on the air

`ble_beacon` is heard by the host adapter: address `E4:0A:75:13:3E:E0`, `Name (complete): xiao-mg24`, flags 0x06, 14
bytes of data, about -83 dBm. The capture was valid in the same window (foreign devices at -57 dBm in it).

| Claim | Evidence |
|---|---|
| The image was the problem, not the radio, the PA or the scheduler | the same link layer, unchanged, is heard once the stack no longer owns the SWD mailbox page |
| Cause | `memory.x` gave the HAL 256 KB of RAM, so `_stack_start = 0x20040000` and frames reached `0x2003F5AC`; the mailbox (`0x2003F000`) and trace words sit in that page. RAM is now 252 KB, `_stack_start = 0x2003F000` |
| Scheduler is healthy | `diag` build, counters read without halting: 114 scheduled transmits in 12 s (9.5/s), two more `RAIL_StartTx` per event, every status 0, lead 99-109 ms before each transmit, cadence 100 ms |
| The default scheduler callback is the one in use | `ll_task_callback = ll_task_default_callback`, which writes `PENDSVSET`; the Rust `default_configuration()` is dead code, because `sl_bt_stack_init()` passes the vendor's own config |
| Not the time source, not buffers | exec timing reads `RAIL_GetTime`, not DWT; pool 0 stays at 32 free of 37 |
| Advertising data defect | `sl_bt_advertiser_general_discoverable` made the stack build its own PDU (flags only, 3 bytes); `start_advertising` / `start_connectable` now use `sl_bt_advertiser_user_data` |
| `ble_advertise_custom` PASS | manufacturer data 0x02E5 on air with the uptime counter advancing; the example had two defects of its own (a 5-byte copy into a 4-byte slice, which panicked, and `set_manufacturer_data` appending instead of replacing) |
| `ble_scanner` | NOT working: starts (`scan: after new`) but no advertisement is reported, with neighbours at -57 dBm. Tried: `SL_CATALOG_BLUETOOTH_FEATURE_SCANNER_PRESENT` plus mapping `scanner_scan_report`; no effect measured, so neither change is kept |

Still weak: -83 dBm for a board next to the host; the same level the RAIL-only test showed.

## Update 2026-11-13 (late): the HCI NULL is closed; the beacon is still not heard

| Claim | Evidence |
|---|---|
| `ble_beacon` no longer faults; `ll_hci.sharedCmd` is set | mailbox `status=PASS`; `mdw ll_hci` shows the pointer (was 0) |
| Cause of the old NULL: `ll_hciInit` ran twice | breakpoints on `ll_hciInit` and after each `bgbuf_alloc`: the first call allocates, the second (inside `sli_bt_start_stack`) returns NULL because the re-initialised pool's type-2 count underflowed when the first call's buffers were freed |
| Fix | the application no longer runs the NCP-style controller initialiser; the host start does that work itself |
| The radio, PA and PHY work on the controller's own RAIL handle | a hand-driven `RAIL_StartTx`/`RAIL_StartScheduledTx` loop on that handle is decoded by btmon at -67..-75 dBm |
| The link layer's own advertising is **not** heard | `ble_beacon` (every command 0); a 100 % duty host scan sees nothing |

Open: why the link layer's transmissions do not radiate while the same handle driven by hand does.

## What is proven, and how

| Claim | Evidence |
|---|---|
| HFXO runs and RAIL accepts a 39 MHz channel table | `chan=0` (NO_ERROR), metadata reads **2402 MHz on channel 0**, **2426 on channel 12** |
| The Bluetooth host initialises with the SDK's own tables | `sl_bt_stack_init()` → `0`, trace slot 5 |
| The host runs | `sl_bt_system_get_version` → `0`, returns **version 7.2** |
| Our timer works from its interrupt | `tick=1/1`, callback from `SYSRTC_APP_IRQn`, compare fires one tick late |
| The heap works | `heap=3664/0/0` — bytes in use, allocation failures, list violations |
| The radio transmits BLE advertising | `rail_ble_tx`: `ADV_IND`, `C0:DE:00:00:00:01`, RSSI −71…−73 dBm, decoded by btmon, BlueZ and the link layer |
| RAIL does not assert | assertion record empty after a clean run |

## The wall today, precisely

```
boot=0483 stcmd=000E cmd=0 ver=7.2 evt=0/0 pop=45 tick=1/1 heap=3664/0/0
```

- `boot=0483` — `sli_bt_start_stack()` returned `0x0483`, a value in the Bluetooth status space (0x0400) that no
  header in the package defines. It may not be a status at all: the function's contract is to schedule the stack.
- `stcmd=000E` — `SL_STATUS_NOT_AVAILABLE`, which is **correct**: without the on-demand start component the stack is
  meant to start by itself at boot, and that command is unavailable by design.
- `cmd=0 ver=7.2` — the host answers, so it is not dead.
- `evt=0/0 pop=45` — `sl_bt_pop_event` returns `SL_STATUS_NOT_FOUND`: the queue is empty. No `system_boot`, and no
  event from any command.

The open question is therefore narrow and specific: **what makes a bare-metal GSDK host queue events when both
scheduler callbacks are null?** The vendor's own answer for a kernel build is `sl_bt_rtos_adaptation.c`
(`sli_bt_rtos_ll_callback` / `sli_bt_rtos_stack_callback`), which is not in any blob and is not compiled here. For
the bare-metal build the SDK's `SL_BT_CONFIG_DEFAULT` sets both callbacks to 0, which is what this project does, so
either something else in the vendor's bare-metal path is missing, or `sl_bt_pop_event` is the wrong accessor for
this configuration.

## Defects found and closed on the way here

1. **`sli_bt_init_stack` takes three arguments.** Calling it with NULL feature and BGAPI-class tables initialises a
   host that registers nothing: it returns 0 and never emits an event. The SDK's `sl_bt_stack_init()` wrapper is now
   vendored and compiled, with a component catalogue for a minimal peripheral.
2. **RAIL has one owner per protocol.** `RAIL_Init` "will do nothing and return NULL" if called twice, so the
   application must be the caller and must configure channels on the instance it created — `sl_rail_util_init.c`,
   the vendor's own answer, does exactly `RAIL_Init` → `RAIL_ConfigData` → `RAIL_ConfigChannels`.
3. **The HFXO must be running before RAIL configures the radio.** Without it the device reports zero hertz, and
   `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (66) is then *correct* rather than a puzzle.
4. **RAIL and the application both touch the SYSRTC.** `CFG` is writable only while the counter is disabled; an
   application that starts the timer first turns RAIL's configuration write into a bus fault at `0x500a_800c`
   (`cfsr=0x8200`), escalated to a HardFault inside the fault handler. The HAL's `init` now returns early on a
   running module, and examples create the timer after the stack.
5. **`nvm3_open` must not lie.** Returning `ECODE_NVM3_OK` while leaving the handle untouched made the stack
   dereference a null handle 0x138 bytes in (`bfar=0x0000_0138`, `cfsr=0x8200`). It now returns
   `ECODE_NVM3_ERR_NO_VALID_PAGES`, which is the truth while persistence is unimplemented.

## Instruments, and the one that lied

- `crates/efr32mg24-radio/examples/ble_boot_probe.rs` — one flow, eight trace words, a reporting HardFault handler,
  and every diagnostic slot cleared before the run.
- `crates/efr32mg24-rail-sys/examples/rail_ble_config_probe.rs` — proves the channel table alone, without the stack.
- `scripts/ble-run-and-report.sh`, `scripts/ble-examples-status.sh`, `scripts/read-mailbox.sh`.

**The instrument that lied was the probe itself.** It grew by patch until it called `stack_init::start` twice, and
the second call's trace overwrote the first call's, so a stage that had failed read as a stage that had passed. It
was rewritten from scratch. The same class of error — a diagnostic record surviving a reset and reading as fresh —
produced two false readings before the slots were cleared at boot.

## The thirteen examples

`scripts/ble-examples-status.sh` builds all thirteen for `thumbv8m.main-none-eabihf`; none loads below
`0x08006000`, which is the bootloader's floor.

- **Proven on the board:** `rail_ble_tx` (advertising, three receivers), `rail_caps` (PHY capabilities),
  `ble_boot_probe` (bring-up trace, timer by interrupt, the named assertion), `rail_ble_config_probe` (the table).
- **Diagnostic:** `rail_ble_rx` (builds; reception not measured), `bt_version` (builds; needs a running host).
- **Proven on air since round 2:** `ble_beacon`, `ble_advertise_custom`.
- **Blocked on the event queue:** `ble_scanner` (see round 2), `ble_gatt`,
  `module_ble_beacon`, `module_ble_motion`, `module_ble_fall`, `base_ble_env`. They build, flash, and stop for the
  same reason: the host runs but no `system_boot` reaches the application.
