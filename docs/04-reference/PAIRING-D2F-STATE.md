# D2f pairing — state of the investigation, 2026-11-17

## What the review established, from the blobs' own code

Disassembly of `smp.c.obj` / `sm.c.obj` inside `libbluetooth_host_efr32xg24_gcc_release.a`:

```
as peripheral, SMP reason 0x0B leaves only two states:
  smp_state_slave_public_key_wait        (no ECDH yet — this is the one that fits pg=pe=pa=0)
  smp_state_slave_dhkey_check_wait       (needs ECDH to have run, so not this)
in public_key_wait: smp_verify_remote_public_key returns non-zero -> movs r1,#11 -> 0x0B on the air
  three ways to fail before psa_import_key:
    debug key used and REJECT_DEBUG_KEYS set     ruled out: our flags 0x04 leave bit 7 clear
    peer key equals our own (zero) buffer        ruled out
    bg_zalloc(sli_psa_context_get_size(6) = 64) returns NULL -> sm_send_system_error(0x1A) -> 0x0B
bonding flags cannot produce 0x0B; public_key_wait has no bonding branch
  -> the "SC_ONLY without BONDING_REQUIRED" theory is refuted
```

That is a mechanism, and it names the state, the function and the air symptom.

## What the board says about the reason

`module_heap_probe`, measured with the stack up, pairing enabled, advertising:

```
heap: inuse=10928  fails=0  viol=0  bad=0  total=14  badsize=0     arena = 65 536 bytes
```

**The arena is 17 % used and no allocation has ever failed.** So "the heap is exhausted" does not hold at steady
state, and the review's fix location (`heap.rs::sl_malloc`) is not obviously the right one.

## The two candidates left

```
1. bg_zalloc(64) does not come from sl_malloc at all, but from the host's system buffer pool
   (SL_BT_CONFIG_BUFFER_SIZE = 8192 — a separate arena). Then the fix is in that pool's configuration, and
   our heap numbers are simply not the numbers that matter.

2. the allocation fails transiently, during the pairing exchange, when the heap is fuller than at the steady
   state measured above. It would need a pairing to be in flight to see it.
```

## The measurement that decides it

Pair from a phone or from the host's dongle, with the device recording, at the moment of failure:

```
heap::stats().1                 (failures — did one happen at all)
heap::last_failed_size()        (the review predicts 64)
the system_error event 0x001A   (the review predicts it arrives just before sm_bonding_failed 0x120B)
the host's system buffer pool   free bytes, if the API exposes it
```

`module_heap_probe` already reports the first two and the audit; the event counters are the missing piece and
belong in the same example so one flash answers both candidates.

## What not to do

Do not raise `ARENA_BYTES` again. It is 64 KiB and 54 KiB of it are free; the file's own comment records that
this constant was already sized once against a pairing failure, and the number did not save the pairing then.
Raise a constant only after a measurement shows the constant is what ran out.

## Update, same day: the SM was not in the build at all, and now the failure has a name

Measured on the board with `module_pairing_probe` (live counters, one pair attempt from a phone):

```
before  sm_cfg=0x000F  SL_STATUS_NOT_SUPPORTED   -> the security manager was compiled OUT
        (config/sl_component_catalog.h had SL_CATALOG_BLUETOOTH_FEATURE_SM_PRESENT commented out)
after   sm_cfg=0x0000  configured;  sm_bond=0x000E SL_STATUS_NOT_AVAILABLE  (bonding database off, needs NVM3)
        conn=1         the phone connects and stays
        sec=1          a security event arrived
        rsn=0x120B     SL_STATUS_BT_SMP_DHKEY_CHECK_FAILED
```

`sl_status.h:303`: *"Indicates to the remote device that the DHKey Check value received doesn't match the one
calculated by the local device."*

So every "pairing fails" observation before this date was **"pairing absent"**, and the 0x0B that was chased in the
review above was measured against a build with no security manager in it. With the manager present, the exchange
runs end to end and fails at the one step that depends on our own elliptic-curve arithmetic.

## What the failure means

LE Secure Connections agrees a shared secret with P-256 ECDH and then each side sends a DHKey check value derived
from that secret. A mismatch means our ECDH result differs from the peer's. The PSA shim's self-test passes because
it compares our own output against our own output — a convention error (byte order of the private key, of the
exported public point, or of the shared secret) is self-consistent and therefore invisible to it.

## The next measurement, which needs no phone

A known-answer test: fix a private key from a published vector, export the public key through the shim, compute the
shared secret against a fixed peer public key, and compare both against a reference implementation (`cryptography`
in Python) on the host. Three values, one comparison each. That is what turns "the crypto is wrong" into "the
crypto reverses bytes in <place>".

## CLOSED for pairing, OPEN for bond storage — measured 2026-11-17

The chain that ended it, each step measured on the board:

```
1. the security manager was compiled out        sm_cfg=0x000F NOT_SUPPORTED
   -> enabling it in sl_component_catalog.h was necessary and not sufficient
2. psa_import_key refused the peer's public key  type 0x4112, 65 bytes, rejected by a multiple-of-8 rule
   -> accepted now (validated with p256::PublicKey::from_sec1_bytes)
3. the PSA shim declared key ids as u64          AAPCS shifts every later argument by one register:
                                                 `data` received 65, and copy_nonoverlapping wrote at 0x41
   -> every extern "C" key id is u32 now, which is what psa_key_id_t is
4. with the bonding database OFF:                ec=1,1,1  gen=1  sec=1 sev=3 (sm_bonded) handle=511
                                                 the phone bonds
5. with the bonding database ON:                 rsn=0x1208 SL_STATUS_BT_SMP_UNSPECIFIED_REASON
                                                 the phone aborts
```

**Therefore the pairing works and the bond storage does not.** The two states differ by one catalogue define, so the
remaining defect is in the path that persists keys: NVM3's flash driver and the region it writes into. That code is
ours (`vendor/nvm3/nvm3_hal_flash.c` plus the flash access behind it), and it can be tested without a phone: write an
object, reset, read it back.

The counters that carry this diagnosis are in `module_pairing_probe` and cost nothing to keep:
`ec` (generate/export/agree), `gen`, `imp` (imports/ successes), `ref` (why the last import was refused), `cmac`
(NIST KAT), `bc` (last PSA entry point, for a fault), `sm` (configure/bondable), `conn`, `sec`/`sev`/`rsn`, and the
heap's `largest`/`inuse`/`fails`.

## Bond storage: three measurements, one defect closed, one still open

`module_nvm3_probe` writes a counter to NVM3, and a `boots` counter in `.uninit` proves a reset actually happened
(SRAM survives a reset, so without that control "read 0 after reset" is indistinguishable from "no reset"):

```
nvm3: boots=1242740210 read=0 previous=0 wrote=1 status=0
nvm3: boots=1242740211 read=0 previous=0 wrote=1 status=0
nvm3: boots=1242740212 read=0 previous=0 wrote=1 status=0
```

The resets happen, `nvm3_writeData` reports success every time, and the object is not found on the next boot.

### Closed on the way: the flash routines were running from flash

```
before   .ram     VMA=0x0802e4ac  LMA=0x0802e4ac   <- MSC_WriteWord executing from the flash it programs
after    .ramfunc VMA=0x20011da4  LMA=0x080384e0   <- 444 bytes, copied by Module::take
```

`memory.x` is the wrong home for that section: cortex-m-rt's `link.x` includes `memory.x` *before* its own `SECTIONS`
and discards a `SECTIONS` of ours (`undefined symbol: __ramfunc_start`). `ramfunc.x` plus a `-T` link argument from
`crates/xiao-mg24-bsp/build.rs` is what works.

This defect was real and independent of the bond: any flash write from this firmware was suspect.

### Still open

The flash region does contain data — 435 of 40960 bytes are non-0xFF, and the value written by an earlier run is
present at offset 968 — so writes reach the flash, while NVM3 does not find the object after a reset. That points at
the page structure NVM3 maintains (page headers, the erase-count/resume logic) rather than at the write itself.

The next instrument is to read NVM3's own view instead of the raw bytes: `nvm3_getObjectInfo` for the key, and the
`nvm3_defaultHandleData` counters (page count, erased pages, object count) recorded in the same mailbox line. That
separates "the object was written and then invalidated" from "the page it went to is never scanned".

Known-good state that must not be lost: **the pairing itself works** (`sm_bonded`, handle 511, `ec=1,1,1`, `gen=1`),
and it works with the bonding database *disabled*. Enabling the database is what makes the peer abort (0x1208),
because the stack then tries to persist and fails.

## WORKING, measured from the host, 2026-11-17 — and the one thing still open

Performed from `lnx` (BlueZ 5.65, controller `90:DE:80:81:44:56`), which reports pairing state more plainly than a
phone. The scan had failed for an hour with `org.bluez.Error.InProgress` and a restart of the service cleared it; the
recipe that works is one `bluetoothctl` process as root, fed through a pipe:

```bash
sudo sh -c '{ printf "scan on\n"; sleep 8; printf "pair E4:0A:75:13:3E:E0\n"; sleep 16; \
              printf "info E4:0A:75:13:3E:E0\nquit\n"; } | timeout 45 bluetoothctl --timeout 40'
```

Result, with the bonding database enabled (`SL_CATALOG_BLUETOOTH_FEATURE_BUILTIN_BONDING_DATABASE_PRESENT`):

```
BlueZ                            device (module_pairing_probe)
Pairing successful               ec=1,1,1  gen=2  imp=13/12  sm=0x0/0x0
Paired: yes                      conn=1    sec=1 sev=3 (sm_bonded) handle=256
Bonded: yes
```

With the database **disabled** the same procedure gives `Paired: yes, Bonded: no` and `sm_bond=0x0E`
NOT_AVAILABLE. So the earlier Android `0x1208` was a stale bond cached by the phone, not a device defect: from a
clean host the device pairs and bonds.

### Open: the bond does not survive a reset

```
reset the board, then connect from lnx without pairing:
  Connection successful -> Connected: yes -> Connected: no
  device: sev=0x2005 (disconnected), rsn=0x1006 = HCI "PIN or Key Missing"
```

The peer offers to encrypt with the key it stored; the device has no key, so the link dies. That is the NVM3
persistence defect below, now confirmed from outside the device by a standard HCI error rather than by our own
counters.

So D2f is closed as *"pairing and bonding work"* and open as *"bonds do not persist"*. A product needs both.

## The NVM3 "defect" was my instrument lying — corrected 2026-11-17

`nvm3_readData` returns `Ecode_t`, an **error code where 0 is success**. The probe treated the return value as a byte
count, so a working read (`0`) was reported as "object not found", the probe then overwrote its own counter with 1,
and every conclusion drawn from it was wrong. Corrected, the same probe shows NVM3 persisting across resets:

```
run 1      previous=2  wrote=3
reset 1    previous=3  wrote=4      <- survived a reset
reset 2    previous=4  wrote=5
```

The lesson is the project's own rule, paid for again: a check whose *interpretation* is wrong can only certify
falsehoods. `Ecode_t` and `size_t` are both 32 bits on this target, so the compiler had nothing to object to.

## What was wrong, and what that leaves

```
the flash controller          fine: erase=0 write=0, pattern read back (module_nvm3_probe, page 0x0803E000)
SYSTEM_GetFlashPageSize        8192, which is FLASH_PAGE_SIZE for this part
NVM3 object lifecycle         fine, once read back with the right return type
.ram section                  WAS broken and is fixed (see above): the flash routines ran from flash
stack stores the bond         NOT OBSERVED: after a pairing the peer says `Bonded: yes` while the device's
                              NVM3 holds zero objects (`nvm=0` from nvm3_enumObjects)
```

So the remaining question is narrower than it has ever been: **does the stack ever write the bond?** A peer that says
"Bonded: yes" only proves its own copy. The device's NVM3 object count is the instrument that answers it, and it
needs one clean pairing on this build (a fresh flash erases the chip, NVM3 included, so `nvm=0` at boot is expected).

## Current state of a pairing attempt, and a caveat

A clean retest at the end of the session was refused with `0x1205` `SL_STATUS_BT_SMP_PAIRING_NOT_SUPPORTED`, after
the host had `remove`d its bond and the board had been reset. That is a *different* failure from the earlier
successful pairing (which reported `sm_bonded` with a handle), and it is unretested: the host may be in a repeated-
attempts state, or the reset left the SM's own NVM3 record inconsistent. The next session must start from a known
clean pair of states (erase the host bond, flash the board fresh) before drawing any conclusion from it.

## What must not be lost from this session

```
1. the PSA key ids are u32, not u64 — every extern "C" key id, plus the *mut u32 out parameter
2. psa_import_key accepts the 65-byte ECC public key (type 0x4112)
3. SL_CATALOG_BLUETOOTH_FEATURE_SM_PRESENT must stay enabled, and so must the builtin bonding database
4. .ramfunc in RAM with its load image in flash, copied by Module::take, via ramfunc.x + -T from build.rs
   (both BSP crates need the pair)
5. the host pairing recipe: one bluetoothctl as root fed through a pipe (see above)
6. NVM3 keys are 20 bits, readData returns Ecode_t, and a fresh flash erases NVM3
```

## The bond IS stored and IS loaded — the reconnect fails to find it (2026-11-17, last measurement)

The instrument that settled it is the stack's own view, not NVM3: `sl_bt_sm_get_bonding_handles` returns how many
bonds the stack believes it holds. Enumerating NVM3 from the main loop was the wrong probe twice over -- the same
handle is used by the stack from its own context, and an enumeration every millisecond returned zeros that looked
like an empty store.

```
bonding cleared                        bh=0/0
pairing from the lnx                   Pairing successful          bh=1/0   conn=1
probe-rs reset (no reflash)             --                         bh=1/0   conn=0
lnx connects WITHOUT pairing            Connected, then dropped    rsn=0x1006  PIN or Key Missing
```

`bh=1` after the reset is the proof that storage works: the bond is in flash, the stack loads it, and the peer's
keys are not the problem.

What fails is the lookup: the stack holds one bond and cannot bind it to the peer that reconnects, so it answers
`0x1006` and the link drops. That is identity -- the address the peer presents versus the address stored -- not
storage. The next instrument is `sl_bt_sm_get_bonding_details`, which returns the stored address and its type, to
compare against the address used on the reconnect.

Consequences for the product, unchanged until this closes: `enable_pairing` stays out of the dosimeter example,
because a device that pairs and cannot be reconnected is worse for the app than one that does not pair at all.

## The bond is stored, loaded, indexed under the right address — and the key is refused

`sl_bt_sm_get_bonding_details` closes the identity question. Read from the device while the host was connected:

```
bd=56448180de90   tipo=0 (publico)   modo=1 (emparelhamento nao autenticado com cifra)   chave=16 bytes
peer=56448180de90 <-- o endereco que se ligou e' exactamente o que esta' guardado
bh=1/0            o stack tem o bond
resultado         a ligacao cai, e o ultimo evento de seguranca traz 0x1006 PIN or Key Missing
```

So the three things that were in doubt are settled: the bond is written, it survives a reset, and it is found by the
address the peer presents. What fails is the key itself -- the host asks for encryption and the device answers that
it has no key for it, or the key it holds is not the one the host derived.

Where the next instrument goes, in the order that costs least:

1. read the stored bond's content from NVM3 and see whether a 16-byte value with the LTK's shape is actually there,
   which separates "the record was saved without its keys" from "the keys were saved and differ"
2. if the record is complete, the disagreement is in derivation: the host's LTK comes from the pairing exchange, and
   ours from whatever the stack kept, so one of the two paths is not the one that produced the other's value

A note on the instruments themselves, because they cost more than the silicon did: `conn` in the probe's live line is
a COUNT of connections, not the current state -- reading `conn=1` as "connected" sent this session after the wrong
problem twice. The record is now read together with the host's own `Connected:` line, and the two disagree only when
something is actually wrong.
