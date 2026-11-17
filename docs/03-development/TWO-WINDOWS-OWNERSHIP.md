# Two windows, one repo — who owns what, 2026-11-17

Two agents are working on this project at the same time. They must not share a tree, a branch, or a file.
This is the split, and the rules that keep it from becoming the mess that already happened once (two runs in
one `Cargo.toml`, `DO-NOT-REPEAT.md` §9.3).

## Window A — main tree `/projects/efr32mg24-rs`, branch `master`

Owns the lower layers and the board:

```
hal/microphone    timer-paced capture at 16 kHz (replacing 303 kHz polling: 37.8 cycles/sample)
hal/clock         HFXO 39 MHz -> DPLL x2 = 78 MHz, with the lock waits, flash wait states and the
                  timing constants that must be recomputed with it
hal/mvp           the driver is done (Mvp::take, add_fp16) and measured (289 vs 921 cycles at N=64)
                  next: MAC/accumulate encodings from sl_mvp.h, then re-measure at 78 MHz
dosimeter-mg24    NR-15 table instead of the continuous formula; the Leq the docs promise
```

Everything Window A does is verified on the board with the mailbox, and every number it produces is a cycle
count or a measured rate.

## Window B — worktree `/projects/_wt/ble`, branch `ble-work`

Owns the radio and the app's contract:

```
crates/efr32mg2x-ble          the one crate per protocol (FFI + safe API), already created and building
dosimeter GATT service        brief: briefs/dosimeter-gatt-service.md, contract fixed there
60 s at 1 Hz measurement      the link's acceptance, with the microphone capturing at the same time
Android app contract          what each characteristic means, so the app is written against facts
```

## The hardware is one board, and it belongs to one window (2026-11-17)

**Only Window A touches the XIAO MG24.** Not a flash, not a reset, not a `probe-rs read` of the mailbox, not a
`socat` to the probe — nothing that opens the CMSIS-DAP. The reason is not tidiness: two agents sharing one
probe produce measurements that belong to neither, and a flash that lands while the other is reading the
mailbox looks exactly like a firmware defect. This project has already paid for that class of error.

```
Window A   may flash, reset, read the mailbox, and owns the probe for the day
Window B   asks Window A for a measurement, or waits. It never opens the probe.
```

The board is at `2886:0062:28DB6336` on `10.10.10.217`; the flash path is `scripts/flash.sh` with `--confirm`
and the address guard, always.

## The rules between them

```
1. Window B never edits crates/efr32mg2x-hal/** or crates/efr32mg2x-pac/**. Window A owns those.
2. Window A never edits anything under /projects/_wt/ble.
3. Neither publishes. Not to crates.io, not a GitHub release. Publication needs an express order.
4. A change that both need (a new Board method, for instance) is written by Window A and announced here.
5. Each window verifies by measurement in its own tree. A worker's self-report is not evidence.
6. When Window B wants to merge, Window A runs the gate on master first, then the merge is a normal merge.
```

## Where the two meet

Two seams, and only two:

- **`Board::mvp()`** and the microphone API. Window B consumes them; Window A changes them.
- **The GATT contract.** Window A's numbers (Leq, dose, bands) are what Window B publishes. If Window A
  changes a scale, the contract file changes first, and the app is told.

Anything else that looks shared is a sign the split is wrong — say so rather than editing across.
