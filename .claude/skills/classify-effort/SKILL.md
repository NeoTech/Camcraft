---
name: classify-effort
description: Set or check the `effort:low|medium|high|max` label on NeoTech/Camcraft issues, by AGENTS.md's effort table and safety floor. `/classify-effort <N>` classifies one issue; `/classify-effort` labels every open issue that has no effort label; `/classify-effort audit` reports open issues whose label looks wrong without changing them. Use when filing or triaging an issue, when an issue has no effort label, before dispatching an issue-worker, or when asked to "classify effort", "weight this issue", "what effort is #N", "label the unlabelled issues".
---

# classify-effort - one effort label per issue, by the table

The effort label says how hard the **work** is. It picks the worker agent
(`.claude/agents/issue-worker-<effort>.md`) and so the model and the token
spend; `P*` still decides which issue is worked next. AGENTS.md ("Every
issue is weighted by effort") is the law; this skill is how to apply it
the same way every time. Load the `gh-issues` skill for the `gh` mechanics.

## What you may change

- **Set** a label on an issue that has none (filing, triage, the sweep).
- **Never lower** an existing label - only the maintainer does. If one
  looks too high, say so (audit mode, or one issue comment); don't edit.
- **Raise** an existing label only through `work-issue`'s escalation (a
  worker that hit the limit), one step, with its comment. Not here.
- **Never touch** an issue labelled `P?` (paused) - read it, nothing more.
- A `needs-research` umbrella gets no effort label; its sub-issues do.
- Exactly one `effort:*` label per issue. Two is a bug: report it.

## How to classify (in this order)

1. **Read the whole issue**: `gh issue view N --comments`. The body gives
   the ask and "Done means"; the comments give what is already done, what
   earlier attempts found, and any split into sub-issues.

2. **Classify the work that is left, not the title.** If earlier PRs
   merged parts of it, weigh only what remains. If the issue is an
   umbrella worked in bounded stages (each PR `Refs #N`), weigh the next
   stage a worker would take, not the whole programme (maintainer,
   2026-11-11: four old roadmap issues labelled `max` were moved to `high`
   because the work actually handed out was a bounded stage, not open
   design).

3. **Match it to the table row it most resembles:**

   | Label | The work | Typical signs |
   |---|---|---|
   | `effort:low` | mechanical, one file, no new logic | docs, a label or string, a layout tweak, a test fixture or field fix |
   | `effort:medium` | one module or crate, a known pattern | a UI page change, an MCP tool over an existing command, a new machine file, a release-script edit that reuses existing code |
   | `effort:high` | several crates, new behaviour with its tests | a new undoable command with its MCP tool, simulation or post logic, a refusal law, a bounded stage of a large programme |
   | `effort:max` | new algorithms, formats or open design | a new geometry or toolpath algorithm, a new collision law, a new file format, a piece whose design is still open |

   Questions that separate the rows:
   - How many crates must change? One -> at most `medium` (before the
     floor); several -> `high`.
   - Is there an existing pattern to copy (another page, tool, post,
     machine file)? Yes -> `medium`. New behaviour with new tests -> `high`.
   - Does one piece of it need an algorithm or format that does not exist
     in the tree and has to be designed from literature? Only then `max`.

4. **Apply the safety floor.** At least `effort:high`, however small the
   diff, when the work can change: posted G-code, a collision or gouge
   check, stock simulation, or geometry. Every `P0` is at least
   `effort:high`. The floor only raises; it never lowers.

5. **Stop there.** These are *not* reasons to go up a row:
   - it is hard to test, or the area "feels risky" (release pipeline,
     build scripts) but is outside the safety floor;
   - the issue text is long, or the issue is old;
   - earlier attempts failed - that calls for a better brief or a
     maintainer decision, not automatically a bigger model;
   - "to be safe". Inflating the label spends the expensive model on
     every such issue.
   And these are not reasons to go down: the diff is small (the floor
   still applies), or the fix "looks obvious" in safety-floor code.

6. **If it is too big for any single run** (several independent
   deliverables, open questions the maintainer must answer), the right
   move is not `max`: propose a split into sub-issues, or `needs-research`
   for the maintainer to confirm, in an issue comment.

## Writing the label

```bash
gh label list --limit 200 | grep '^effort:'        # the four must exist
gh issue edit N --add-label effort:<x>
gh issue comment N --body "effort:<x> - <one line: the table row it matches, and 'safety floor' if that set it>"
gh issue view N --json labels --jq '[.labels[].name]'   # verify
```

The one-line reason is required: it is what lets the table be
recalibrated later (AGENTS.md "Measured").

## Modes

### `/classify-effort <N>` - one issue
Steps 1-6, then write the label if the issue has none. If it already has
one: report whether you agree, and change nothing.

### `/classify-effort` - the sweep
```bash
gh issue list --state open --limit 400 \
  --search '-label:effort:low -label:effort:medium -label:effort:high -label:effort:max -label:"P?" -label:needs-research' \
  --json number,title,labels --jq '.[] | "\(.number)\t\(.title)"'
```
Classify each (steps 1-6) and label it. More than a handful: print the
list with the proposed label and reason first, then apply. Report a
table at the end: issue, label, reason.

### `/classify-effort audit` - read-only
For every open issue that has an effort label (skip `P?`), apply steps
1-5 and list only the disagreements: issue, current label, the label the
table gives, the reason. Change nothing; the maintainer decides. Group
the list as "looks too high" (token spend) and "below the safety floor"
(the second kind is the urgent one - say so first).

## Report

One line per issue: `#N effort:<x> - <reason>` (and `set` / `already
set, agree` / `already set, would be <y>`). Name any issue you skipped
and why (`P?`, `needs-research`, two labels).
