---
name: repro-first
description: Demonstrate a reported CamCraft bug through MCP or a pinning test on current main before fixing it. Preserve the before reading for the PR, and stop with issue evidence when the bug does not reproduce or reproduction is unsupported.
---

# repro-first - measure the bug before changing its cause

Use this skill before a bug fix, including a takeover or review fix.
Read [AGENTS.md](../../../AGENTS.md), [gh-issues](../gh-issues/SKILL.md),
[work-issue](../work-issue/SKILL.md) and the
[ownership guard](../issue-claims/references/ownership.md).
AGENTS.md takes precedence. Reproduction is not permission to change an
unclaimed issue or a branch that another writer or reviewer owns.

## 1. Establish the baseline

1. Read the full issue and comments. Identify the input, steps, expected
   result and observed result. Search for an existing fix or duplicate PR.
   Check the claim, ownership, effort and release-cut prerequisites in
   `work-issue` before edits. Record the current process rules.
2. Fetch the canonical repository's latest `main`. Record its full SHA.
   In a fork, follow [fork-sync](../fork-sync/SKILL.md): this check does not
   merge upstream `main` into the fork's `main` or refresh an existing PR.
   Use an isolated worktree at the recorded upstream commit if necessary.
   Never switch or pull a checkout that is running a gate.
3. Keep the implementation unchanged while collecting the before reading.
   Record the operating system, build command, binary provenance and
   relevant feature flags. Do not use an old binary as evidence for the
   fetched `main`. Keep user data private; prefer a small original fixture.
4. For a takeover, also record the PR's exact starting head. A failure only
   on that branch is not a reproduced bug on current `main`. Report the
   distinction; do not refresh the branch merely to obtain a baseline.

## 2. Reproduce before writing the fix

Choose the smallest supported route:

- **MCP:** load [camcraft-mcp](../camcraft-mcp/SKILL.md). Use the shipped
  tools to create or open the minimal document and run the reported steps.
  Save the requests, actual responses, diagnostic or numeric reading, and
  expected result. The binary must come from the recorded baseline.
  A server connection or screenshot alone does not demonstrate the bug.
- **Pinning test:** add a focused assertion in an existing domain directory
  suite, in the lowest crate that provides its imports. Load
  [gate](../gate/SKILL.md). Run the named test against the unchanged baseline
  implementation and save its actual failure. The failure must assert the
  reported behaviour, not a build error, missing fixture or service outage.

Do not start the implementation fix until one route demonstrates the bug.
For either route, record the input and exact command so another contributor
can repeat it. Use only permitted clean-room inputs. Do not run generated
G-code on machinery as part of this software procedure.

### Outcomes that are not a reproduced bug

| Observation | Action |
|---|---|
| Current `main` gives the expected result | Stop the fix. Post the SHA, input, command, expected result and actual reading on the issue. Ask for the missing case through that evidence; do not invent a failure or close the issue. |
| The build fails, MCP lacks the required operation, or the fixture is unavailable | State that reproduction is unsupported or blocked, not that the bug is absent. Post the limitation and partial evidence on the issue. Stop until a supported test or input is available. |
| The issue changes prose only | Demonstrate the missing or incorrect text and run focused documentation checks. Mark runtime reproduction not applicable. Do not claim that MCP or a regression test failed. |
| A known-fail test already records the defect | Run that exact test and confirm the expected assertion fails on the baseline. An ignored test that was never run is not evidence. |

A new test committed red on purpose must use AGENTS.md's **Tests that start
red use the known-fail marker**, not plain `#[ignore]`. Keep the open issue,
full test name, target and expected panic prefix accurate. A temporary red
run for a fix does not justify shipping an unmarked failing test.

## 3. Fix, then repeat the same measurement

1. Continue `work-issue` only after the baseline demonstrates the defect.
   Preserve the input and assertion. Do not weaken the expected result to
   make the test pass, suppress a diagnostic, or remove a safety refusal.
2. Make the smallest fix. Run the same MCP sequence or named test again.
   Record the after reading and exact fixed head. For an MCP reproduction,
   add a pinning test where possible; state any remaining coverage limit.
3. Confirm that the test detects the defect: its recorded baseline run
   must fail at the relevant assertion and its fixed run must pass.
   If the assertion changed, repeat the baseline check in an isolated
   worktree. Do not overwrite unrelated work to manufacture a red run.
4. Run the selected quick gate and the other applicable checks from `gate`.
   A dry run or zero-step plan is not a test pass. Keep full heavy lanes,
   snapshots and release gates in their required lanes. Report unrelated
   failures separately; do not replace actual output with expected output.

## 4. Put the before reading in the PR

Use this structure with actual readings, not placeholder success text:

```text
Reproduction before fix:
- Baseline: <canonical main full SHA; build/binary provenance>
- Input and environment: <fixture path and relevant settings>
- Command or MCP requests: <repeatable sequence>
- Expected: <assertion, diagnostic or numeric result>
- Observed before: <actual result; log or evidence link>
- Pinning test: <full name and baseline assertion failure, or limitation>

After fix:
- Head: <full fixed SHA>
- Same measurement: <actual result; evidence link>
- Checks: <commands and actual results>
- Not exercised: <limits, unsupported routes or unrelated failures>
```

For prose-only work, replace the runtime fields with the before text,
location and focused checks. Keep the normal PR fields from `work-issue`,
including `Process change: yes` when applicable. Publish as a draft first.
Verify publication and release writer ownership before independent review.
A contributor pre-check is not a counted collaborator review.
