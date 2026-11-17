# BLE: what "fully operational" means here, and what is measured (2026-11-17, evening)

The requirement, in the user's words: *"a phone pairs once, and thereafter — including after every reset of the MCU —
connects and reads the dose without the user doing anything."* This file records what is now measured against it, and
what is still open. Every number was taken on the board, product example (`module_ble_dosimeter`), external bonding
database, at 80 MHz with the core divided for PCLK and the radio as the vendor requires.

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

## Still open, in the order I would take them

```
A  orphan bond      device holds a bond, phone forgot it -> the phone's pairing is answered with the old key and
                    cancelled. The button-hold-at-boot path exists but is unverified on hardware, and the policy
                    (who clears, when) is a product decision.             plan step 9
B  permissions      the characteristics still declare no security requirement, so an unpaired client can read them
                    if the link is not encrypted. The link is encrypted now because the device asks; the attribute
                    permissions should make that a rule rather than a habit.     plan step 7
C  RPA              a phone using a resolvable private address needs the peer IRK path; unmeasured, and its
                    prerequisite measurement is named in the plan.                plan step 6
D  sleeptimer       64-bit wrap at 36.4 h, accuracy reported as 500 ppm while the code records 6000. plan step 8
E  notifications    subscribed-only with sequence numbers and no blocking; `dfail=` is the counter to watch.
                                                                                 plan step 10
F  orphan eviction  one bond per identity, evict when full.                        plan step 9
```

The plan these letters refer to is `_tmp/ble-plan/PLAN.md` (12 ordered steps, written by the analysis model), and
`_tmp/ble-plan/IMPLEMENTED.md` records which steps the implementation model completed and what it could not verify.
