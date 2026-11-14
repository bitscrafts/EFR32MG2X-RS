# BLE — the state of the bring-up, measured

Updated 2026-11-13. Every number below came from the board, not from a datasheet or from hope.

## One line

**Advertising, scanning and both sensor examples are measured on the board; serving a GATT attribute is not.**
"BLE works" is the kind of sentence this section exists to avoid, and it was written once here and withdrawn: what
follows is a per-example table, each row with the evidence that produced it and the date it was measured.

## The matrix, measured 2026-11-14

The rule for a row: the board's own mailbox is **not** evidence. A row is PASS only when something outside the board
(a host capture, a client, or a register read over SWD) shows the effect. `scripts/ble-example-matrix.sh` runs one
example and prints the build range, the mailbox and the host capture together, so the two cannot drift apart.

| Example | State | External evidence |
|---|---|---|
| `ble_beacon` | **PASS** | host `btmon`: `Name (complete): xiao-mg24`, flags 0x06, 14 bytes, `E4:0A:75:13:3E:E0`, -85 dBm |
| `ble_scanner` | **PASS** | hears the host's own advertiser by address: `scan: 90:de:80:81:44:56 rssi=-75 len=13`. **Control**: with the host's advertising off the same image reports a different device (`49:7a:43:ea:f3:f0`) and never that address |
| `ble_advertise_custom` | **PASS** | host decodes the payload: `Company: ... (741)` = 0x02E5, `Data[9]: 01010000010eca3919` -- version, device id, fw 0.1, and a live cycle counter |
| `ble_gatt` | **PASS**, all three operations | A BlueZ client connects, discovers the service and lists **two** characteristics: `6e400002-...` `read,notify` and `6e400003-...` `read,write`. It reads, writes four bytes, reads them back, and subscribes. Measured: **118-243 notifications in 6-12 s** (~20/s, counter incrementing in the value), and the **board stores the client's bytes and echoes them back inside the notification** -- `echo of the written bytes in notifications: 24 PASS`, board record `cv=5aa5d0b8`. The echo is the point: a read-back can be answered from the host's cache, so only bytes the board repeats prove the write crossed the link |
| `module_ble_beacon` | **PASS** | after forwarding the BSP's `stack` feature to `efr32mg24-radio/stack`: `Name: xiao-mg24-bsp` in the host capture, 8 reports, -87 dBm |
| `base_ble_env` | **PASS, but sparse** | env frame on air at the board's address: `Data[9]: 01b50cf503d4192900`; it carries no name by design, so the payload is the evidence. Duty cycle is the problem below: 2 advertisements in an 18 s window |
| `module_ble_motion` | **PASS, but sparse** | board address on air with a 10-byte motion frame: `Data[10]: 01002dff0900f1030604` = version, flag, X `-211`, Y `9`, Z `1009` mg, magnitude `1031`; mailbox `motion: |a|=1029 mg still n=1`. 7 advertisements in a 10 s window |
| `module_ble_fall` | **PASS, but sparse** | board address on air with a 12-byte fall frame: `Data[12]: 0100000000000000fffff803`. 12 advertisements in a 10 s window |
| `hal_led_blink` | **PASS** | PA7 (`0x5003C040` bit 7) sampled over SWD without halting: `0x80 -> 0x00 -> 0x80` -- the LED physically blinks |
| `bt_version`, `hal_mic_level`, `module_mic_level`, `rail_ble_tx`, `rail_ble_rx`, `rail_caps`, `ble_boot_probe` | build-verified | three of them (`bt_version`, `hal_mic_level`, `module_mic_level`) had message-buffer panics that would have hit on first run; fixed, and `scripts/check-copy-len.py` now runs in the gate |

**The instrument was wrong twice in this session, and both times it hid working firmware.** First a host capture with
no self-test (a window in which nothing arrived proved nothing); then a name search, which reported "nothing on air"
for `base_ble_env`, `module_ble_motion` and `module_ble_fall` -- all three were transmitting manufacturer data and have
no name to find. The criterion is now the board's address and its payload, and `scripts/ble-example-matrix.sh` prints
all three, so "no name" cannot be read as "no signal" again.

One firmware defect did come out of that chase, and it was real: `module_ble_motion` wrote a record only when the
motion flag *changed*, so a board sitting still said nothing at all and looked dead. It now writes once before the
loop and once with the first frame.

**A published number that had to be withdrawn: the "duty cycle".** An earlier version of this section reported
`base_ble_env` 2 advertisements in 18 s, `module_ble_motion` 7 in 10 s and `module_ble_fall` 12 in 10 s, and concluded
that restarting the advertising set per update was starving the radio.

That conclusion does not survive a calibration. `ble_beacon` -- the one example whose advertising is known good, a
phone and the host both see it within a second -- measures **2 reports in 11 s** under the same capture. The limit is
the receiver, not the transmitter: the host's dongle is owned by `bluetoothd`, and the scan parameters that would give
a 100% window are answered `Command Disallowed (0x0c)` while it holds the controller. Every one of those counts
measures the scanner's duty cycle.

What survives: an advertising report seen twice **75 ms apart** (beacon) and **76 ms apart** (`ble_advertise_custom`)
shows the interval is in the right order; the per-second totals do not.

**The receiver saturates, and that is measurable.** With `bluetoothd` stopped and the scan parameters set directly
(`LE Set Scan Parameters`, window = interval = 0x30, then `LE Set Scan Enable`), the same window gives the beacon
**19 reports in 11 s** and the neighbours **130**. If the five neighbours around this desk advertise at the usual
100 ms -- 30 packets per second each, since every event goes out on three channels -- the adapter is capturing about
**8% of what is on the air**. Applying that factor, the beacon's 19 reports correspond to roughly 22 packets/s, about
seven advertising events per second, which is what a 100 ms interval produces.

So the counts in this section are a property of the adapter, not of the firmware, and no arrangement of
`hcitool`/`bluetoothctl` commands changes it: `hcitool lescan` overwrites the scan parameters that were just set, and
`LE Set Scan Enable` by hand still leaves the controller capturing a fraction. A rate measured on this bench would need
a second board as the receiver, with its window under this project's control.

That closes the question the previous version of this section got wrong: the advertising is not starved, and the
`-87 dBm` these examples read is the same number the RAIL-only test produced, from a dongle sitting inside a metal case
a few metres away -- not a consequence of a low duty cycle.

## Update 2026-11-14: the security manager links, P-256 works on the board, pairing does not complete

Enabling `SL_CATALOG_BLUETOOTH_FEATURE_SM_PRESENT` and the builtin bonding database made the linker name exactly
seven missing PSA functions:

```
psa_cipher_encrypt  psa_export_key  psa_export_public_key  psa_generate_key
psa_get_key_attributes  psa_mac_compute  psa_raw_key_agreement
```

All seven are now in this crate, in Rust:

| group | implementation | state |
|---|---|---|
| `psa_mac_compute` | AES-CMAC over `libraries/aes-modes` (RFC 4493, tested there) | built |
| `psa_cipher_encrypt` | AES-CTR with PSA's `input = IV \|\| plaintext` convention, over `libraries/aes-ctr-radio` | built |
| `psa_export_key` | copy from the key slot table | built |
| `psa_generate_key`, `psa_export_public_key`, `psa_raw_key_agreement`, `psa_get_key_attributes` | P-256 via the `p256` crate, with an RNG over the SE's TRNG | **verified on the board** |

The boot record says what the firmware can do:

```
gatt: adv waiting a=1 ec1 pk65 dh1 sm1
                     ^^^  ^^^^  ^^^  ^^^
                     |    |     |    security manager configured and bondable
                     |    |     two freshly generated key pairs agree on the same secret
                     |    public key is the 65-byte uncompressed point
                     P-256 key generation works
```

`psa_get_key_attributes` writes a 36-byte structure whose layout is asserted at compile time
(`size_of::<KeyAttributes>() == 36`) and whose two fields the security manager reads -- `type` and `bits` -- sit at
offsets 0 and 2 with no padding possible. That is weaker than measuring `sizeof` against the C compiler: the PSA
headers are not in this repository, so there is no C translation unit to compare against. It is written down as the
weaker claim it is.

**Pairing itself does not complete**, and the board now says exactly where it stops.

Measured with `scripts/host-pair-probe.py` (a D-Bus client that discovers, connects and pairs, because
`bluetoothctl pair` silently does nothing when the device is not in BlueZ's cache -- silence that was twice read as a
firmware fault):

```
pair: org.bluez.Error.AuthenticationFailed (0.2s)     Paired: False   Bonded: False
board: gatt c0 notif=0 refused=0 ev=0 se=4/4616 sm=1 h=010603a0 ua=00000000x0 cv=
                         ^^^^^^^^^^     ^   ^^^^^^^^  ^^^^^^^^^^  ^^^^^^^^^^^^
                         |              |   |         |           no algorithm was ever refused
                         |              |   |         last event: connection_closed
                         |              |   the SM's error code
                         |              event 4 = bonding_failed
                         one security-manager event consumed
```

`4616` = `0x1208` = `SL_STATUS_BT_SMP_UNSPECIFIED_REASON`, matching the `Pairing Failed: 0x08` on the air. Two facts
narrow it: the SM consumes the pairing request (so it is not refusing the request), and **it never calls any of the
seven PSA functions** -- the elliptic-curve failure recorder stays empty and no algorithm is refused. The abort is
therefore inside the security manager's own state machine, before any cryptography, and the cryptography this project
wrote has not been shown to be at fault at all.

Also measured on the way: with `SL_BT_SM_CONFIGURATION_BONDING_REQUIRED` set, the board refuses a peer that does not
request a bond (BlueZ's `pair` sends "No bonding") with the same `0x08` before any exchange. Flags are now `SC_ONLY`
alone; that removed one refusal, not the failure.

Next step, and it needs the blob: find where the security manager returns `0x1208` on the pairing-request path. The
REA dossier in `_tmp/efr32mg24-rs/rea-dossier/` is the tool for it, and the question is narrow -- which state the SM
expects to be different before it will proceed.

A defect found on the way: the key table did not store the handle it handed out, so `psa_destroy_key` and any lookup
guessed `handle n = slot n - 1` -- true only until the first destroy. Slots now carry their id.

## Update 2026-11-13 (night): the scanner works, and the GATT gap is named

`ble_scanner` reports advertisements: `scan: 56:44:81:80:de:90 rssi=-76 len=13 n=4`. Three independent defects sat
between the radio and the application, and the counters are what separated them.

| Claim | Evidence |
|---|---|
| The link layer was asking the radio to receive all along | `diag` `RAIL_ScheduleRx` = **601 calls, every one status 0**, `RAIL_ConfigRxOptions` once -- read without halting |
| The stack was delivering events | `scan: p=41 a=0 id=010500a0` -- 41 events in two seconds, **zero** recognised as advertisements |
| Defect 1: the scanner class was not registered | `SL_CATALOG_BLUETOOTH_FEATURE_SCANNER_PRESENT` added; `sli_bgapi_class_bt_scanner_optimized` appears in the image (count 0 before) |
| Defect 2: the catalogue edit was inert | `build.rs` watched `vendor/` but not `config/`, so the C that reads the catalogue was never recompiled; `rerun-if-changed=config/` is the fix |
| Defect 3: the parser matched the raw header | `BleEvent::parse` compared `evt.header` instead of `header & 0xffff00f8` (`SL_BGAPI_MSG_ID`); the low three bits are flags, so every event fell through the match and `poll_event` returned `None` |
| Fourth defect, exposed by the third | `ble_scanner` panicked at `msg[at..at + 6].copy_from_slice(b" len=")` -- five bytes into six. Nothing had reached that line before |
| Reproducibility defect | `sl_component_catalog.h` is ours and was inside the gitignored `vendor/`; a clone would have built firmware with no advertiser component. It now lives in `config/`, tracked |

`ble_gatt` advertises as `xiao-mg24-gatt`, the host connects (`Connection successful`) and a phone on nRF Connect sees
it. It serves **no attributes**: a client listing characteristics gets zero, because the counter was published on
`CharHandle(0x000B)`, a handle no attribute ever occupied.

| GATT attempt | Measurement |
|---|---|
| A GATT database built in Rust (`gatt_db`, using `sl_bt_gattdb_*`) | with the `dynamic_gattdb` component off: every command answers **15** = `SL_STATUS_NOT_SUPPORTED` |
| Enabling `SL_CATALOG_BLUETOOTH_FEATURE_DYNAMIC_GATTDB_PRESENT`, first try | `sli_bt_system_start_bluetooth` returned **7** = `SL_STATUS_TIMEOUT` and `system_boot` never arrived. Read at the time as "the component cannot work in this build" |
| The actual cause | the feature's configuration instance had the **wrong type**. `bt_dynamic_gattdb_init` is thirty bytes: `ldr r0,[r0]` then `and r1,r0,#1` (GATT caching) and `ubfx r0,r0,#1,#1` (include static database), then `b.w gatt_db_dynamic_init`. It consumes a `sli_bt_gattdb_config_t` (`{ uint32_t flags; }`), not the `sli_bt_dynamic_gattdb_config_t` (`{ uint8_t max_dynamic_gattdbs; }`) that its own header advertises. With the wrong shape, the flags were whatever the byte happened to be |
| After fixing the type (flags 0) | the stack boots, `add_service` returns 0, the client lists both characteristics, and the three GATT operations work |
| Value storage | `max_len: 4/8` gives the stack its own buffer (`sl_bt_gattdb_fixed_length_value`); a `user_managed` value reads back empty, and a notification larger than `max_len` is truncated -- which is how an echo test first reported a write that had in fact arrived |

The example now reports the whole state in one record -- `gatt: adv waiting a=0` -- instead of a second record
overwriting the first, which had hidden the empty database behind a cheerful "advertising, waiting".

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
