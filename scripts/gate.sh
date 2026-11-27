#!/usr/bin/env bash
# The gate that can run without the board.
#
# Every check here has a reason to exist, and each one has failed for real at least once
# in this workspace:
#
#   fmt        the tree drifts into unformatted code that later hides a real diff
#   clippy     zero warnings is the standing rule; 16 accumulated in one afternoon
#   examples   a crate whose examples do not build is a crate nobody can try
#   tests      the classifier's thresholds are only as good as the tests that pin them
#
# What it deliberately does not do:
#
#   - it does not touch hardware. Flashing belongs to scripts/flash.sh, which refuses
#     an image that would overwrite the bootloader.
#   - it does not build the Bluetooth sys crate. That crate has 17 errors from its own
#     bindgen output (`E0609 no field ... &__BindgenUnionField`) and the headers it needs
#     are gitignored, so it cannot be built from a clean checkout by anyone. It is named
#     here rather than silently skipped: a gate that hides a broken crate is worse than no
#     gate.
#
# Usage: scripts/gate.sh
set -uo pipefail
cd "$(dirname "$0")/.."

TARGET=thumbv8m.main-none-eabihf
HOST=x86_64-unknown-linux-gnu

# ## cargo on the PATH, whoever called this
#
# The pre-commit hook runs this script from whatever environment git had, and git does not inherit an
# interactive shell's PATH. Measured 2026-11-21: a commit attempt through the hook reported six unrelated
# failures -- `scripts/gate.sh: line 225: cargo: command not found` among them -- on a tree that had passed the
# same gate minutes earlier from a shell that had exported PATH. A gate whose verdict depends on who called it
# is not a gate; it is a coin. The earlier steps that already prefixed `PATH=... python3` fixed this one call at
# a time instead of at the top.
export PATH="/usr/local/cargo/bin:${PATH}"
if ! command -v cargo >/dev/null 2>&1; then
  echo "VERDICT: FAIL"
  echo "  FAIL  cargo is not on the PATH, so nothing here can be checked"
  exit 1
fi
FAILED=0
pass() { printf '  PASS  %s\n' "$1"; }
fail() { printf '  FAIL  %s\n' "$1"; FAILED=1; }

echo "=== format ==="
if cargo fmt --all --check >/dev/null 2>&1; then
  pass "cargo fmt --all --check"
else
  fail "cargo fmt --all --check (run: cargo fmt --all)"
fi

# `msg[at..at + N].copy_from_slice(b"...")` with N that does not match the literal compiles and panics on the board.
# It happened three times in one night (`" len="` six into five, `"gatt: no attrs s"` fifteen into sixteen, `"c p="`
# five into four), and every instance was found by flashing a firmware and reading a mailbox that never changed.
# It costs milliseconds here, which is the whole argument for putting it in the gate rather than in a note.
echo "=== message buffers ==="
# The crate documentation's example page is generated. If it does not match the tree it is naming examples that no
# longer exist -- which is a documentation defect that compiles and looks fine on the page.
# Every README, always: a dead link or a path that is not there is the documentation equivalent of a test
# that passes while the code is broken.
if python3 scripts/check-readmes.py --check >/tmp/gate-readmes-all.log 2>&1; then
    printf '  PASS  docs: every README points at something that exists\n'
else
    printf '  FAIL  docs: a README points at something that is not there\n'
    sed -n '1,20p' /tmp/gate-readmes-all.log | sed 's/^/        /'
    FAILED=1
fi

# The module pages are the whole point of the generated docs, and svd2rust writes none of them. A regeneration
# that drops them must fail here, not on docs.rs where nobody would notice until somebody looked.
# "No Description" e' o que o SVD do vendor diz 6 859 vezes; o docs.rs mostrava-o em todos os registos. Se uma
# regeneracao o trouxer de volta, falha aqui.
if python3 scripts/placeholder-register-docs.py --check >/tmp/gate-pac-regs.log 2>&1; then
    printf '  PASS  pac: no register says "No Description"\n'
else
    printf '  FAIL  pac: registers carry a vendor No Description again\n'
    sed -n '1,6p' /tmp/gate-pac-regs.log | sed 's/^/        /'
    FAILED=1
fi

if python3 scripts/add-pac-module-docs.py --check >/tmp/gate-pac-docs.log 2>&1; then
    printf '  PASS  pac: every module says what it is\n'
else
    printf '  FAIL  pac: module descriptions missing or stale\n'
    sed -n '1,10p' /tmp/gate-pac-docs.log | sed 's/^/        /'
    FAILED=1
fi

# libraries/: a crate here that reaches the hardware stack cannot be tested on the host,
# and that is the whole reason the folder exists. It reads the cargo metadata graph, not a
# grep of cargo tree output: the shell version stayed green while checking nothing, and its
# control proved neither direction. Control: add [dependencies.efr32mg2x-pac] to a library
# in a clone and run this script -- it must print VERDICT: FAIL.
if python3 scripts/check-library-purity.py >/tmp/purity.out 2>&1; then
    printf '  PASS  libraries: all of them stay testable on the host\n'
else
    printf '  FAIL  libraries: one of them reaches the hardware stack\n'
    sed 's/^/        /' /tmp/purity.out | head -5
    FAILED=1
fi

# O exemplo que o README manda copiar tem de compilar, como um estranho o leria. Todos os outros checks leem a
# crate; este le o par (Cargo.toml, programa) que o leitor recebe, e foi o unico que apanhou o panic-halt em falta.
if python3 scripts/check-readme-example.py crates/efr32mg2x-pac/README.md /tmp/gate-readme-example --target thumbv8m.main-none-eabihf >/tmp/gate-readme-example.log 2>&1; then
    printf '  PASS  docs: the README example builds in an empty crate\n'
else
    printf '  FAIL  docs: the README example does not build\n'
    sed -n '1,6p' /tmp/gate-readme-example.log | sed 's/^/        /'
    FAILED=1
fi

if python3 scripts/gen-pac-examples-doc.py --check >/tmp/gate-examples-doc.log 2>&1; then
  pass "crate docs: the example list matches the tree"
else
  fail "crate docs: $(tail -1 /tmp/gate-examples-doc.log)"
fi

# The HAL README states how many public modules and examples the crate has, and those numbers are typed by hand. They
# drifted twice on 2026-11-25 -- one module added, the count left at the old value -- which is the same failure as a rubric
# row that describes code that no longer exists: a number a reader trusts and nobody checked. Compared against the tree.
stated_mods=$(grep -oE '[0-9]+ public modules' crates/efr32mg2x-hal/README.md | head -1 | grep -oE '[0-9]+')
stated_examples=$(grep -oE 'public modules \(`pub mod` in `src/lib.rs`\) and [0-9]+ examples' crates/efr32mg2x-hal/README.md | head -1 | grep -oE '[0-9]+ examples' | grep -oE '[0-9]+')
real_mods=$(grep -c '^pub mod' crates/efr32mg2x-hal/src/lib.rs)
real_examples=$(ls crates/efr32mg2x-hal/examples/*.rs 2>/dev/null | wc -l)
if [ "$stated_mods" = "$real_mods" ] && [ "$stated_examples" = "$real_examples" ]; then
  pass "hal README counts: $real_mods modules, $real_examples examples"
else
  fail "hal README says ${stated_mods:-?} modules and ${stated_examples:-?} examples; the tree has $real_mods and $real_examples"
fi

if python3 scripts/check-root-readme.py >/tmp/gate-root-readme.log 2>&1; then
  pass "root README counts: $(tail -1 /tmp/gate-root-readme.log | sed 's/^PASS //')"
else
  fail "root README counts: $(grep -m1 '^FAIL' /tmp/gate-root-readme.log | sed 's/^FAIL //')"
fi

# Every crate that ships gets its README generated from templates/readme-<name>.tmpl, and the gate refuses a README
# that has drifted from it. The list of "crates that ship" comes from check-publishable.py, so adding a crate to the
# published set immediately requires a template for it -- the rule cannot be forgotten, only enforced.
if python3 scripts/gen-crate-readme.py --all --check >/tmp/gate-readmes.log 2>&1; then
  pass "docs: every README is generated from its template"
else
  fail "docs: $(grep -m1 '^FAIL' /tmp/gate-readmes.log | sed 's/^FAIL //')"
fi

# A generated file whose generator is lost can only be edited by hand, in the way its own header forbids. This one was
# generated by a script living in /projects/_tmp, one cleanup away from gone (`B18`). The check regenerates and
# compares, so the table cannot drift from the script that produces it.
if PATH="/usr/local/cargo/bin:$PATH" python3 scripts/emit-ble-config.py --check >/tmp/gate-ble-config.log 2>&1; then
  pass "radio: the BLE config table matches its generator"
else
  fail "radio: $(grep -m1 '^FAIL' /tmp/gate-ble-config.log | sed 's/^FAIL //')"
fi

# Every `unsafe` block in the HAL has to say what it is relying on. The check exists because the sweep below happened
# twice already (2026-11-17 and 2026-11-25) and the notes decay as soon as a new register write arrives unexamined.
if python3 scripts/check-safety-notes.py >/tmp/gate-safety.log 2>&1; then
  pass "unsafe: $(tail -1 /tmp/gate-safety.log | sed 's/VERDICT: PASS //')"
else
  fail "unsafe: $(grep -m1 '  ' /tmp/gate-safety.log)"
fi

# What would be published, read the way a stranger reads it. cargo publish --dry-run compiles the package and never
# looks at it: that is how ef32mg2x-hal 0.1.0 reached crates.io announcing "I2C/SPI/Timers next" with all three in the
# tree, and linking a docs/PHASE2_PLAN.md that does not exist.
if PATH="/usr/local/cargo/bin:$PATH" python3 scripts/check-publishable.py >/tmp/gate-publishable.log 2>&1; then
  pass "crates.io: the package says what the tree does"
else
  fail "crates.io: $(grep -m1 '^  ' /tmp/gate-publishable.log)"
fi

# Every crate must point at the layer below it: hal -> pac, bsp -> hal and pac. A crate that never
# names what it is built on reads as though it were standalone, and a reader landing there cannot walk
# the stack. The rule lives here, not in a habit, because this is the kind of thing that erodes.
if python3 scripts/check-crate-refs.py >/tmp/gate-crate-refs.log 2>&1; then
  pass "docs: every crate points at the layer below it"
else
  fail "docs: $(grep -m1 '^FAIL' /tmp/gate-crate-refs.log | sed 's/^FAIL //')"
fi

# Every linker script that can produce an image must start at or above 0x08006000, and a dangerous one must never ship
# inside a published crate. This is the brick check, at the source rather than at the flash command.
if PATH="/usr/local/cargo/bin:$PATH" python3 scripts/check-flash-addresses.py >/tmp/gate-flash-addr.log 2>&1; then
  pass "flash: no image can be written into the bootloader"
else
  fail "flash: $(grep -m1 '^  ' /tmp/gate-flash-addr.log | sed 's/^  //')"
fi

# The product itself, which now lives in `apps/dosimeter-ble` -- outside the workspace, because it depends on
# `dosimeter-mg24` and that crate is outside the repository by its own project's rule. A path dependency pointing outside
# the checkout is resolved when the workspace loads, so keeping the firmware as a workspace member made every CI job
# fail with `failed to read /projects/dosimeter-mg24/Cargo.toml` (measured 2026-11-27). Quarantined there, the repository
# builds anywhere and the firmware builds where the model lives.
#
# The skip is named, never silent: on a machine without the model there is nothing to build, and saying so is the honest
# result. A skip that hid a broken product build would be worse than no check -- so where the model *is* present, a
# failure is a failure.
if [ ! -f /projects/dosimeter-mg24/Cargo.toml ]; then
  echo "  SKIP  product: /projects/dosimeter-mg24 is not on this machine, so the firmware cannot be built here"
elif PATH="/usr/local/cargo/bin:$PATH" cargo build --release --manifest-path apps/dosimeter-ble/Cargo.toml \
    --target "$TARGET" >/tmp/gate-dosimeter.log 2>&1; then
  pass "product: the dosimeter firmware builds with the features it ships with"
else
  fail "product: $(grep -m1 '^error' /tmp/gate-dosimeter.log | sed 's/^error[^:]*: *//')"
fi

# The Embassy time driver, same story: `required-features` means the loop above builds no such target, and this is
# the example that proves `schedule_wake` reaches a waker through the comparator.
if PATH="/usr/local/cargo/bin:$PATH" cargo build --release -p efr32mg2x-xiao-bsp --target "$TARGET" \
    --features "sense" --example embassy_driver_systrc >/tmp/gate-embassy.log 2>&1; then
  pass "embassy: the time driver's example builds (waker through the comparator)"
else
  fail "embassy: $(grep -m1 '^error' /tmp/gate-embassy.log | sed 's/^error[^:]*: *//')"
fi

# The other half of the same seam: the executor and `Timer` on the same driver. Two examples because the two can
# fail differently -- a driver without a consumer calls a waker nobody waits on, a consumer without a driver sits
# with its tasks unrun.
if PATH="/usr/local/cargo/bin:$PATH" cargo build --release -p efr32mg2x-xiao-bsp --target "$TARGET" \
    --features "sense" --example embassy_task_systrc >/tmp/gate-embassy-task.log 2>&1; then
  pass "embassy: the executor's example builds (a Timer waking a task)"
else
  fail "embassy task: $(grep -m1 '^error' /tmp/gate-embassy-task.log | sed 's/^error[^:]*: *//')"
fi

# The night queue must refuse to mark an item "done" when no work came out of the worker. The rule lives in
# `queue-run-once.sh`; this exercises the script itself, in a throwaway repository, with a worker that is not pi
# -- including the case where the only thing that changed was the queue file the worker edited.
if bash scripts/test-queue-run-once.sh >/tmp/gate-queue.log 2>&1; then
  pass "queue lane: four ways of not doing the work, all refused ($(grep -c '^  PASS' /tmp/gate-queue.log) cases)"
else
  fail "queue lane: $(grep -m1 '^  FAIL' /tmp/gate-queue.log | sed 's/^  FAIL *//')"
fi

if python3 scripts/check-copy-len.py crates/ >/tmp/gate-copy-len.log 2>&1; then
  pass "copy_from_slice lengths ($(tail -1 /tmp/gate-copy-len.log))"
else
  fail "copy_from_slice length mismatch"
  sed -n '1,8p' /tmp/gate-copy-len.log | sed 's/^/        /'
fi

# Every crate in the workspace gets checked, discovered rather than listed: a hardcoded list
# silently stops covering the crate that was added last, which is the one that needs it most.
CRATES=$(ls -d crates/*/ libraries/*/ 2>/dev/null | sed 's:/*$::')
# Named here so the gap is visible instead of silent:
#   efr32mg24-bluetooth-sys, efr32mg24-rail-sys  bindgen output does not compile from a clean
#                                                checkout; their headers are gitignored
#   efr32mg24-bootloader, bootloader-test-app     separate workspaces, excluded from this
#                                                one on purpose
SKIP="efr32mg24-bluetooth-sys efr32mg24-rail-sys efr32mg24-bootloader bootloader-test-app"
pkg_name() {
  # The crates.io name lives in the manifest; the directory keeps its old name. Read it, never infer it. A directory
  # with no manifest is not a crate: return nothing instead of sending sed after a file that is not there.
  [ -f "$1/Cargo.toml" ] || return 1
  sed -n 's/^name *= *"\([^"]*\)"/\1/p' "$1/Cargo.toml" | head -1
}

features_for() {
  # Keyed by directory basename on purpose: the crates.io name may change, the directory may not.
  case "$1" in
    efr32mg2x-pac|efr32mg2x-hal) echo "--features rt" ;;
    xiao-mg24-bsp) echo "--features sense" ;;   # the microphone and the IMU exist only on the Sense variant
    *) echo "" ;;
  esac
}

echo "=== clippy, zero warnings ==="
for dir in $CRATES; do
  pkg=$(pkg_name "$dir")
  # The skip list is keyed by directory: a rename must not resurrect a crate that is deliberately not gated.
  case " $SKIP " in *" $(basename "$dir") "*) continue ;; esac
  [ -f "$dir/Cargo.toml" ] || continue
  extra=$(features_for "$(basename "$dir")")
  out=$(cargo clippy --release --target "$TARGET" -p "$pkg" $extra 2>&1)
  # `cargo:warning=` from a build script is echoed as "warning: <crate>@<version>: ..." and is an
  # INFORMATIVE message, not a lint. Counting those made this check fail crates whose own code had
  # zero warnings — a gate that lies in the strict direction is still a gate that lies.
  n=$(printf '%s\n' "$out" | grep -E '^warning: [a-z]' | grep -vE '^warning: [a-z0-9_-]+@[0-9.]+:' | wc -l)
  if printf '%s\n' "$out" | grep -qE '^error'; then
    fail "$pkg clippy (does not compile)"
  elif [ "$n" -eq 0 ]; then
    pass "$pkg clippy, 0 warnings"
  else
    fail "$pkg clippy, $n warnings"
  fi
done

# The skip list above leaves efr32mg24-rail-sys out of every other step. That crate gets its own
# check, so being excluded from the gate does not mean being unmeasured.
echo "=== rail-sys (skipped above, checked here) ==="
if bash scripts/check-rail-sys.sh; then
  pass "rail-sys: builds, clippy clean, 802.15.4 bindings present"
else
  fail "rail-sys: $(grep -m1 'FAIL' /tmp/check-rail-sys.log 2>/dev/null || echo 'see scripts/check-rail-sys.sh')"
fi

echo "=== examples build ==="
for dir in $CRATES; do
  pkg=$(pkg_name "$dir")
  # The skip list is keyed by directory: a rename must not resurrect a crate that is deliberately not gated.
  case " $SKIP " in *" $(basename "$dir") "*) continue ;; esac
  [ -f "$dir/Cargo.toml" ] || continue
  extra=$(features_for "$(basename "$dir")")
  count=$(ls "$dir/examples/"*.rs 2>/dev/null | wc -l)
  # A domain crate's examples run on the host: they are how its API is exercised without a
  # board, and building them for the embedded target would be the wrong question.
  case "$dir" in
    libraries/*) example_target="$HOST" ;;
    *)          example_target="$TARGET" ;;
  esac
  if cargo build --release --target "$example_target" -p "$pkg" $extra --examples >/tmp/gate-examples-$pkg.log 2>&1; then
    pass "$pkg: $count examples"
  else
    # Keep the compiler's own words. A gate that reports "do not build" without saying why forces the
    # next person to reproduce the build by hand -- and on a CI runner they often cannot.
    fail "$pkg: examples do not build"
    echo "--- first errors from /tmp/gate-examples-$pkg.log:"
    if grep -qE "^error(\[|:)" "/tmp/gate-examples-$pkg.log"; then
      grep -E "^error(\[|:)" -A4 "/tmp/gate-examples-$pkg.log" | head -20 | sed 's/^/      /'
    else
      # Not every failure announces itself with "error:": a build script that panics, a missing
      # tool, or a linker that dies all read differently. Print the tail rather than nothing.
      tail -20 "/tmp/gate-examples-$pkg.log" | sed 's/^/      /'
    fi
  fi
  # A hardware crate whose API nothing builds is a crate whose API is not tested. Only the
  # crates that own pins are held to this; the PAC is exercised by its own 19 examples and
  # the domain crates by their host tests.
  case "$pkg" in
    *-bsp)
      if [ "$count" -eq 0 ]; then
        fail "$pkg: exposes hardware but has no examples"
      fi
      ;;
  esac
done
# An example that lives in a directory is only built by `--examples` if its [[example]]
# block declares it. A directory nobody declared compiles never and fails silently, so the
# check is on the declaration, from the name the manifest gives it.
for dir in crates/*/examples/*/; do
  [ -f "${dir}main.rs" ] || continue
  crate=$(basename "$(dirname "$(dirname "$dir")")")
  rel="examples/$(basename "$dir")/main.rs"
  name=$(grep -B4 "path = \"$rel\"" "crates/$crate/Cargo.toml" | grep -m1 '^name = ' | cut -d'"' -f2)
  if [ -n "$name" ]; then
    pass "$crate: $rel declared as example \"$name\""
  else
    fail "$crate: $rel is not declared in Cargo.toml (it would never build)"
  fi
done

echo "=== layer boundaries: the PAC belongs to the PAC, the wiring to a BSP ==="
# These are the four rules the stack exists to keep. Each one has been broken at least once,
# and every time it was broken the code still compiled — which is why they are checks here and
# not paragraphs in a document.

# 1. An example may only name the layer it demonstrates. A HAL example that mentions the PAC
#    is a HAL that failed to hand out what it owns; the caller then has to steal the singleton.
# Comments are stripped before the search: the rule is about code that *names* a PAC type, and a documentation line
# that mentions `pac::X::steal()` is explaining why the example does not use it. Matching those made the honest
# examples the ones that failed, which is how a rule gets reworded away instead of kept.
offenders=$(grep -rn "efr32mg24_pac\|pac::" crates/*/examples/ --include=*.rs 2>/dev/null \
  | grep -vE ':[0-9]+:[[:space:]]*(//|/\*|\*)' \
  | grep -vE "^crates/(efr32mg2x-pac|efr32mg24-bluetooth-sys|efr32mg24-rail-sys)/" \
  | cut -d: -f1 | sort -u || true)
if [ -z "$offenders" ]; then
  pass "examples outside the PAC crate name no PAC type"
else
  fail "examples outside the PAC crate name a PAC type: $(echo "$offenders" | tr '\n' ' ')"
fi

# 2. A BSP must not re-export the PAC. An application holding it can reconfigure what the board
#    claimed to own, and every layer above becomes advisory.
offenders=$(grep -rln "pub use efr32mg24_pac" crates/*/src/ 2>/dev/null | grep -v "^crates/efr32mg2x-hal/" || true)
if [ -z "$offenders" ]; then
  pass "no BSP re-exports the PAC"
else
  fail "a BSP re-exports the PAC: $(echo "$offenders" | tr '\n' ' ')"
fi

# 3. The HAL must not put a register block in a public signature. The moment it does, every
#    caller needs a PAC handle to use a driver.
if grep -rqE "pub (fn|const fn) [^\n]*pac::[a-z0-9_]*::RegisterBlock" crates/efr32mg2x-hal/src/ 2>/dev/null; then
  fail "the HAL exposes a register block in a public signature"
else
  pass "the HAL exposes no register block publicly"
fi

# 4. Every crate in the hardware stack carries at least one example, and a crate with hardware
#    in it carries examples for each level it exposes. A crate with zero examples passes every
#    other check trivially and its API rots in silence.
for dir in crates/*/ libraries/*/; do
  [ -f "${dir}Cargo.toml" ] || continue
  name=$(basename "$dir")
  case "$name" in
    cmsis-stubs|efr32mg24-bootloader|bootloader-test-app) continue ;;
  esac
  # one example is a file, or a directory with a main.rs; a project's modules are not examples
  n=$(find "$dir/examples" -maxdepth 1 -name '*.rs' 2>/dev/null | wc -l)
  for d in "$dir"examples/*/; do [ -f "${d}main.rs" ] && n=$((n + 1)); done
  if [ "$n" -gt 0 ]; then
    pass "$name: $n examples of its own level"
  else
    fail "$name: no examples -- its API is not built by anything"
  fi
done


# 5. An example may not steal a core peripheral. SysTick and the cycle counter belong to the HAL
#    (`board.delay()`, `hal::dwt`), for the same reason the PAC does: an example that steals one
#    has no owner for it, and two owners of SysTick is a delay that lies.
offenders=$(grep -rln "Peripherals::steal" crates/*/examples/ --include=*.rs 2>/dev/null \
  | grep -v "^crates/efr32mg2x-pac/" \
  | grep -v "^crates/efr32mg24-bluetooth-sys/" \
  | grep -v "^crates/efr32mg24-rail-sys/" || true)
if [ -z "$offenders" ]; then
  pass "no example steals a core peripheral"
else
  fail "an example steals a core peripheral: $(echo "$offenders" | tr '\n' ' ')"
fi


# 6. A BSP example names only its own crate. Reaching for `efr32mg2x_hal::` inside a BSP
#    example makes it an example of the layer below, living in the wrong crate -- the same
#    defect as a HAL example that names the PAC. The BSP re-exports the HAL, so the path
#    `xiao_mg24_bsp::hal::...` is available and keeps the name in the right crate.
offenders=$(grep -rln "efr32mg2x_hal::\|efr32mg24_pac::" crates/*-bsp/examples/ 2>/dev/null || true)
if [ -z "$offenders" ]; then
  pass "BSP examples name only their own crate"
else
  fail "a BSP example names the layer below: $(echo "$offenders" | tr '\n' ' ')"
fi

# 7. A crate that declares the PAC must use it.
#
# Measured 2026-11-25: both BSP crates listed `efr32mg2x-pac` as a dependency and neither contained a single `pac::`. They
# were going through the HAL correctly -- and the unused dependency was a door left open, because the next person who
# cannot find a HAL method has the PAC one `use` away and no signal that they crossed a layer. A dependency nobody uses
# is not neutral: it is the shortest path to the wrong one.
#
# Crates that legitimately sit below the HAL are exempt by name, not by pattern.
pac_unused=""
for dir in crates/*/; do
  [ -f "${dir}Cargo.toml" ] || continue
  name=$(basename "$dir")
  grep -q 'efr32mg2x-pac' "${dir}Cargo.toml" || continue
  case "$name" in
    efr32mg2x-pac|efr32mg24-bootloader|bootloader-test-app|embassy-efr32|cmsis-stubs) continue ;;
  esac
  uses=$(grep -rc 'pac::' "$dir/src" 2>/dev/null | awk -F: '{s+=$2} END {print s+0}')
  [ "${uses:-0}" -eq 0 ] && pac_unused="$pac_unused $name"
done
if [ -z "$pac_unused" ]; then
  pass "every crate that declares the PAC uses it"
else
  fail "declares the PAC and never uses it:$pac_unused"
fi

echo "=== host tests: domain crates ==="
# Every crate under libraries/ must build and test on the host, and must not drag in
# anything from the hardware stack. That second half is the rule that keeps them testable:
# one dependency on a HAL turns `cargo test` into a cross-compilation.
for dir in libraries/*/; do
  [ -f "${dir}Cargo.toml" ] || continue
  name=$(basename "$dir")
  pkg=$(pkg_name "$dir")

  if cargo test -p "$pkg" --target "$HOST" >"/tmp/gate-$name.log" 2>&1; then
    n=$(grep -oE '[0-9]+ passed' "/tmp/gate-$name.log" | head -1)
    pass "$name: host tests (${n:-passed})"
  else
    fail "$name: host tests"
  fi

  # Doc-tests, because a code block in a doc comment is documentation a reader will copy, and until 2026-11-25 nothing
  # compiled one. `cargo test` does not run them; only `--doc` does. A crate with no doc-tests reports "0 passed", which
  # is honest: the check exists so that a doc-test that stops compiling is a red gate rather than a broken page.
  if cargo test -p "$pkg" --doc --target "$HOST" >"/tmp/gate-$name-doc.log" 2>&1; then
    n=$(grep -oE '[0-9]+ passed' "/tmp/gate-$name-doc.log" | head -1)
    pass "$name: doc-tests (${n:-0 passed})"
  else
    fail "$name: doc-tests"
  fi

  # The hardware stack, as a dependency, in any kind: normal or build.
  #
  # Until 2026-11-22 this check could not run and printed "pass" anyway -- three faults, each hiding the next:
  # `-p "$name"` asked cargo for the *directory* name (`adc-core`) instead of the package name
  # (`efr32mg2x-adc-core`); `-e no-dev` cannot be combined with `--edges normal,build`, so cargo exited 101 and
  # printed no tree at all; and stderr went to /dev/null, so grep found nothing and the `else` branch took the pass.
  # It is the guard of the premise that these crates test on the host, and it has never guarded anything.
  #
  # A check that cannot run is a FAIL, never a pass: the pass is what certifies the defect it exists to catch.
  #
  # Near-misses that must still take the normal path, and were shown to against real `cargo tree` output: a library
  # depending on another library of the same family (`efr32mg2x-adc-core` -> `efr32mg2x-sysrtc-core`), and the root
  # line itself, whose name starts `efr32mg2x-` -- which is why line 1 is skipped.
  if ! arvore="$(cargo tree -p "$pkg" -e normal,build --prefix none 2>&1)"; then
    fail "$name: cargo tree failed, so this check could not run"
  elif printf '%s\n' "$arvore" | tail -n +2 \
      | grep -qE '^(efr32mg2x-(pac|hal)|efr32mg24-|motion-detection|cortex-m|xiao-)'; then
    fail "$name: depends on the hardware stack"
  else
    pass "$name: no hardware dependency"
  fi
done

echo
echo "not gated on purpose:"
echo "  cmsis-stubs              -- a build-support crate: C only, no Rust API, no examples"
echo "  efr32mg24-radio/ble_gatt -- needs the PSA crypto blobs; documented in docs/04-reference/RADIO-NOTES.md"
echo
if [ "$FAILED" -eq 0 ]; then
  echo "VERDICT: PASS"
else
  echo "VERDICT: FAIL"
fi
exit "$FAILED"
