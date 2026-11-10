# Flash read-back backups

Two images are captured from the board over SWD (read-only, `dump_image`), so the
state that actually exists on the chip can be restored without a rebuild:

| file | region | sha256 |
|---|---|---|
| `mg24-bootloader.bin` | 0x08000000-0x08005FFF (24 KiB Gecko bootloader) | `8aa5b2f7abc8ee001ae66ab5a8d96410d24f06452bf60bd80811e74013d7a54f` |
| `mg24-app-64k.bin` | 0x08006000-0x08015FFF (application area) | `340d8764fa1b8f4195cea6ce6b7b64c29c6824534b755409be73cc4d7d78d8c2` |

Taken 2026-11-10. The files live outside the repository at
`/projects/_tmp/efr32mg24-rs/backups/` with a `SHA256SUMS` and a README that
explains why a restore needs a flash driver (our OpenOCD config deliberately has
none, so it can read and cannot program).

The bootloader backup matters: it is the only way back from a debug-locked
Access Port, which is the failure mode that cost a board on 2026-06-29.

Sanity check on the dumps, decoded from the images themselves:

```
bootloader vector table: SP=0x20001008 reset=0x08000171
app vector table:        SP=0x20001208 reset=0x08006159
```
