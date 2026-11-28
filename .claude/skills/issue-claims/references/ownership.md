# Branch writers and PR reviewers (#3661)

This is a cooperative ownership guard, not an atomic distributed lock.
GitHub comments are the shared record. A local file cannot coordinate two
machines. Read this procedure before dispatch, before local work without a
worker, and immediately before a push. AGENTS.md and
[issue-claims](../SKILL.md) still govern claim lifetimes, withdrawals and
refresh timing. Ownership grants no new push, review or merge permission.

## Record and read

Use the upstream issue thread for the canonical writer record. Extend the
normal `claimed:` comment; do not create a second issue claim for a worker
already dispatched under it. The dispatcher writes the record on behalf of
one named worker, then passes its URL. The worker confirms that same record.
A self-started worker performs the dispatcher steps itself. It posts one
claim with its writer record only when a new claim is needed. For a live
claim of the same claimant, link the new writer record to that claim after
the previous writer releases. A writer record must come from the claimant
or its verified dispatcher and name that claimant's worker. Another
account's unsupported record cannot replace or extend the claim.
A coordinator is not a second writer and must not push the worker's commits.

Use the upstream PR thread for the canonical review record, including a
delta review. Link the issue's writer record from the PR when it exists.
Before a PR exists, use the issue only. If a PR has no issue, put its writer
record on the PR. Search other PRs with the same head repository and branch;
the writer scope is the branch, not a PR number. Read their records too.

Only collaborator-authored reviewer records count, as in AGENTS.md
"Only collaborators' reviews count". Verify both the record author and the
reviewer's GitHub login against the base repository's collaborators with
`permissions.push` (use the paginated collaborator API from AGENTS.md).
The dispatcher must use a collaborator account to post a counted review
record. A fork owner is not a base collaborator by owning the fork.
A contributor pre-check uses no reviewer record, no counted review marker
and no review labels. A known non-collaborator record does not block a
collaborator review, whatever its stated role or age. Failed authority
reads mean STOP; do not infer collaborator status from a marker or label.

Read the complete threads, not only the latest comment or search snippets:

```bash
gh api --paginate repos/NeoTech/Camcraft/issues/ISSUE/comments
gh api --paginate repos/NeoTech/Camcraft/issues/PR/comments
gh pr view PR --repo NeoTech/Camcraft --json headRefOid,headRefName,headRepository,baseRefName,baseRefOid,state,isDraft
git ls-remote HEAD_REPOSITORY_URL refs/heads/BRANCH
```

Replace the uppercase tokens with the exact issue, PR, repository URL and
branch. Skip the PR reads only when no PR exists. Read the issue labels,
assignment and claim history as well. Failed or incomplete reads mean STOP,
not "no owner". A branch read succeeds with no matching ref only when the
branch is absent; an API 404 does not prove absence.

Append this block to the writer's claim or to a PR review comment. Do not
use `claimed:` for a review; it must not change the issue claim.

```text
<!-- ownership: writer | reviewer -->
Owner: <GitHub login / session / unique worker identity>
Dispatcher: <GitHub login / session>
Issue: <upstream issue URL, or none>
PR: <upstream PR URL, or not yet open>
Branch: <head repository owner/name> <exact branch>
Base: <base repository owner/name> <base ref> <full exact SHA>
Starting head: <full exact SHA>
Expected remote: <full exact SHA, or absent>
Started: <UTC ISO time>
State: active
```

Choose exactly one role in the marker. For a new branch, `Starting head` is
the local base commit and `Expected remote` is `absent`. Otherwise both name
the fetched branch head. Never substitute a short SHA, branch name or
`FETCH_HEAD` for an exact SHA. Use a unique worker identity even when two
agents share one GitHub account. Record the actual PR head and base, not an
assumed `main`. Review ownership is PR-wide, not just per head.

## Before dispatch

1. Read the issue claim and all applicable ownership records. Use
   issue-claims to decide whether the issue claim is live or ended. Apply
   the author checks above before treating a record as an owner.
2. A live writer means stand down and name the owner and record URL.
   A live reviewer means stand down and name the owner and record URL.
   Do not dispatch a writer during a review, or a reviewer during writing.
   The already named worker may resume its own active record; another agent
   under the same account may not.
3. If free, append the record. Read it back by its comment URL. Re-read the
   threads and remote head after posting and before dispatch. Confirm the
   expected head and sole owner. Pass the record URL, base and starting
   exact head to the worker. If two active records appear, both sessions
   STOP until one owner explicitly releases. Do not select a winner by
   comment order or overwrite another record.
4. The worker repeats the reads before it edits or reviews. A missing or
   mismatched record means STOP. No record from an older process is not
   permission to ignore an existing live claim: name that claimant and stop.

Writer liveness follows the existing issue claim rules: 4 hours for an
owner-account agent and the three 48-hour activity tests for a contributor.
These are the only activity tests; there is no writer-specific timeout.

| Contributor claim evidence (not withdrawn or released) | Liveness |
|---|---|
| Claimant has an open PR for the issue | Live |
| Claimant's branch has a commit in the last 48 hours | Live |
| Claimant commented on the issue in the last 48 hours | Live |
| All three absent | Stale; follow issue-claims release steps |
| Withdrawn or released, even with recent activity or an open PR | Ended; only a new claim can start again |

An ended claim stays ended; later activity cannot revive it. A new writer
needs a new claim after a claim ends. Releasing writer ownership after a
push does not release the issue claim; the same claimant can record its
next review-fix worker without reclaiming against itself. A different
claimant must still stand down while that issue claim is live.
A released writer record stays released even while its issue claim is live.
An unreleased writer record cannot outlive an ended or stale issue claim.

A counted review record remains active until its owner posts a result or
release. It has no expiry timer. Age alone does not release a running review.
If a counted review appears abandoned, confirm failure with its dispatcher.
If failure cannot be confirmed, STOP and name the owner; the owner or
maintainer must release it. Append the confirmed failure and release before
a replacement record. Keep `.claude/pr-review.lock` and its existing
one-review rule, including its local stale-lock checks. A free local lock
does not release a counted public owner.

## Immediately before a push

For a refresh, run at least `cargo check --tests -p <affected-package>` for
each affected code package, including incoming base changes. A clean merge,
SAME classification or zero-step quick plan does not replace compilation.
For shared inputs, including `Cargo.lock`, check each affected package one
package at a time with `-p`; never `--workspace` or an unqualified
full-workspace build.
Record prose-only scope when no code is affected.

Finish the refresh compile check and targeted tests on the final committed
tree first. Any later tree change invalidates those results. Then do the
final owner and remote reads below without a build or edit between them
and the push.

1. Re-read the public owner and claim records and the PR head. Confirm that
   you are still the sole writer and that no reviewer is active.
2. Read the exact remote branch with `git ls-remote` again. It must equal
   `Expected remote` in your active record (or still be absent for the first
   push). If the remote head moved, STOP. Report the expected and observed
   SHAs, owner URL and saved local head. Do not merge that unexpected head
   and retry. Never force-push, including `--force-with-lease`.
3. Push plainly to the recorded repository and branch. A rejected push is
   STOP. Preserve the worktree and local commits; report the rejection.
   Do not retry by rebasing, resetting, deleting or force-pushing.
4. Read the remote exact head and PR back. Confirm that both equal your
   local published head. Append a progress comment referencing the owner
   URL with `Published head: <full SHA>` and `Expected remote: <full SHA>`.
   That verified progress updates the expected remote for this owner only.
   If confirmation fails, STOP; do not announce success or dispatch review.

Before posting a review or changing its labels, re-read the records and PR
head. If the head moved, STOP without a verdict for the new head. Preserve
findings for the reviewed SHA; release the record before a fresh dispatch.

## Release and handoff

Append a comment referencing the exact owner record URL:

```text
Ownership released: <record URL>
Role: <writer or reviewer>
Owner: <same unique identity>
Final head: <full SHA>
Result: <published PR/review URL, or stopped with reason and saved work>
```

The named owner or maintainer releases the record. A dispatcher may record
its worker's completion only with a verified result or confirmed failure.
Do not edit or delete another owner's comment. Release on completion,
withdrawal or confirmed failure, including when escalation stops work.
Handoff requires release, fresh reads and a new named owner record. An issue
claim release uses the existing issue-claims texts separately.

## Guarantee and limits

The guard makes ownership and the expected head visible to cooperating
sessions. It requires stand-down on observed conflicts and moved heads.
It cannot serialize simultaneous GitHub comment writes or make a read and
push atomic. A competing write after the last read can still race. Git's
plain push rejects non-fast-forward updates; it is not an exact-head lease
and can accept a fast-forward after a race. Never describe this procedure
as race-free exclusion. No daemon, service or new dependency is required.

Contract checks pin the dispatch actors and skill links, required record fields, stop
instructions, lifecycle rules and refresh compilation. They test the
published procedure, not execution by an agent or distributed exclusion.

| Observed case | Required action |
|---|---|
| Live issue claimant, no owner block yet | Stand down; name the claimant |
| Live writer in another session | Stand down; name the writer |
| Live counted PR reviewer, same or different head | Stand down; name the reviewer |
| Known non-collaborator reviewer record | Does not block; contributor pre-checks claim no reviewer ownership |
| Withdrawn or stale issue claim | Follow issue-claims; only a new claim can start ownership |
| Writer released, issue claim still live | Same claimant can name its next worker; others stand down |
| Two active records after readback | Both stop; require explicit release |
| Remote moved, even if the local tip contains it | Stop; preserve local work; never force |
| Rejected push | Stop; preserve local work; never force |
| Clean refresh with affected code | Compile and test before push |
| Failed or incomplete owner/head read | Stop; do not infer that it is free |
