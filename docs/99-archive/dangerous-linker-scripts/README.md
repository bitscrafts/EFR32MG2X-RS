# Linker scripts kept out of the published crate

Two files that used to live inside `crates/efr32mg24-pac/` and therefore shipped in every published copy of
`efr32mg2x-pac`. They are kept here as a record, not as something to use.

## `memory_direct.x`

```
FLASH : ORIGIN = 0x08000000, LENGTH = 1536K     /* Direct flash - NO BOOTLOADER - for testing only! */
```

An image linked with this starts at the very bottom of flash, where the Gecko bootloader's vector table lives. Flashing
it replaces the bootloader: the board stops booting and only a debugger brings it back. It was written during bring-up
when the plan was still "no bootloader"; the plan changed, the file did not, and it was published inside the crate where
a reader could find it with no context at all.

Nothing in the tree referenced it. `scripts/check-flash-addresses.py` now refuses any linker script below
`0x08006000` that is not declared as a bootloader, and refuses to let such a file travel inside a published crate.

## `memory.x.bootloader`

A duplicate of `memory.x` with a slightly different RAM size (`256K - 8`). Safe on flash (`0x08006000`) but unreferenced
and confusing: two files claiming to be the layout, one of them wrong about the top 4 KiB. `memory.x` is the layout.
