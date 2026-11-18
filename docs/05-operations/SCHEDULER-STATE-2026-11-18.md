# Scheduler: what exists, what is broken, and why it is parked

**Written 2026-11-18, at the end of the session that built it.** The operator's words: *"já fiz várias
tentativas de ter um orchestrator/scheduler implementado e sempre por alguma maneira é falhada!
pidag, pi-orchestration, minilam entre outras e todas frustadas! essa é outra guerra!"*

This document exists so the next attempt starts from the failures, not from zero. It is deliberately
short: the long version of a system nobody uses is the same trap again.

## The record: four orchestrators, four failures

| system | what it was | how it failed |
|---|---|---|
| `pidag` | spec → DAG → workers | the DAG is generated, the workers run, the review is manual |
| `pi-orchestration` | pi implements specs under a gate | worked for a while; the harness grew its own docs and drifted |
| `minilam` | a step oracle holding a method as JSON | correct and well tested (172 tests), never adopted; it asks the agent to answer in words |
| this scheduler | queue + runner + cron, one item per tick | three root causes closed, four holes open (below) |

The pattern in all four is the same: **the machinery is finished before the work it was meant to
carry.** The HAL still has no core moved, and four orchestrators exist to move it.

## What exists today, and what is proven

```
docs/05-operations/QUEUE.md            C1..C6 ready, B1..B8 backlog        written
docs/05-operations/QUEUE-METHOD.md     the method, six sections + model rule   written
scripts/queue-run-once.sh              one item per invocation                 written, not proven end to end
~/.hermes/scripts/scheduler-efr32.sh   env, model, session                     PONG proven
cron a19d206385db                     every 30m, no_agent                     proven to fire, never to finish an item
```

Proven by measurement: `pi --model deepseek-v4.1-flash:cloud` answers `PONG` **only** when
`/root/.hermes/.env` is sourced inside the wrapper; the shell has the key, a background process does
not. `--provider ollama-cloud` breaks authentication (it looks in the environment, not in
`models.json`). The session file must be reused across items or every item starts blind.

## The four holes, honestly

1. **Context between specs is not guaranteed.** The worker's session carries it, and nothing else
   does. A compaction, a lost `.jsonl`, or an edit made outside the worker desynchronises it.
   *Fix:* a handoff file per item, written after the commit, fed into the next brief.
2. **A failed item leaves the tree dirty.** The next item runs `git add -A` and commits the previous
   item's half-work inside its own commit. This already produced commits with foreign files.
   *Fix:* commit only the paths in the spec `Files` block; on a red gate, save the diff as a patch
   under `_trash/attempt-<id>-<n>/` and reset the tree to HEAD before exiting.
3. **No attempt memory.** The next worker does not know how the previous one failed, so it repeats
   it. *Fix:* put the failed patch and the three `FAIL` lines into the next brief.
4. **`BLOCKED` does not stop anything.** It writes a log line and an alert. If nobody reads it, a
   stopped queue looks exactly like a working one. *Fix:* a `## BLOCKED` header in `QUEUE.md` plus
   `cronjob pause` from the runner itself.

And one defect of its own, `B8`, still open: the runner marks an item `- [x]` **before** committing,
and treats a gate pass over an unchanged tree as work done. Two items were marked done on
2026-11-18 with zero lines of code. They were reset to `- [ ]` by hand.

## Decision

**Parked.** The HAL has a deadline and the scheduler does not. It stays paused with this document as
its state, and it is resumed only if the HAL work ever produces a repeating item where a missed tick
costs more than a broken tick.

The rule that survives from this attempt, and it is worth keeping in every future one: *prove the
worker answers before giving it work, and prove the check fails before trusting it to pass.*
