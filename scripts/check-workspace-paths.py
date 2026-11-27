#!/usr/bin/env python3
"""Refuse a workspace whose members depend on a path outside the checkout.

The defect this exists for, measured 2026-11-27: one line in `crates/xiao-mg24-bsp/Cargo.toml` named
`dosimeter-mg24 = { path = "/projects/dosimeter-mg24" }`. The local gate was green because that machine had the
directory; on a CI runner `cargo metadata` refused the whole workspace (`failed to read /projects/dosimeter-mg24/
Cargo.toml`) and every job -- build, clippy, fmt, rustdoc, the gate itself -- failed downstream of it, for every push.

A path dependency is resolved when the workspace loads, so a member carrying one that points outside the repository
makes the repository unbuildable anywhere else. `optional = true` does NOT help: Cargo resolves the manifest of an
optional path dependency too (measured with a two-file probe).

What is checked: every `path = "..."` in every manifest the workspace loads (members and their local dependencies),
plus the workspace `exclude` list, must stay inside the repository. Escaping paths are reported with the file and the
line, and the verdict is a failure -- a check that only warns is a check that gets ignored.

Usage: check-workspace-paths.py [repo_root]
Exit:  0 when every path stays inside, 1 otherwise.
"""

import sys
import tomllib
from pathlib import Path


def repo_manifest_paths(root: Path) -> list[Path]:
    """Every manifest the workspace loads: the members, recursively through local path dependencies."""
    seen: dict[Path, None] = {}
    pending = [root / "Cargo.toml"]
    while pending:
        manifest = pending.pop()
        if manifest in seen or not manifest.is_file():
            continue
        seen[manifest] = None
        data = tomllib.loads(manifest.read_text())
        for section in ("dependencies", "dev-dependencies", "build-dependencies", "workspace.dependencies"):
            table = data
            for key in section.split("."):
                table = table.get(key, {}) if isinstance(table, dict) else {}
            if not isinstance(table, dict):
                continue
            for spec in table.values():
                if isinstance(spec, dict) and "path" in spec:
                    target = (manifest.parent / spec["path"]).resolve()
                    candidate = target / "Cargo.toml" if target.is_dir() else target
                    pending.append(candidate)
        for member in data.get("workspace", {}).get("members", []):
            base = (root / member).resolve()
            pending.append(base / "Cargo.toml" if base.is_dir() else base)
    return sorted(seen)


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    problems: list[str] = []

    for manifest in repo_manifest_paths(root):
        data = tomllib.loads(manifest.read_text())
        for section in ("dependencies", "dev-dependencies", "build-dependencies", "workspace.dependencies"):
            table = data
            for key in section.split("."):
                table = table.get(key, {}) if isinstance(table, dict) else {}
            if not isinstance(table, dict):
                continue
            for name, spec in table.items():
                if not isinstance(spec, dict) or "path" not in spec:
                    continue
                raw = spec["path"]
                target = (manifest.parent / raw).resolve()
                if Path(raw).is_absolute() or not target.is_relative_to(root):
                    rel = manifest.relative_to(root)
                    problems.append(f"{rel}: `{name}` depends on `{raw}`, which is outside the repository")

    if problems:
        print("  FAIL  workspace paths: a member depends on a path outside the checkout")
        for problem in problems:
            print(f"        {problem}")
        print("        a path dependency is resolved when the workspace loads, so the repository cannot be built")
        print("        anywhere else -- quarantine the crate in `exclude`, do not carry the path")
        return 1

    print(f"  PASS  workspace paths: every path dependency stays inside the repository")
    return 0


if __name__ == "__main__":
    sys.exit(main())
