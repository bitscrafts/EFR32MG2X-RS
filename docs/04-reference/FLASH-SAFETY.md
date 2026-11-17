# Flash safety: what protects the bootloader, and what does not

## The incident, 2026-11-17

Clearing the NVM3 store before a pairing test, this was run:

```
probe-rs erase --chip EFR32MG24B220F1536IM48
```

It erases the **whole chip**, bootloader included. The board then booted from erased flash, read `0xFFFFFFFF` as its
stack pointer and reset vector, and took an instruction access violation at `0xFFFFFFFE`. Restoring an app did not
help: the BLE stack crashed with the same fault inside `ll_hciCall`, because it reads a pointer from a region that had
been erased.

Recovery, and it only worked because a read-back backup existed from 2026-11-10:

```
file    /projects/_tmp/efr32mg24-rs/backups/mg24-bootloader.bin
size    24 576 bytes, region 0x08000000-0x08005FFF
sha256  8aa5b2f7abc8ee001ae66ab5a8d96410d24f06452bf60bd80811e74013d7a54f
write   probe-rs download --binary-format bin --base-address 0x08000000
check   the 24 KB were read back and hashed: identical to the file
```

The board boots and runs again, and the stack no longer faults. **Nothing else was lost**: no NVM3 content of value
(the store had been written for tests only), no calibration.

## Why `memory.x` did not prevent it

`memory.x` decides where the **linker** may place code. It cannot constrain a debugger: `probe-rs download
--base-address` and `probe-rs erase` write wherever they are told and never read a linker script or an ELF. So the
only protection is procedural, and it has to live in the script that everyone uses:

```
flash.sh refuses an ELF whose load_min is inside 0x08000000-0x08005FFF   (it always did)
flash.sh now also refuses to write at all when the bootloader is missing  (added after this incident)
```

The new guard reads the two words at `0x08000000` before any write and requires a plausible vector table: a stack
pointer in `0x20000000-0x2003FFFF` and a reset handler in `0x08000000-0x08005FFF`. Controls, both mandatory:

```
SP=20001008 reset=08000171   accepts   (the real bootloader)
SP=ffffffff reset=ffffffff   refuses   (the erased region — the case that happened)
SP=2003f000 reset=08006171   refuses   (the eight-byte stub written as an emergency; it points into the app)
```

## The rules that follow

```
1. never run `probe-rs erase` on this board; it is a chip erase, not a page erase
2. to clear NVM3, erase only its own region: 0x08176000-0x0817FFFF (40 KB, five 8 KB pages)
3. before any such write, confirm the bootloader backup exists and matches its sha256
4. a board that boots is not the same as a board that is intact: check 0x08000000, not just that the app runs
```

## What a fresh flash does to the bond store

`probe-rs download` erases only the sectors it writes, so **NVM3 survives every flash**. A bond from an earlier
session therefore outlives the firmware that created it, and a host that has since forgotten the bond will fail
authentication (`HCI 0x05`, and `0x1205` on this side) in a way that looks like a firmware regression. This cost an
hour of misdiagnosis in the same session. Erase the NVM3 region only when the test actually calls for a fresh store.
