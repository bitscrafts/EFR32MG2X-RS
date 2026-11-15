# Documentation that ships with this crate

Four files travel inside the package. These are the ones:

| file | what it is |
|---|---|
| [../README.md](../README.md) | the crate page: what is in the tree, what was measured on the board, what carries a defect |
| [crate-doc.md](crate-doc.md) | the crate-level rustdoc — included from `lib.rs`, so it is what docs.rs shows |
| [LINKER_SETUP.md](LINKER_SETUP.md) | the linker script and the memory map: where the bootloader ends and why `--nmagic` matters |

Everything else that used to live in this directory — the phase plans, the build-system notes, the per-module task
sheets, the status file — was developer material. It moved to the repository's `docs/` tree, because a package is read by
strangers who have no repository and no history to interpret it with:

- phase plans and per-module task sheets: `docs/99-archive/hal-module-plans/`
- the build system notes: `docs/02-guides/HAL-BUILD-SYSTEM.md`
- the dated status: `docs/03-development/HAL-STATUS.md`

Those paths are repository paths, not package paths. They will not resolve here, and that is deliberate: the package
ships only what is true of the crate.
