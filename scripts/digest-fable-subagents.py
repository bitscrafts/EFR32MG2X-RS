#!/usr/bin/env python3
"""Turn the sub-agent transcripts of a dead review into a readable evidence dossier.

Why: the audits of 2026-11-17 read for 78-81 tool calls each and died on the account's five-hour limit without writing
a conclusion. Their sessions are not resumable (`--resume` refuses a non-UUID), but every file they read and every
vendor comparison they made is still in their transcripts. That reading has already been paid for; this makes it
usable, so the next session can write findings from evidence instead of re-reading 10 000 lines of Rust.

Output: one file per sub-agent, sections keyed by the file or command that was examined, each with a bounded excerpt.
Usage: digest-fable-subagents.py <parent-session-id> [out-dir]
"""
import json
import os
import re
import sys
import glob

MAX_EXCERPT = 2200          # per examined file: enough to judge a defect, not enough to flood a context
MAX_SECTION_PER_FILE = 3    # repeated reads of the same file: keep the three largest


def blocks(path):
    """Yield (kind, name, payload) for each tool call and its result, in order."""
    calls = {}
    results = []
    for line in open(path, errors="ignore"):
        try:
            d = json.loads(line)
        except Exception:
            continue
        msg = d.get("message") or {}
        content = msg.get("content")
        if isinstance(content, list):
            for b in content:
                if not isinstance(b, dict):
                    continue
                if b.get("type") == "tool_use":
                    name = b.get("name", "")
                    inp = b.get("input", {}) or {}
                    key = b.get("id")
                    target = (
                        inp.get("file_path")
                        or inp.get("path")
                        or inp.get("command")
                        or inp.get("pattern")
                        or ""
                    )
                    calls[key] = (name, str(target))
                elif b.get("type") == "tool_result":
                    key = b.get("tool_use_id")
                    body = b.get("content")
                    if isinstance(body, list):
                        body = " ".join(
                            x.get("text", "") for x in body if isinstance(x, dict)
                        )
                    results.append((key, str(body or "")))
    seen = set()
    for key, body in results:
        if key in seen or key not in calls:
            continue
        seen.add(key)
        name, target = calls[key]
        yield name, target, body


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    sid = sys.argv[1]
    out_dir = (
        sys.argv[2]
        if len(sys.argv) > 2
        else "/projects/efr32mg24-rs/_tmp/review-hal-fable/parts"
    )
    sub_dir = f"/root/.claude/projects/-projects-efr32mg24-rs/{sid}/subagents"
    if not os.path.isdir(sub_dir):
        sys.exit(f"no subagents dir at {sub_dir}")
    os.makedirs(out_dir, exist_ok=True)

    written = []
    for path in sorted(glob.glob(os.path.join(sub_dir, "agent-*.jsonl"))):
        agent = re.search(r"agent-([0-9a-f]+)\.jsonl$", path).group(1)
        desc = ""
        meta = path.replace(".jsonl", ".meta.json")
        if os.path.exists(meta):
            try:
                desc = json.load(open(meta)).get("description", "")
            except Exception:
                pass
        per_target = {}
        for name, target, body in blocks(path):
            if not body.strip() or len(body) < 200:
                continue
            key = target.strip().split("\n")[0][:160] or name
            per_target.setdefault(key, []).append((name, body))
        if not per_target:
            continue
        slug = re.sub(r"[^a-z0-9]+", "-", (desc or agent).lower()).strip("-")[:44] or agent
        target_file = os.path.join(out_dir, f"evidence-{slug}-{agent[:8]}.md")
        with open(target_file, "w") as fh:
            fh.write(
                f"# Evidence already read: {desc or agent}\n\n"
                f"<!-- distilled from {path} -->\n"
                f"<!-- {len(per_target)} distinct targets examined; each excerpt is bounded to {MAX_EXCERPT} chars -->\n\n"
                "This is reading that has already been paid for by an earlier audit that ran out of window before it\n"
                "wrote a single finding. Use it as the starting evidence for a new audit: verify only what a finding\n"
                "actually needs, and write each finding to the part file as you establish it.\n\n"
            )
            for key, items in sorted(per_target.items(), key=lambda kv: -len(kv[1])):
                body = max(items, key=lambda x: len(x[1]))[1]
                fh.write(f"## {key}\n\n")
                fh.write("```\n")
                fh.write(body[:MAX_EXCERPT].replace("```", "'''"))
                fh.write("\n```\n\n")
        written.append((slug, agent, len(per_target), os.path.getsize(target_file)))

    print(f"dossiers written to {out_dir}")
    for slug, agent, n, size in written:
        print(f"  {agent[:8]}  targets={n:3d}  {size:8d} B  evidence-{slug}-{agent[:8]}.md")


if __name__ == "__main__":
    main()
