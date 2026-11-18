# The queue method

**What this is.** The single place that explains how work moves from a conversation into the tree
without a person prompting it. It exists because the same instruction was given three times in one
session ("prossiga", "corra", "deixe andar") — each time the work stopped when the words stopped.

**The rule that makes it work:** *anything we decide in conversation becomes a queue item in the
same turn it is decided.* A decision that is not in `QUEUE.md` did not happen; it is a sentence in a
terminal scrollback. The cost of writing the item is two minutes; the cost of losing the decision is
the whole conversation again.

---

## 1. The files

| file | who writes it | what it is |
|---|---|---|
| `docs/05-operations/QUEUE.md` | anyone, by hand | the ordered list of work |
| `specs/NN-<slug>.md` or `docs/05-operations/QUEUE-ITEM-<ID>-<slug>.md` | whoever adds the item | what "done" means for that item |
| `scripts/queue-run-once.sh` | the repository | does exactly one item, then exits |
| `docs/05-operations/QUEUE-RUN-LOG.md` | the runner | what happened, with the gate verdict |
| `_tmp/queue/state.json` | the runner | consecutive reds; nothing else |

## 2. The item format

```
- [ ] <ID> | <path-to-spec> | <one line: what done looks like>
```

- **Position is priority.** The runner takes the first unchecked line. To move something forward,
  move the line up; to park it, move it down. There is no separate priority field to disagree with
  the order.
- **The ID** is short and never reused: `C1`, `C2`, … for a series, `H17` for a review finding.
- **The spec path must exist** before the runner runs: it refuses an item without one, with
  `rc=2`, and touches nothing. That check is there because a worker given no spec invents one.
- Marking is the runner's job. Editing `[ ]` to `[x]` by hand is allowed only when the work is
  already committed and the gate already proved it.

## 3. What a spec must contain

Six sections, and the spec is rejected by review if any is missing:

1. **Overview** — why the work exists, in terms of the defect or the constraint, not the feature.
2. **Requirements R1..Rn** — each with a *worked example*: an input and an output someone can
   check without reading the code.
3. **Files** — every file the item will touch, in backticks. This is what stops a worker from
   writing in the wrong repository, which has already happened once.
4. **TDD contract** — a table: test name, input, expected output. The failing test comes from here.
5. **Exit criteria** — one shell command per requirement, no semicolons, each returning 0 on
   success. These are the only opinions that count.
6. **Guardrails** — what must not happen, phrased as prohibitions with the reason.

## 4. The cycle

```
queue-run-once.sh
  1  first unchecked item of QUEUE.md .......... none left -> "QUEUE EMPTY", exit 3
  2  dispatch the worker with the spec ......... pi --yolo, 50 min ceiling, log in _tmp/queue/
  3  run scripts/gate.sh ....................... rc, VERDICT line
  4  green   -> mark the line, commit as Marcelo Correa, reset the red counter
     red     -> commit nothing, the item stays, increment the red counter
  5  three reds in a row -> BLOCKED, exit 5, and stop trying on its own
```

Exit codes: `0` done · `1` gate red, item stays · `2` environment error · `3` queue empty ·
`5` blocked.

## 5. Who decides what, and what never happens here

- **The worker's report is a self-report.** The gate's verdict is the fact. A worker that says
  "all tests pass" and a gate that says FAIL means FAIL.
- **The queue never publishes.** No `cargo publish`, no tags, no pushes beyond the working branch.
  Publication needs an express order from the operator, in words, at the moment.
- **The queue never flashes.** Not even with `--confirm`. Hardware changes are a person's call.
- **The queue never deletes.** It marks, it commits, it moves things to `_trash/` with a timestamp.
- **No worker commits.** The runner commits, after the gate, with a human identity.

## 6. Adding an item, which is the point of this document

Any of these is an item, in the same turn:

- a defect found while doing something else (this session: the gate line that checked nothing);
- a decision taken in conversation ("the libraries do not go to crates.io");
- an instruction that is not yet actionable ("when the HAL is published, put `hal-examples` back");
- a boundary discovered by measurement ("the HAL compiles on the host" — so a test can live there).

Write the item, write the spec, put it at the end of `QUEUE.md`, and it will be done. If it is not
written, it will be re-derived, re-discussed, and re-decided at the cost of another session.

## 7. Why there is no daemon

The runner is a script and the scheduler is the gateway's cron: one process per item, no state
machine to corrupt, and the whole history readable in two files (`QUEUE.md`, `QUEUE-RUN-LOG.md`).
A queue that needs a service to be up is a queue that stops when the service does — and this one
had to keep working while a person slept.

---

## 8. The model is a command-line argument, never a default

`queue-run-once.sh` refuses to start without `QUEUE_MODEL`, and exits `2` with the accepted forms.
The reason is measured, not stylistic: the first version of this cron inherited the session's model,
picked Gemini, hit `429 RESOURCE_EXHAUSTED` in fifteen seconds, and died before the worker ran. An
implicit model is a model chosen for you at the worst moment.

```
QUEUE_MODEL=nvidia:deepseek-ai/deepseek-v4-flash-0731   worker chain, first choice
QUEUE_MODEL=deepseek:deepseek-chat                      paid top-up, last
forbidden: any ollama-cloud route (operator directive), gemini (quota exhausted)
QUEUE_MAX_TURNS=60                                      turn ceiling per item
```

The scheduler wrapper (`~/.hermes/scripts/fila-efr32.sh`) is the one place that names the model, so
`hermes cron list` plus that file answer "what will run and on what" without reading a session.

**Same rule for the scheduler itself.** The cron runs with `no_agent: true` and `script:` pointing
at the wrapper: no LLM in the loop, so no quota can stop the queue and the report is the script
stdout, not a summary of it.
