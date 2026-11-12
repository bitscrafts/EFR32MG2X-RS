# cmsis-stubs

The `CORE_*` helpers that Silicon Labs' binary blobs reference, compiled once for the whole
workspace.

## Why this exists

`libbluetooth_*` and `librail_*` are prebuilt objects. They were compiled against CMSIS headers
where `CORE_EnterCritical`, `CORE_ExitAtomic`, `CORE_Yield` and friends are `static inline` — so
the blobs contain *references* to them and expect the application to provide the definitions.

Two crates in this workspace link those blobs. If each compiled its own copy, the linker would
either see duplicate symbols or, worse, two implementations that disagree: `CORE_EnterCritical`
masks with `BASEPRI`, `CORE_EnterAtomic` masks with `PRIMASK`, and getting that difference wrong
does not fail to build — it corrupts a buffer only under interrupt load.

## What it is not

It is not a CMSIS port and it does not implement an RTOS. It is the minimum set of symbols the two
blobs ask for, with the semantics the blobs assume. `src/cmsis_stubs.c` says what each one does and
why the mask it uses is the right one.

## Host builds

The helpers are ARM assembly, so `build.rs` compiles nothing for a non-`thumbv*` target: `cargo
test` on the host still works, it just does not have these symbols — and nothing on the host needs
them.
