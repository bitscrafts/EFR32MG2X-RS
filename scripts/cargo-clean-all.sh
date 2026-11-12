#!/usr/bin/env bash
# cargo clean across the workspace's Rust projects, measured before and after.
#
# ## Why this script names the target directory explicitly
#
# The first version asked `cargo metadata` for each project's target directory and cleaned that. On this
# workspace that is dangerous: one project's `.cargo/config.toml` points its target dir at
# `/projects/_cache/targets`, which every build of that workspace shares. The loop therefore cleaned that
# shared cache on its FIRST project and then reported "no target dir" for all the others — the sizes it
# printed did not match the folders a reader would look at.
#
# So: each project's own `./target` is cleaned by explicit path, and the shared cache is handled
# separately and named out loud.
#
# ## What it refuses
#
# A project with an autonomy cycle running (`cycle.sh` holding a lock) is skipped: cleaning its target dir
# destroys the artefact a gate in flight is using, and a cold gate on that workspace is ~40 GB.
set -uo pipefail

SHARED=/projects/_cache/targets
before=$(df --output=used -BG /projects | tail -1 | tr -dc '0-9')
echo "disco usado antes: ${before}G"
echo

for project in /projects/*/; do
  [ -f "${project}Cargo.toml" ] || continue
  name=$(basename "$project")
  target="${project}target"
  [ -d "$target" ] || continue

  if pgrep -f "${project}.*scripts/autonomy/cycle.sh" >/dev/null 2>&1; then
    printf '%-24s REFUSED (autonomy cycle running) %s\n' "$name" "$target"
    continue
  fi

  size=$(du -sh "$target" 2>/dev/null | cut -f1)
  out=$(cargo clean --manifest-path "${project}Cargo.toml" --target-dir "$target" 2>&1)
  if [ $? -eq 0 ]; then
    printf '%-24s cleaned %-6s %s\n' "$name" "$size" "$target"
  else
    printf '%-24s FAILED: %s\n' "$name" "$(echo "$out" | tail -2 | tr '\n' ' ')"
  fi
done

# The shared cache, on request only: SHARED_CACHE=1 also cleans it.
# `cargo clean`, never `rm -rf`: the tool knows what belongs to the build and the project's rule about
# destructive filesystem commands has no exception for caches that happen to be regenerable.
if [ -d "$SHARED" ] && [ "${SHARED_CACHE:-0}" = "1" ]; then
  size=$(du -sh "$SHARED" 2>/dev/null | cut -f1)
  cargo clean --manifest-path /projects/efr32mg24-rs/Cargo.toml --target-dir "$SHARED" >/dev/null 2>&1
  printf '%-24s cleaned %-6s %s\n' "shared cache" "$size" "$SHARED"
fi

after=$(df --output=used -BG /projects | tail -1 | tr -dc '0-9')
echo
echo "disco usado depois: ${after}G   (libertado: $((before - after))G)"
df -h /projects | tail -1
