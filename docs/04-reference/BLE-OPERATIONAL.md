# BLE: what "fully operational" means here, and what is measured (2026-11-17, evening)

The requirement, in the user's words: *"a phone pairs once, and thereafter — including after every reset of the MCU —
connects and reads the dose without the user doing anything."* This file records what is now measured against it, and
what is still open. Every number was taken on the board, product example (`module_ble_dosimeter`), external bonding
database, at 80 MHz with the core divided for PCLK and the radio as the vendor requires.

> **Retracted in part, 2026-11-17 (night): "at 80 MHz".** The core frequency was never read at the time of these
> captures; the same figure (`19_000_000`) sat unread in three places in the tree (commit `6f51ed8`, body). Measured
> afterwards, against the 16 kHz sample timer, the core ran at `hz=41151471` (commit `6f51ed8`, body; see "Tonight's
> three changes" below). I cannot say from the tree whether the clock configuration changed between the two
> measurements, so the 80 MHz claim is withdrawn, not replaced. The per-minute cycle counts in the capture block below
> are unaffected as counts; any percentage derived from them was not.

## Measured, in the order that matters

```
clean pairing                'Paired: yes, Bonded: yes, Connected: yes', first attempt
                             device: sec=1 (encrypted), mtu=247, windows=3780, published=60
reset, no pairing            host: Connection successful / Bonded: yes / Connected: yes
                             device: sec=1 (encrypted), mtu=247, windows=3780, published=60,
                                     dfail=0, disc=0, adv=0
notifications              64 in 55 s on handle 0x0003, values 0x0043/0x0044/0x003f
capture with the stack on   windows=3780 published=60, cycles 1 189 930 746 per minute
                            (2 350 795 993 before the PCLK divider -- the divided bus is cheaper for this loop)
```

`sec=1` is the load-bearing number. It is the **device's** statement about the link: 0 none, 1 unauthenticated
encryption (what Just Works gives), 2 authenticated, 3 authenticated Secure Connections. A host printing
`Bonded: yes` proves only that the host stored a bond; it prints that whether or not the link is encrypted. Everything
before today's last change was measured without it, and the link was open.

## What made it work, in the order it was found

```
1  heap           psa_reset_key_attributes wrote 232 bytes into a 64-byte block
2  arguments      store_data/get_data had the last two arguments swapped against the caller
3  bookkeeping    no connection -> peer table, so a bonding-data request named no bond
4  answer         every request was answered with zero length: encryption from stored data impossible
5  clearing       the stack cannot delete with an external database; the store is ours
6  gatt cache     types 8/9/10 answered with stale data -> INVALID_PARAMETER, conversation stalled
7  identifiers    the stack names keys (0x48000 | handle<<8 | type) and we assigned our own
8  persistence    the id was squeezed into a 20-bit NVM3 key; 0x48003 came back as 0x8003
9  pump           the stack was serviced once per second; pairing needs it per sample
10 clocks         PCLK and the radio clock were undivided and out of spec above 50/40 MHz
11 encryption     nothing asked for it, so the link stayed bonded and open
12 lifetime       the example stopped measuring after 60 s and stopped advertising on a disconnect
```

## The build the product needs

`module_ble_dosimeter` declares `required-features = ["stack", "sense", "external-bonding"]`, so a plain
`cargo build --example module_ble_dosimeter` refuses with
``requires the features: `stack`, `sense`, `external-bonding` `` instead of producing a device that pairs, stores the
bond, and then cannot find the key after a reset. That is not a preference: the stack's own bonding database loses the
key across a reset (measured, `rsn=0x1006` with the key present in flash), which is why the external database exists in
this project at all.

## Tonight's three changes (2026-11-17, night)

Each item says where its numbers come from. "Brief" is `_tmp/brief-docs-night.md`, the instruction this section was
written from. "Header" is `crates/efr32mg24-bluetooth-sys/vendor/bluetooth/sl_bt_api.h` (byte-identical to
`vendor/bt_stack/include/sl_bt_api.h`, checked with `cmp`). Commits: `c42a11c`, `6f51ed8` (in git); the permission
change is **uncommitted** in the working tree (`git status`: `gatt_db.rs`, `module_ble_dosimeter.rs` modified).

### 1. The characteristics now require a bond

```
before   crates/efr32mg24-bluetooth-sys/src/blocking/gatt_db.rs passed 0 as the security requirement of every
         characteristic, commented "an unpaired client may use it"            (git diff, removed lines)
now      the three dosimeter characteristics declare  SEC_BONDED_READ | SEC_BONDED_NOTIFY = 0x2 | 0x80
         (module_ble_dosimeter.rs:419; constants in gatt_db.rs)
```

Header lines, opened: `SL_BT_GATTDB_BONDED_READ 0x2` is `sl_bt_api.h:8921`, `SL_BT_GATTDB_BONDED_NOTIFY 0x80` is
`sl_bt_api.h:8943`. The brief cites `8922` and `8939`; **neither matches**: line 8922 is blank and line 8939 is
`SL_BT_GATTDB_ENCRYPTED_NOTIFY 0x40`, a weaker requirement than the one declared. The values `0x2` and `0x80` are right;
the line numbers in the brief, and in the doc-comments in `gatt_db.rs`, are a few lines off. Not fixed here (code is out
of scope tonight).

The pair of measurements asked for, honestly:

```
read of handle 0x0003 AFTER pairing    returns  0e 00          source: the brief. The capture log is not in the tree
                                                               and I did not open it. A read of 0x0e 0x00 is the u16 RMS 14.
read of handle 0x0003 WITHOUT a bond   NOT CAPTURED AS AN ATT ERROR
```

For the unpaired case there is no capture I can open. `scripts/security-read-probe.sh` (untracked) exists to take it
with `gatttool --sec-level=low`; its own header records that "the first version of this script printed nothing for the
unpaired read and left 'no output' indistinguishable from 'refused'". So the most this document can say is that the
check was attempted and **its output is not available here**. It does not show the stack answering Insufficient
Authentication, and "no output" must not be read as "refused". Until that line exists, the claim is: *the declaration is
in the code; the refusal is unmeasured.*

### 2. The device records its own pairing failures

```
before   sreq=255 sec=0 disc=0 adv=0                              reads as "nobody tried"
now      sreq=2   sec=0 smfail=1 smrsn=4613 disc=1 adv=1          orphan case: device holds a bond, phone forgot it
```

Source: commit `c42a11c` (body), measured "twice, with the host bond removed and the device's kept"; the host side of
the same runs said `org.bluez.Error.AuthenticationFailed`. `smfail=` counts `sl_bt_evt_sm_bonding_failed`, `smrsn=`
carries the SMP code. `4613 = 0x1205` (arithmetic checked: `python3 -c "print(hex(4613))"`).

**Correction to the name.** The commit body and the brief call `0x1205` `SL_STATUS_BT_SMP_AUTHENTICATION_FAILURE`. The
vendored `sl_status.h` has no such symbol (`grep -rn SMP_AUTHENTICATION_FAILURE crates/` finds nothing). In the header
`0x1205` is `SL_STATUS_BT_SMP_PAIRING_NOT_SUPPORTED` ("Pairing is not supported by the device", `sl_status.h:297`); the
authentication-flavoured codes are `0x1203` `..._AUTHENTICATION_REQUIREMENTS` and `0x1204` `..._CONFIRM_VALUE_FAILED`.
The number is measured; its *name* in the commit is wrong against the header. What the code means for the orphan case is
therefore: the exchange failed with SMP reason `0x1205` ("pairing not supported") -- I do not know from the tree why the
stack picks that code when the peer has lost its key, and the commit's reading ("the exchange failed against a key the
peer no longer has") is the commit author's, not something the header states.

### 3. The clock and the loop's cost are measured

Source for every number: commit `6f51ed8` (body), measured on the board with the 16 kHz sample timer as reference.

```
hz=41151471      core frequency, against the timer (the tree carried 19_000_000 in three places, never read)
ps=2571          elapsed cycles per sample; 2571 x 16 000 = 41 136 000, which is hz by construction -- it measures
                 nothing, and the commit says so
work=4880000     cycles per second that are work rather than the paced wait
cpu=118          11.8 % of the core   (4 880 000 / 41 151 471 = 0.1186, my arithmetic, agrees)
```

The stack poll moved from once per sample (16 128 polls/s, 929 cycles each -- commit body) to once per 16 samples
(`POLL_EVERY_SAMPLES 16`, `module_ble_dosimeter.rs:61`, 1 000 polls/s). My arithmetic from the commit's figures, not a
new measurement: 16 128 x 929 = 14 982 912 cycles/s, 36.4 % of `hz` ("a third", as the commit says); over a minute that
is 898 974 720 against 55 740 000 after, a saving of 843 234 720 -- the commit's "843 million cycles per minute".
Measured after the change, per the commit: pairing on attempt 1, `sreq=2 sec=1 mtu=247`, 64 notifications in the wait
window, `windows=3780` per period sustained to 180 s.

## Still open, in the order I would take them

```
A  orphan bond      device holds a bond, phone forgot it -> the phone's pairing is answered with the old key and
                    cancelled. The button-hold-at-boot path exists but is unverified on hardware, and the policy
                    (who clears, when) is a product decision.             plan step 9
   [UPDATED 2026-11-17 night: the failure is now recorded by the device (`smfail=`/`smrsn=`, commit `c42a11c`) and
    the policy is decided -- see PAIRING-WINDOW.md. The mechanism is in the working tree, uncommitted, and has not
    been measured on the board. Still open: the measurement.]
B  permissions      the characteristics still declare no security requirement, so an unpaired client can read them
                    if the link is not encrypted. The link is encrypted now because the device asks; the attribute
                    permissions should make that a rule rather than a habit.     plan step 7
   [RETRACTED 2026-11-17 night: "the characteristics still declare no security requirement" is no longer true in the
    working tree -- the three dosimeter characteristics declare `SEC_BONDED_READ | SEC_BONDED_NOTIFY`. Uncommitted;
    the unpaired-client half is not captured as an ATT error. See "Tonight's three changes", 1.]
C  RPA              a phone using a resolvable private address needs the peer IRK path; unmeasured, and its
                    prerequisite measurement is named in the plan.                plan step 6
D  sleeptimer       64-bit wrap at 36.4 h, accuracy reported as 500 ppm while the code records 6000. plan step 8
E  notifications    subscribed-only with sequence numbers and no blocking; `dfail=` is the counter to watch.
                                                                                 plan step 10
F  orphan eviction  one bond per identity, evict when full.                        plan step 9
```

The plan these letters refer to is `_tmp/ble-plan/PLAN.md` (12 ordered steps, written by the analysis model), and
`_tmp/ble-plan/IMPLEMENTED.md` records which steps the implementation model completed and what it could not verify.

## Endurance, one power-up (measured 2026-11-21)

The product ran **4680 seconds (78 minutes) on a single boot**, publishing one record per second throughout:

```
ble: seconds=4680 windows=294840 published=4680 notifies=0 cycles=2597629022 hz=na ps=na work=na cpu=na
     polls=4717440 pollcyc=63766168 perpoll=13 timer=1 ovf=0 rms=19 b0=77431691 sreq=255 sec=0 mtu=23 cfb=0
     smfail=0 smrsn=0 cure=0 cfok=0 cfno=0 cfwhy=0 disc=0
```

`seconds` and `published` agree (`4680` each), so no record was skipped and the loop never restarted -- a reset
would have put `seconds` back to 0. `disc=0` over the same window: no disconnect, and therefore no advertising
restart to get wrong. `windows=294840` is `4680 x 63`, the sampling side keeping its rate for the whole run.

`hz=na` in that record is the instrument, not the product: reading the mailbox over SWD clears `DEMCR.TRCENA`,
which stops the cycle counter (see `QUEUE.md` B17 and `/projects/docs/01-overview/DO-NOT-REPEAT.md` §11). The fields
that come from counting -- `seconds`, `windows`, `published`, `polls`, `disc` -- are unaffected by the read.

What this record supports: the product samples, publishes and advertises without a restart for at least 78 minutes.
What it does not support: anything about a connection, because no phone was attached for the window (`notifies=0`,
`sec=0`, `mtu=23`).
