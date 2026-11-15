# Brief: stop committing the vendor trees, fetch them at build time

You are working in `/projects/efr32mg24-rs`. Two crates commit tens of megabytes of Silicon Labs vendor
code into the repository. One crate already does it properly, and it is the model to copy:

```
crates/efr32mg24-rail-sys/build.rs     <-- read this first, it is the pattern
crates/efr32mg24-bluetooth-sys          18 MB of vendor, committed
crates/efr32mg24-emlib-sys              28 MB of vendor, committed
```

## The one job

Give `efr32mg2x-bluetooth-sys` and `efr32mg2x-emlib-sys` the same build-time vendor fetch that
`efr32mg2x-rail-sys` already has, so the archives stop living in the repository, and make the failure
mode **closed**.

The rail-sys pattern, which you are copying:

- a pinned commit, a `codeload` tarball URL, and the tarball's sha256, all three as constants;
- a cache under `$CARGO_HOME/efr32mg2x-vendor/<commit>/repo.tar.gz`, keyed by commit so one download
  serves every crate on the machine;
- `curl` to fetch when the cache is cold, sha256 verified before anything is extracted;
- extraction into the crate's own `vendor/` (and `blobs/` where it applies).

What must differ from rail-sys: **it currently prints `cargo:warning=` and carries on when the vendor is
missing**. That is fail-open, and a build that silently proceeds without the vendor fails later with an
undefined symbol and no explanation. For these two crates, a missing vendor that cannot be fetched must
be a build **error** with a message that says what is missing, which URL was tried, and that the network
or the pinned commit is the thing to look at.

## Hard constraints

- **Never** run `git commit`, `git push`, `cargo publish`, or `cargo yank`.
- **Never delete anything.** The vendor trees are pre-existing and irreplaceable if the fetch is wrong.
  Move them out of the way with `mv` into `/projects/_tmp/efr32mg24-rs/trash/vendor-<crate>-<timestamp>/`
  and leave them there for a human. A `rm -rf` in this repository is forbidden without exception.
- **Never** touch `crates/efr32mg2x-pac` or `crates/efr32mg2x-hal`.
- **Never** flash or talk to hardware.
- Do not rename directories; the `efr32mg24-*` directory names are a later job.

## Acceptance criteria

1. `du -sm` of the repository's `crates/` directory is **at least 40 MB smaller** than before you started.
   Report both numbers.
2. With a **cold cache** the build still works: build once with
   `CARGO_HOME=$(mktemp -d) cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg2x-radio`
   and show it fetching and verifying the archive. This is the criterion that proves the fetch is real;
   a build that passes only because the vendor tree was already there proves nothing.
3. With the vendor tree empty **and** an unreachable URL, the build **fails** with the explanatory error.
   Demonstrate it (point the URL constant at a path that does not exist in a scratch copy, or unset
   network access for one run) and paste the error. Restore the constant afterwards.
4. `bash scripts/gate.sh` ends with `VERDICT: PASS`.
5. `git status --porcelain` shows your work and `git log --oneline -1` is unchanged.

## Report

Say in this order: the `du` before and after, the cold-cache build output (the fetch lines), the
fail-closed error text, the gate verdict, and anything you found that a reviewer must check by hand.
