You have already mapped the blast radius across two sessions. Stop reading and start moving code.

The tree is clean; nothing of yours is written yet. Work in this order, and build after each step:

1. Move the board knowledge out of `crates/efr32mg2x-hal/src/board.rs` into `crates/xiao-mg24-bsp`
   (package `efr32mg2x-xiao-bsp`) with `git mv` where a file moves wholesale. Keep the public API shape.
2. Delete the HAL's `board` module and stop exporting it from `crates/efr32mg2x-hal/src/lib.rs`.
3. Move the HAL examples that use `Board` into `crates/xiao-mg24-bsp/examples/`, keeping their names and
   behaviour, with whatever support files they need.
4. Point `crates/xiao-expansion-base-bsp` at the new home of `board::i2c_static` -- it imports the HAL's
   board module today, and leaving it broken is not an option. It may be edited for imports only.
5. Update both READMEs so `python3 scripts/check-crate-refs.py` passes: it reads the imports and fails if a
   README stops explaining a name the code takes from the layer below. Then
   `python3 scripts/gen-crate-readme.py crates/efr32mg2x-xiao-bsp` and the same for the HAL.
6. `bash scripts/gate.sh` must end with `VERDICT: PASS`, and
   `grep -rn "PA7\|PC8\|PC9\|PD5\|PB2\|PB3" crates/efr32mg2x-hal/src/` must print nothing.

If a step cannot be done as described, do the smallest thing that keeps the tree building and say what you
changed instead. Do not commit, do not push, do not publish, do not `rm -rf`.
