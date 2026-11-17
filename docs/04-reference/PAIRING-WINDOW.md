# The pairing window: when the device accepts a new bond (decided 2026-11-17, not yet measured)

**Status in one line:** the policy is decided and the code for it is in the working tree; it is **uncommitted, has not
been built by me, and has not been run on the board**. Everything below about what the device *does* is read from the
code; everything about what it *measured* is limited to the orphan failure that motivated it. Where the two differ I say so.

Sources used, all opened for this document:

```
brief     /projects/efr32mg24-rs/_tmp/brief-docs-night.md          the instruction, and the decision's wording
code      crates/xiao-mg24-bsp/examples/module_ble_dosimeter.rs    (uncommitted: git status shows it modified)
          crates/efr32mg24-bluetooth-sys/src/blocking/stack.rs     bonding_confirm()           (uncommitted)
          crates/efr32mg24-bluetooth-sys/src/bonding_ext.rs        any_bond()                  (uncommitted)
          crates/efr32mg24-bluetooth-sys/src/blocking/events.rs    BleEvent::ConfirmBonding
header    crates/efr32mg24-bluetooth-sys/vendor/bluetooth/sl_bt_api.h   (identical to vendor/bt_stack/include/, cmp)
commit    c42a11c   the orphan measurement
probe     scripts/pairing-window-probe.sh                          (untracked; covers cases 2 and 3 only)
```

## 1. The problem it solves

Two failures pull in opposite directions, and the policy is the line between them.

**A device that keeps a bond the owner's phone has forgotten can never be re-paired.** Measured on the board, commit
`c42a11c` (body), twice, with the host bond removed and the device's kept:

```
host     "Failed to pair: org.bluez.Error.AuthenticationFailed"
device   before the counter   sreq=255 sec=0 disc=0 adv=0                          reads as "nobody tried"
         with the counter     sreq=2   sec=0 smfail=1 smrsn=4613 disc=1 adv=1      the device's own record
```

`4613 = 0x1205` (`python3 -c "print(hex(4613))"`). The commit and the brief name that code
`SL_STATUS_BT_SMP_AUTHENTICATION_FAILURE`; the vendored `sl_status.h:297` names `0x1205`
`SL_STATUS_BT_SMP_PAIRING_NOT_SUPPORTED`, and no symbol called `SMP_AUTHENTICATION_FAILURE` exists in `crates/`
(`grep -rn`). I use the number, which is measured, and not the name, which is not backed by the header. The same
correction is recorded in `BLE-OPERATIONAL.md`.

**A device that accepts any bond can have it taken by anyone in radio range.** This one carries a person's noise
exposure (the three dosimeter characteristics; `BLE-OPERATIONAL.md`, "Tonight's three changes", 1). That half of the
argument is reasoning from the Bluetooth pairing model, not a measurement: nothing in the tree demonstrates an attack.
The commit body states the same trade-off in the same terms ("accepting a new bond on authentication failure would let
anyone in radio range take the bond ... refusing it leaves a device that cannot be re-paired until its bonds are wiped").

## 2. The policy

Decided 2026-11-17 (brief; the same date is in the code comment on `PAIRING_WINDOW_SECONDS`):

```
accept a new bond   when the device holds no bond at all
                    OR  within 600 s of power-on
otherwise           refuse to store a bond for this pairing
```

`120` is `PAIRING_WINDOW_SECONDS` (`module_ble_dosimeter.rs:54`). The code's own justification, a comment and not a
measurement: long enough to take a phone out and pair it, short enough that an attacker must be present when the device
is switched on.

What "within 600 s of power-on" means in the code: `inside_window = boot_second < PAIRING_WINDOW_SECONDS` (the constant is **600** since 2026-11-17)
(`module_ble_dosimeter.rs:301`), where `boot_second` is the loop's `second` counter, handed to `service_stack`
(`:711`, `:797`). That counter advances once per paced second of audio samples; it is **not** a hardware wall clock and I
have not checked it against a stopwatch. The loop's cost is measured (`cpu=118`, `BLE-OPERATIONAL.md`), which says it
has headroom to keep pace, not that its seconds are 120.000 real ones.

"Holds no bond at all" is `efr32mg24_radio::any_bond()` (`:300`), defined in `bonding_ext.rs:430`: it asks NVM3 whether
piece 8 exists for each of the `MAX_SLOTS = 8` slots (`bonding_ext.rs:33`) and returns true on the first that does.

Outside the window the link still works for that connection; it just does not become a bond (brief). That is the
intended behaviour; whether the stack really keeps the link up after a refusal is one of the unmeasured items in section 6.

## 3. The mechanism

```
flags   enable_pairing_with(0x84, 0x03)  ->  enable_pairing_with(0x8C, 0x03)      module_ble_dosimeter.rs:542
        the change is exactly one bit: 0x08 = SL_BT_SM_CONFIGURATION_BONDING_REQUEST_REQUIRED
        (sl_bt_api.h:10801; the doc comment above it: "Bonding requests need to be confirmed. Received bonding
         requests are notified by sl_bt_evt_sm_confirm_bonding")
event   sl_bt_evt_sm_confirm_bonding   -> BleEvent::ConfirmBonding { connection, bonding_handle }    events.rs:97, :325
answer  sl_bt_sm_bonding_confirm(connection, confirm)        sl_bt_api.h:11243
        confirm 0 = Reject, 1 = Accept bonding request      sl_bt_api.h:11237-11238
        wrapper: Bluetooth::bonding_confirm(connection, accept)          stack.rs:713
```

The decision, as the code makes it (`module_ble_dosimeter.rs:296-318`):

```
no bond held            accept = 1, cfwhy = 1
bond held, inside 600 s accept = 1, cfwhy = 2
bond held, past 600 s   accept = 0, cfwhy = 3
```

Two things in that code differ from the picture the brief paints, and a reader should know them:

1. **Refusal sends no answer.** The code calls `bonding_confirm` only when `accept == 1`
   (`if accept == 1 && ble.bonding_confirm(connection, accept) == 0 { cfok += 1 } else { cfno += 1 }`). In the refusal
   case it never calls `sl_bt_sm_bonding_confirm(connection, 0)`; it increments `cfno` and leaves the event
   unanswered. The header says what `0` means; it does not say what the stack does with a confirm request nobody
   answers. That is not established by anything I opened, and it is the step the measurement must cover. `cfno` also
   counts an *accept that failed* (a non-zero status from the call), so `cfno > 0` alone does not mean "refused by policy" --
   `cfwhy = 3` does.
2. **The bonding handle is not consulted.** The event carries `bonding_handle`, and the header
   (`sl_bt_api.h:10981-10987`) says that anything other than `0xff` means a bonding already exists for this connection
   and that overwriting it is a potential security risk. The code ignores it (`ConfirmBonding { connection, .. }`) and
   uses `any_bond()` instead. Whether the handle is meaningful with the external bonding database is not something the
   header settles (see section 6).

**The flag byte, against the header.** The comments in `module_ble_dosimeter.rs:534-541` describe `0x84` and `0x8C` as
"bonding required + MITM". The header's bit table (`sl_bt_api.h:10790-10816`) reads differently:

```
0x01 MITM_REQUIRED          0x10 CONNECTIONS_FROM_BONDED_DEVICES_ONLY
0x02 BONDING_REQUIRED       0x20 PREFER_MITM
0x04 SC_ONLY                0x40 OOB_FROM_BOTH_DEVICES_REQUIRED
0x08 BONDING_REQUEST_REQUIRED   0x80 REJECT_DEBUG_KEYS
```

By that table `0x84` is `SC_ONLY | REJECT_DEBUG_KEYS` and `0x8C` adds `BONDING_REQUEST_REQUIRED`; neither sets `MITM_REQUIRED`
or `BONDING_REQUIRED`. I have not run anything that tests which reading is right, so I only record the disagreement:
the **0x08 bit is what the window depends on** and it is correct in either reading, but the code comments' words
("MITM", "bonding required") do not match the header's names for those bits. It does not change section 5: with
`io_capability 0x03` (No Input No Output, `sl_bt_api.h` `sl_bt_sm_io_capability_noinputnooutput`) the pairing is Just
Works either way.

## 4. What the owner does to re-pair

**Power-cycle the device, then pair within ten minutes.** After a reset the loop's counter starts again, so
`boot_second < 120` is true and a new bond is accepted even though the device still holds the old one (`cfwhy = 2`).
That is the whole gesture; there is no button path in this policy (the older "button-hold-at-boot" path in
`BLE-OPERATIONAL.md` item A is a different, unverified mechanism).

Consequence I can see from the code and have not measured: re-pairing inside the window adds a bond, and nothing here
removes the old one. With `MAX_SLOTS = 8` that is a finite number of re-pairs; "one bond per identity, evict when full"
is item F of the open list in `BLE-OPERATIONAL.md`.

## 5. What it does not protect against

**Just Works has no man-in-the-middle protection, so a pairing inside the window can still be an attacker's.** The
window limits *when* a bond can be created, not *who* creates it. The code comment on `PAIRING_WINDOW_SECONDS` says the
same ("It is **not** authentication"). Concretely, a person within radio range at the moment the device is switched on,
or within 120 s of it, can pair; so can anyone, at any time, if the device holds no bond (`cfwhy = 1`). The window is a
reduction of the exposure, not a lock. A passkey or numeric comparison would be stricter; the code comment records that
this product has no display to show one on.

It also does not protect the data of a connection that is already bonded; that is the separate permission change
(`SEC_BONDED_READ | SEC_BONDED_NOTIFY`), whose refusal for an unpaired client is itself **not yet captured as an ATT error**
(`BLE-OPERATIONAL.md`, "Tonight's three changes", 1).

## 6. The counters that say it worked, and what is open

Printed in the mailbox line (`module_ble_dosimeter.rs:640-649`):

```
cfb=    the stack asked        incremented on every BleEvent::ConfirmBonding            (:248)
cfok=   accepted               the confirm call was made with 1 and returned 0          (:312-313)
cfno=   refused                no confirm sent, OR the confirm call failed              (:315)
cfwhy=  the rule behind the last answer: 1 no bond held, 2 inside the window, 3 outside with a bond   (:204-208)
smfail= / smrsn=   the pairing failure the device recorded, from commit c42a11c
```

**The question that decides whether any of this runs:** a device whose `cfb` stays `0` with the flag set means the stack
does not raise `sl_bt_evt_sm_confirm_bonding` with the external bonding database -- every pairing would be accepted
silently, which is the state before this change. The code comment (`:298-299`) and the brief both flag it as the open
question of the plan's step 9. The one thing the header says on the subject is indirect: `sl_bt_api.h:11598-11604` lists the
events whose `bonding` field is forced to `SL_BT_INVALID_BONDING_HANDLE (0xff)` under the external database; it lists
`sm_bonded` but **does not list `sm_confirm_bonding`** (opened, not inferred), so the header neither says the event is
raised nor that it is not. The brief cites `11597-11603`; the list opens at 11598 and its last line is 11603-11604.

What would settle it, and what has and has not been done:

```
done     the code is written and the stack/ext functions exist (git status: stack.rs, bonding_ext.rs, module_ble_dosimeter.rs modified)
done     scripts/pairing-window-probe.sh is written (untracked): case 2 pairs inside the window, then waits 130 s,
         case 3 removes the host bond and pairs again outside it. EXPECTED, not observed: a refused bond leaves the host
         with Bonded: yes = 0 and the device with cfwhy=3, cfno=1.
NOT done a run of that script, or any capture of cfb/cfok/cfno/cfwhy from the board
NOT done case 1 (no bond held) -- the probe says it needs a separate flash with the probe's CLEAR_BONDS
NOT done what the stack does with an unanswered confirm request (section 3, point 1)
```

Until a capture exists the honest summary is: **the policy is decided, the mechanism is written, and no number in this
file says it works.**

## Why the window is ten minutes and not two (changed 2026-11-17)

Two minutes was measured as too short on the bench: the operator powers the board, opens the app, and by the time the
phone has scanned, connected and asked to bond, the window is gone. A refusal at that point is invisible on Android —
the peripheral declines inside SMP, the phone's bond state never leaves `NONE`, so `ACTION_BOND_STATE_CHANGED` never
fires and the app has no event to show; it sits on "pairing" with nothing to report, which reads exactly like a hang.

Ten minutes keeps the property that matters — a bond can only be created while somebody is physically present at
power-on — and removes the failure the demonstration kept hitting. The mechanism, the counters (`cfok`, `cfno`,
`cfwhy`) and the refusal path are unchanged; only `PAIRING_WINDOW_SECONDS` moved.

The companion change is on the app side: when the device refuses, the app now **drops the link**. A connected
peripheral stops advertising, so every attempt after a refusal used to find nothing to scan and reported "the
dosimeter is not there" while it was advertising again a second later.
