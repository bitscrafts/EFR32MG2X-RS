# The bond across a reset: every measurement, and the one seam still open (2026-11-17, night)

Status: **the key now survives a reset; the reconnect is still not encrypted.** Three defects of ours were found and
fixed tonight, and the remaining failure is a single, precisely located seam in how key identifiers are exchanged with
the vendor's stack. Everything below was measured on the board, with the `sense,stack` feature set — the same one
`module_ble_dosimeter` requires — and with `scripts/ble-bond-cycle.sh`, which reproduces the whole cycle unattended.

## The cycle

```
scripts/ble-bond-cycle.sh          build -> flash (phase A, bonds cleared) -> flash (phase B, the test)
                                   -> pair from the host -> reset WITHOUT reflashing -> reconnect without pairing
                                   -> read the device's record -> read 40 KB of flash and look for the host's LTK
_tmp/ble-bond-cycle.log            one line per measurement, timestamped
```

The two flashes matter: the device's bonds can only be cleared by firmware, and the reset under test must not clear
what the pairing wrote.

## What the two databases do

```
builtin (vendor blob)   pairing completes: 'Pairing successful', 'Bonded: yes', device ec=1,1,1 gen=2 sev=3 rsn=256
                        the bond record and the key material both reach the flash
                        after a reset the host reconnects and the device drops it with rsn=0x1006
external (ours)         the stack asks for bonding data (dr=1, dt=3 = local_ltk)
                        we answer (ds=0), the stack confirms data_ready (dry=1)
                        the stack stores ONE piece: trc=...3:0:40  (40 bytes of record, never a key)
                        the SMP then aborts with rsn=0x1208 and nothing is ever encrypted
```

Both paths fail in opposite directions, and the trace shows why they are not the same problem: with the external
database the stack never hands the key to the database it was given; with the builtin one it keeps the key itself and
then cannot find it after a reboot.

## The three defects of ours, found and fixed tonight

1. **The restore asked for more bytes than the object holds.** `nvm3_readData` refuses a length larger than the
   object; the restore asked for `PERSIST_HEADER + MAX_KEY` (73 bytes) against 40-byte objects, so **every** read
   failed and the table came back empty while the keys sat in the flash. Read the object's length from
   `nvm3_getObjectInfo` first.
2. **The restore disabled itself before it could work.** It took its "already done" flag *before* checking the NVM3
   handle, so the first PSA call of a boot — which arrives before the stack has opened the instance — turned the
   restore off for the whole boot. Measured as `rcal=1/0/0/0`: one call, handle fine, zero objects, never again.
3. **The retry flag was a function-local `static mut`.** In Rust that is a separate variable per function, so the
   helper that was supposed to allow another attempt cleared a flag nobody read. The flag is at module scope now, and
   a lookup that misses triggers one more restore before reporting the key as absent.

Measured effect, builtin database, same cycle before and after:

```
before   after a reset: rst=0/0   (nothing restored), stack misses key 0x40037, link drops with 0x1006
after    after a reset: rst=6/10  (six keys restored), the host connects, the device still drops with 0x1006
```

So the storage and the restore are no longer the problem. The identifier is.

## The seam that is still open

The stack asks our PSA shim for a key by an identifier it computes itself, and our table holds the keys under the
identifiers our shim assigned.

```
ours, after a restore      ids b, d, e, f, 10, ...          (small, sequential, assigned by NEXT_ID)
the stack asks for         0x48003 (at reconnect), 0x40037 (once, during pairing)
where 0x48000 comes from   the blob itself: `orr.w r0, r0, #0x48000` inside bondingdb_psa_delete_key,
                           i.e. the PSA-backed bonding variant encodes the key id as 0x48000 | (handle<<8 | type)
what the attributes carry  the import path receives a blob whose first eight words are
                           802400, 0, 0, 1400, 3c00200, 0, 0, 0 -- no id, so the id is auto-assigned by us
```

`0x48003` = `0x48000 | 3`, and type 3 is `sl_bt_external_bondingdb_data_local_ltk` — the long-term key of bond 0. So at
reconnect the security manager asks for the right thing under the right name, and our shim answers "no such key",
because it stored it under `0xb`.

The next question is therefore narrow and answerable: **with the builtin database, which call carries the id the stack
will later ask for — and is it `psa_import_key` with the id set in attributes, or a generate/copy variant that takes
the id as an argument?** The answer decides the fix: either the shim honours a requested id on import, or the stack's
own store path must be given the key under `0x48000 | (handle<<8 | type)`.

## Instruments left in place

They answer the next question as well as they answered tonight's, and each is one word in the mailbox record:

```
atr=   first eight words of the last attributes blob seen by psa_import_key
latr=  the same, for the last import whose payload was 16 bytes (a long-term key), plus how many were seen
miss=  lookups that found nothing / the id of the last one
rst=   keys restored from flash / the id of the last one
trc=   the last eight steps of the external bonding conversation: 1 asked, 2 answered, 3 stored, 4 read, 5 ready
```

The mailbox record is 200 bytes, so the live line carries what fits; the rest is available through
`bonding_stats()` and `psa_*` accessors when a test needs it.

## What the cycle proves tonight, in one paragraph

A bond written by the builtin database survives the reset — the key material is in the flash and, after tonight's
three fixes, our shim restores it into RAM at boot (`rst=6/10`). What fails is the lookup: the security manager asks for
the key under the identifier it encodes (`0x48003`), and the key sits under ours. Everything before that point —
pairing, storage, restore — is measured and working.

## The identifier seam, closed later the same night

Two further defects, both found from the disassembly and confirmed on the board.

**The stack names the key and we were ignoring the name.** `bondingdb_psa_add_ecb_key(handle, type, key)` builds
`0x48000 | type | (handle << 8)`, writes it into the attributes blob at **offset 8** (`str r6, [r4, #8]`) and only then
calls `psa_import_key`. The security manager later asks PSA for exactly that identifier. Our shim handed out its own
small id instead, so the key was stored under one name and looked up under another. Honouring the requested id made
the attributes captured at pairing read `802400, 101, 48003, 100, 4404400, ...` — the identifier sitting exactly where
the disassembly said — and the post-reset lookup stopped missing.

**An identifier names one key.** With the id honoured, the table held the same identifier twice — the long-term key
of an earlier pairing in front of the new one — and the lookup answered with the stale key. From the host that looks
like a device that answers with yesterday's key: `AuthenticationCanceled`, while the device records a bond. A
re-import under an identifier already present now replaces that slot and persists it.

State after both: pairing green again (`Pairing successful`, `Bonded: yes`), the long-term key in the flash after a
reset (`reversed=14686` of 40960 bytes), and the reconnect still links and then drops. The next thread is visible in
the instruments: `del=2` — the stack destroys two keys during the boot after a reset, and what it destroys is what
the reconnect then needs.

## Where the night ended, and the thread to pull next

```
pairing                       green: 'Pairing successful', 'Bonded: yes', device ec=1,1,1 gen=2 sev=3
bond record in flash          yes (bh=1 after a reset)
long-term key in flash        yes (found inverted, offset varies with the NVM3 repack)
restore at boot               works: rst=9 in one run, rst=3 in another
reconnect after a reset       still drops, and the miss says which name it wanted: miss=0x48003
```

Two more defects were closed after the ones above, and one instrument was made honest:

- **the identifier was truncated by the store.** NVM3 object keys are 20 bits; `PERSIST_BASE | (id % 0x10000)` turned
  the stack's `0x48003` into `0x8003`, so a restored key came back under a different name. Measured as `rst=9` and
  `miss=0x48003` **in the same boot**. The id now travels in the object's payload.
- **an identifier names one key.** Re-importing under an identifier already present used to append a second slot, so
  the lookup answered with the stale key and the host cancelled the pairing.
- **the mailbox record was capped at 200 bytes** and silently cut the last fields of the live line — twice that was
  `sev`/`rsn`, the fields that explain the failure. It is 512 now. The host's pairing step also retries, because a
  measurement that depends on one timing-sensitive `bluetoothctl` attempt is not a measurement.

**Next thread:** after a reset the stack destroys two keys during initialisation (`del=2`, `dl=1`), and the lookups
then ask for `0x48003`, which the restore did not put back in the last runs. Two questions, in order: does the pairing
that precedes the reset ever import under `0x48003` in that run (`latr=` shows the attributes of the last 16-byte
import), and what does the boot delete? Both are one cycle each with the instruments already in place.
