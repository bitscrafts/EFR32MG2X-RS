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
