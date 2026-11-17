#!/usr/bin/env bash
# Salvage the sub-agent audits of a Fable review run into standalone files.
#
# Why: the three audits run in their own contexts and return to the parent session, which is what writes the report.
# If the account's five-hour window closes before the parent aggregates them, everything they found exists only inside
# their transcripts. This copies it out while it is still there, so the work survives the window and can be assembled
# later — by the parent on resume, or by hand.
#
# Usage: salvage-fable-subagents.sh <parent-session-id> [output-dir]
set -u
SID="${1:?usage: salvage-fable-subagents.sh <parent-session-id> [output-dir]}"
OUT="${2:-/projects/efr32mg24-rs/_tmp/review-hal-fable/parts}"
PROJ="/root/.claude/projects/-projects-efr32mg24-rs/${SID}/subagents"

[ -d "${PROJ}" ] || { echo "REFUSE: no subagents dir at ${PROJ}"; exit 1; }
mkdir -p "${OUT}"

python3 - "${PROJ}" "${OUT}" <<'PY'
import json, os, sys, glob, re

subagents, out = sys.argv[1], sys.argv[2]
index = []
for path in sorted(glob.glob(os.path.join(subagents, "agent-*.jsonl"))):
    agent = re.search(r"agent-([0-9a-f]+)\.jsonl$", path).group(1)
    # The task description lives beside the transcript.
    desc = ""
    meta = path.replace(".jsonl", ".meta.json")
    if os.path.exists(meta):
        try:
            m = json.load(open(meta))
            desc = m.get("description") or m.get("task") or ""
        except Exception:
            pass
    texts, tools = [], 0
    for line in open(path, errors="ignore"):
        try:
            d = json.loads(line)
        except Exception:
            continue
        msg = d.get("message", {})
        content = msg.get("content")
        if not isinstance(content, list):
            continue
        for b in content:
            if not isinstance(b, dict):
                continue
            if b.get("type") == "text" and b.get("text", "").strip():
                texts.append(b["text"].strip())
            elif b.get("type") == "tool_use":
                tools += 1
    if not texts:
        continue
    slug = re.sub(r"[^a-z0-9]+", "-", (desc or agent).lower()).strip("-")[:50] or agent
    target = os.path.join(out, f"{slug}-{agent[:8]}.md")
    with open(target, "w") as fh:
        fh.write(f"<!-- salvaged from {path} -->\n")
        fh.write(f"<!-- task: {desc or '(no description)'} -->\n")
        fh.write(f"<!-- tool calls: {tools}, assistant text blocks: {len(texts)} -->\n\n")
        fh.write("\n\n---\n\n".join(texts))
    index.append((slug, agent, tools, len(texts), os.path.getsize(target), target))

print(f"salvaged {len(index)} agent(s) into {out}")
for slug, agent, tools, n, size, target in index:
    print(f"  {agent[:8]}  tools={tools:3d}  blocks={n:2d}  {size:7d} B  {os.path.basename(target)}")
PY
