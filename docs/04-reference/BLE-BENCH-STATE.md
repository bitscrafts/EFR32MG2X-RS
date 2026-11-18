# BLE on the bench: the assert that cost a night, and its cause

Written 2026-11-17 after the board refused to boot the Bluetooth firmware for roughly two hours, and closed at
22:29 with the fix measured from a cold power-on and end to end over the air. Every number below was measured; the
hypotheses that failed are kept, because a record that only shows the answer teaches nothing.

## The symptom

The product firmware (`module_ble_dosimeter`) would not run. No advertisement, no mailbox record (`magic = 0`, which
the read script reports as *"record still being written"*), and the core found at `RAILCb_AssertFailed`.

```
pc = 0x08007e78   ->  RAILCb_AssertFailed
r1 = 0x0000004b   =   75
```

## The cause

`stack_init::start` called `Sysrtc::enable_clocks`, which gates the SYSRTC's bus clocks and **leaves the counter
stopped**. RAIL's bring-up requires the counter **running** and asserts at its own line 380 otherwise.

The comment above that call had concluded the opposite in an earlier session — *"clocked and disabled is the only
state that satisfies both"* — from a `CFG` write fault at `0x500a_800c`. That fault was our own later re-init writing
`CFG` while the module ran, not RAIL's write, which happens while the counter is still stopped. **A wrong comment in
the code produced a wrong precondition, and the wrong precondition produced this assert.**

The evidence, in the order it arrived:

```
0x2003F500   41535254 0000004b 20011a60 00000001     "ASRT", code 75, handle, count
0x20011dfc   0000017c                                RAIL_AssertLineNumber = 380
0x500A8014   00000001   (after module_clock_probe)   SYSRTC STATUS.RUNNING = 1
0x500A8018   000004 076e                           COUNT counting
```

Notice what `0x2003F500` means: `RAILCb_AssertFailed` is **ours** (`crates/xiao-mg24-bluetooth-sys/src/rail_assert.rs`),
already instrumented, and it writes the code to that buffer before spinning. The information existed all along; nobody
read the buffer.

## The fix

`Sysrtc::ensure_running(cmu)` — enable the clocks, then set `EN` and issue `CMD_START`, each only when not already in
that state, so a warm reset into a running counter is left alone and a cold boot starts it. `stack_init::start` now
calls it instead of `enable_clocks`.

Commit `1373f3f`.

## The proof

```
cold power-on (unplug, replug), no help from any prior firmware:
    ble: seconds=60 windows=3780 published=60 hz=41298389 rms=14

end to end over the air, scripts/ble-end-to-end.sh:
    1. advertisement carries the service UUID      BlueZ: UUID: Vendor specific (e1d0e000-…)
    2. Pairing successful / Bonded: yes
    3. rms 0x0003 = 0c 00 (12)   bands 0x0006 = 8 powers   status 0x0009 = 01 99 09 01 1e
    flags=1 calibrated | source=datasheet | uncertainty 3.0 dB | status 5 bytes
    VERDICT: PASS
```

## The six hypotheses that were wrong, and what refused each

```
1  the bond table is full (MAX_SLOTS = 8)      the probe with CLEAR_BONDS = true also asserted
2  NVM3 corrupt                                 erased to 0xFFFFFFFF with module_nvm3_wipe; asserted the same
3  the firmware version                        800988a (the build that had passed) asserted identically
4  the debugger's soft reset leaves RAIL wedged three physical power cycles did not clear it
5  the phone was connecting and tripping it     phone Bluetooth disabled; same assert
6  a power cycle clears it                      written down, then measured three times, then withdrawn
```

Two more that were not hypotheses but mistakes of method:

- **Halting the core every 12 s** (`wait-for-record.sh`) while the radio ran, *waiting for the fault to heal*. The
  instrument was part of the failure.
- **`probe-rs write` to flash silently does not land** without an erase. It was read back unchanged, which is the only
  reason the mistake was caught; `MSC_ErasePage` from our own example is what actually erases.

## What the board needs now

Nothing special. Flash it and power it on:

```
FLASH_HOST=10.10.10.217 FLASH_USER=mvcorrea bash scripts/flash.sh <elf> --confirm --run
```

The earlier procedure — flash, then power cycle, then pray — is gone, because the precondition it was working around
is now satisfied in code.

## Tools built during the night, each one earning its place

```
scripts/ble-end-to-end.sh    discovery by UUID + bond + the three reads, with a PASS/FAIL verdict
scripts/ble_minimo.py        unfiltered scan, prints every device, connects, reads: no filter and no asserts
scripts/ble_pair_check.py    the validated version of the same walk
scripts/wait-for-record.sh   did the board write anything (do not run it while the radio matters: it halts)
scripts/watch-resets.sh      does the counter restart
crates/xiao-mg24-bsp/examples/module_nvm3_wipe.rs   erase the NVM3 pages only, refusing anything below 0x08006000
crates/xiao-mg24-bsp/examples/module_dosimeter_offline.rs   measure without a radio
crates/xiao-mg24-bsp/examples/module_clock_probe.rs        leaves SYSRTC running — the workaround that proved the cause
```

## What is not claimed

- **dB(A) and dose % are not on the wire.** The NR-15 arithmetic is tested but not wired to the stream; A-weighting
  needs the per-band sum, not the broadband level.
- Calibration is **from the datasheet** (−38 dBV/Pa, DC 0.7 V, cross-checked against the measured bias at 0.9 %), not
  from a calibrator. Uncertainty ±3.0 dB, and the app says so on every screen.
- Nothing here is a certification.
- The BLE link was proven **from the host** (`gatttool` at `sec-level=medium`, BlueZ). The phone's own run is the next
  measurement, not a conclusion of this document.

## Two traps that cost time after the fix, both operational

**The board may be running an example, not the product.** Measured 2026-11-17: the app could not find the dosimeter
while `bluetoothctl` showed the address advertising as `xiao-pair-probe`. A probe or any other example left on the
board does not carry the service UUID in its advertisement, so an app that filters by that UUID — which is the generic
way, and the one we want — cannot see it. Before every handover: flash the product, then confirm the advertised name
and UUID with `bluetoothctl info`.

**The user LED is not a "firmware running" indicator.** `module_ble_dosimeter` never drives it; the only code that
touches the LED is the bond-clear path. An LED that is off means nothing about the firmware, and the LED that is on
during charging is the charger's. The instrument is the mailbox record and the advertised name.

## The app defects found on a real phone (same night, same shape)

All three were the same failure mode — a step that fails without saying so:

```
never paired                        the app subscribed without ever calling createBond, and with the
                                    characteristics requiring a bond the subscription would fail silently
paired but the app did not know     the receiver was RECEIVER_NOT_EXPORTED and ACTION_BOND_STATE_CHANGED comes
                                    from the system Bluetooth app, so the event never arrived; the device was
                                    listed as paired in the phone's own settings while the app said "pairing"
connected but no values             Android allows one pending GATT operation per link: three reads (and
                                    three CCCD writes) in a burst meant the first was accepted and the other
                                    two returned false with no visible error
```

The fixes, in `dosimeter-mg24`: pair before subscribing; register the receiver exported and also poll
`device.bondState` every 400 ms; one GATT operation at a time, each callback asking for the next, with a two-second
guard. The general rule this night produced: **prefer reading a state to waiting for an event, and never leave a
failure path without a line on the screen.**

## Why the phone is asked to pair again after the dosimeter is power-cycled

Measured 2026-11-17, reported by the operator as *"quando desligo esta pedindo sempre para emparelhar novamente"*,
and located in our own code:

```
crates/efr32mg24-bluetooth-sys/src/bonding_ext.rs:585
    sli_bt_external_bonding_resolve_address(address, address_type) -> 0     // always "not mine"
```

A bonded phone presents a **Resolvable Private Address**. Before it can match the connection to a stored bond, the stack
asks the external bonding database to resolve that address — and this layer answers "no" every time, by design, with a
comment saying the honest thing rather than guessing which record piece holds the key. The consequence is exactly the
symptom: after a reboot the device does not recognise the phone, so the stack creates a new bond and the phone shows a
pairing prompt.

What the record says, measured across two resets:

```
before reset   cfb=2 cfok=2 cfno=0 smfail=1 smrsn=4102
after reset 1  cfb=1 cfok=1 cfno=0 cfwhy=2      new bond created, inside the window
after reset 2  cfb=0 cfok=0 cfno=0              no bond requested -- the phone had not reconnected yet
NVM3 header    b29a0001 d0000001 -> d0000002    writes are landing; the store is not the problem
```

`4102 = 0x1006` is the SMP reason the phone reports on the failed attempt. So the bond is written and readable; what is
missing is **address resolution**.

### The fix, and why it is not a five-minute patch

Resolving an RPA needs the peer's IRK and the `ah` function (AES-128 over `prand`, top 24 bits compared with the hash),
plus knowing **which piece of the bonding record holds the IRK** — and that layout is not documented anywhere in the
vendor tree; the comment above the stub says so. Finding it is a read-only experiment: dump every piece of one stored
bond after a pairing and identify the 16-byte key. Then `aes-modes` already has ECB, and the resolution itself is small.

Until that is done, the demonstration-safe procedure is: **pair once and do not power-cycle the dosimeter afterwards.**
Resetting the phone's app is harmless; resetting the board costs a re-pair. `CALLS_RESOLVE` already counts these calls
in `bonding_ext.rs` but is not published anywhere yet — exposing it in the record is the first step of the fix, because
it turns "the stack probably calls this" into a number.

## The bond is not held: 0x1006, and the plan that makes the fix finite

Measured on the board at the end of the night, with the phone trying to reconnect:

```
cfb=9  cfok=9  cfno=0  smfail=7  smrsn=4102
4102 = 0x1006 = SL_STATUS_BT_CTRL_PIN_OR_KEY_MISSING
       (vendor/bluetooth/sl_status.h:218: "Pairing failed because of missing PIN, or authentication
        failed because of missing Key")
```

Read as a sequence: the phone tries to encrypt with the key it kept, the device cannot find that key, the
link fails with `0x1006`, the phone drops its bond and pairs again — nine accepted bonds and seven failed
authentications in one boot. The reason the device cannot find the key is the one already written above: the
phone presents a Resolvable Private Address and `sli_bt_external_bonding_resolve_address` answers "not mine"
every time, so every connection looks like a new peer.

### The plan, so the next session does not start from zero

1. **Learn what the pieces are, from the stack itself.** `sli_bt_external_bondingdb_data_request_event_id`
   (`sli_bt_api.h:644`) tells us which piece the stack wants, and `bonding_ext::note_data_request` already
   records those calls. Log the piece index and the length for one full pairing, then dump the same pieces:
   the IRK is the 16-byte piece that is neither all zeros nor equal to the identity address.
2. **Implement `ah`.** `ah(IRK, prand)` is AES-128 of `prand` with the IRK; the top 24 bits are compared with the
   hash part of the address. `libraries/aes-modes` already carries ECB, so this is a small function with the
   Core Specification vectors as tests (three published vectors are enough to catch a wrong byte order).
3. **Fill in `resolve_address`**: walk the stored bonds, fetch the IRK piece, run `ah`, return the matching
   handle. It must return `0xFFFFFFFF`/0 for "no match" and never guess.
4. **Prove it where it can fail**: pair once, power-cycle the board, and check that the first connection after
   the reboot reports `sec=1` with `cfb=0` — no new bond created. That is the measurement that closes this.

Until then, the demonstration-safe rule stands: pair once and do not power-cycle the dosimeter.

## The actual cause of the re-pairing: two callbacks we never defined

Found by reading the library's own undefined symbols instead of guessing:

```
arm-none-eabi-nm libbluetooth_host_efr32xg24_gcc_release.a | grep "U sli_bt_external_bonding"
    sli_bt_external_bonding_alloc                 defined by us
    sli_bt_external_bonding_bonding_data_ready   NOT defined by us
    sli_bt_external_bonding_get_database_type    defined by us
    sli_bt_external_bonding_is_bonded            defined by us
    sli_bt_external_bonding_request_bonding_data NOT defined by us
    sli_bt_external_bonding_store_data           defined by us
```

The host asks our database for a stored record through `request_bonding_data`, and expects to be answered through
`bonding_data_ready`. We define neither, so the vendor's weak stubs answer instead — and the stack never receives the
record it needs to read the peer's IRK. Without the IRK it cannot resolve the phone's private address, so every
connection looks like a new peer: a new bond is created, the phone's key no longer matches, and the link dies with
`0x1006` (see above). That is the whole re-pairing story, and none of it is the phone's fault.

Two corollaries worth writing down, because both cost time tonight:

- **`sli_bt_external_bonding_resolve_address` (ours, `bonding_ext.rs:585`) is dead code.** It is not in the host's
  undefined list; the host resolves addresses itself with the data we are supposed to hand it. Its `return 0`
  (instead of the vendor's `0xFFFFFFFF` sentinel, visible in
  `objects/bondingdb-stubs.c.obj`: `mov.w r0, #4294967295`) is therefore not the bug — but it is a trap for whoever
  reads it next.
- The plumbing to fix it is already in the file: `note_data_request`, `queue_data_answer`,
  `take_queued_data_answer`. What is missing is the two `extern "C"` entry points that the host links against.

### The fix, concretely

```
sli_bt_external_bonding_request_bonding_data(connection, bonding, data_type)
    -> read the piece from NVM3, then answer with sli_bt_external_bonding_bonding_data_ready(bonding, len, data)
sli_bt_external_bonding_bonding_data_ready(bonding, len, data)
    -> the answer path; the host keeps the buffer until it has consumed it
```

Disassemble `objects/bondingdb-stubs.c.obj` for the exact argument counts and return conventions before writing the
bodies, and keep the vendor's sentinel values. Then prove it where it fails: pair once, power-cycle the board, and
read `cfb=0` with `sec=1` on the first connection afterwards.

## 2026-11-18, end of night: the board does not advertise, and what is unverified

**State measured.** After the NVM3 wipe (`module_nvm3_wipe`) and a verified flash of the product, the board stops
writing its mailbox (`record still being written after 5 attempts (core halts mid-write)`) and does not appear in
`hcitool lescan`. The same message appears with `module_dosimeter_offline`, which has no radio at all, so the halt
message is not evidence about the BLE stack: it is what the probe says when it cannot stop the core.

**What is NOT verified, and must not be presented as working.** `SECURITY_REQUIRED = false` in
`module_ble_dosimeter.rs` is committed to remove the pairing dialog: the device cannot resolve the phone's private
address, so every connection ended in `0x1006 PIN_OR_KEY_MISSING`, the phone dropped its bond and asked to pair again.
With the switch off there is no `increase_security` and no security demand on the characteristics. **This change has not
been seen running on hardware** — it was flashed four times and the board never wrote a record afterwards.

**First thing next session, in this order.**
1. Power-cycle the board by hand (remove the USB cable, not a `probe-rs reset`): the probe keeps the core halted in a
   way a reset does not clear, and every earlier recovery tonight needed a physical replug.
2. Read the mailbox. A `ble: seconds=…` line means the image runs; no line means the image is bad, not the radio.
3. Only then decide about `SECURITY_REQUIRED`. If the image runs with it false, the demo is unblocked and the RPA work
   below is what restores `true`.

**The RPA work, now with the map found in the vendor header.** `sl_bt_api.h` documents the external bonding data types
and they are NOT the `sl_bt_sm_bonding_data_t` indices: the external enum inserts `remote_address_type` at 1, so the IRK
is type **6**, not 5. `piece_for` uses the type as the NVM3 key and round-trips consistently, so storage is not the
defect; finding the bond for a private address is. `sli_bt_bonding_resolve_address` in the host compares against the
local identity address, and `sli_bt_bonding_find_by_address` is a weak stub that answers `0xFFFFFFFF` — the peer lookup
has nowhere to land. That is the function to fill, with `ah(IRK, prand)` over the stored IRK.

## 2026-11-18: verified after a physical replug -- the switch works

The replug was the missing step, as the note above predicted. Read from the board:

```
ble: seconds=300 windows=18900 published=300 notifies=277 cycles=... hz=42122628
     sreq=255 sec=1 mtu=247 cfb=1 smfail=0 smrsn=0 rms=14
```

- `sreq=255` is the sentinel this code writes when `SECURITY_REQUIRED` is false: no security request was made, so the
  phone has nothing to prompt about.
- `smfail=0 smrsn=0` — no pairing failure at all in this session, against `smfail=7 smrsn=4102` when the device was
  demanding a bond it could not resolve.
- `notifies` climbing (277 → 337 in twelve seconds) means a client is subscribed and consuming: the app is receiving.
- `sec=1` is the existing bond still encrypting the link; it is not being requested, it is already there.

**Conclusion, measured, not assumed:** the pairing loop is closed for the prototype by removing the demand, and the only
thing that was wrong with the previous four flashes was the core left halted by the probe. `probe-rs reset` did not
clear it; unplugging the cable did, every time.

**What this does not settle:** the data still flows unencrypted to any client in range, and the RPA resolution below is
what restores `SECURITY_REQUIRED = true`.
