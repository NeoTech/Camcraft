---
name: takeover
description: Continue another author's stalled CamCraft PR only with recorded approval or a stale claim. Check claims, branch ownership, maintainer edits, refresh timing and exact heads; preserve commits and provide a patch if a push is refused.
---

# takeover - continue a PR without competing with its author

Use this skill for one named PR, not a sweep of old PRs. Read
[AGENTS.md](../../../AGENTS.md), [gh-issues](../gh-issues/SKILL.md),
[issue-claims](../issue-claims/SKILL.md) and the
[ownership guard](../issue-claims/references/ownership.md) first.
AGENTS.md takes precedence. This procedure grants no repository permissions.

## 1. Establish whether a takeover is allowed

1. Read the complete issue and PR threads, reviews, labels and commits.
   Search for other PRs with the same head repository and branch.
   Record the issue, PR, head repository, branch, base and full exact SHAs.
2. Stop during a release cut (an open `release/*` PR). Do not take a `P?`,
   `blocked` or `needs-research` issue for implementation.
3. Require the author's explicit approval in the thread, or evidence that
   the claim is stale under **A claim must be live**. Link that evidence.
   The age of the PR, silence on the PR, or a missing label is not enough.
4. A contributor's claim remains live if any of these is true:
   - The claimant has an open PR for the issue, even an old draft.
   - The claimant's branch has a commit in the last 48 hours.
   - The claimant commented on the issue in the last 48 hours.
   Only when all three are absent can it be stale. Owner-account agent
   claims use the separate 4-hour rule in `issue-claims`.
5. Do not push while the author is active. Approval alone does not release
   an active writer or reviewer. Obtain an explicit handoff and release.
   A released writer record does not release a live issue claim. If the
   claimant changes, record the old claim's withdrawal or release, then a
   new claim. An ended claim never revives without a new claim line.
6. Follow the ownership guard before dispatch or local work. A live writer
   or counted reviewer means stand down and name the owner and record URL.
   A counted reviewer has no expiry timer. Two active records mean STOP
   until one owner releases. Failed or incomplete reads also mean STOP.

## 2. Record one writer and prepare the existing branch

1. Use `work-issue`'s existing-PR path. Keep the issue and its branch;
   do not create a competing fix or overwrite the author's history.
   Post or confirm one writer record on the upstream issue, and link it
   from the PR. Include the approval or stale-claim evidence.
2. Read the record back by its comment URL. Re-read the complete threads
   and the exact remote head. Confirm the sole named writer before edits.
3. Fetch the author's exact head into a worktree under `.claude/worktrees/`.
   Check that it equals the recorded `Starting head` and `Expected remote`.
   Preserve the author's commits. Never rebase or force-push, including
   `--force-with-lease`. Do not switch or edit the main checkout.
4. Check GitHub's `maintainer_can_modify` field for the PR. Maintainer edits
   let a base-repository maintainer update a permitted fork branch; the
   flag does not give an arbitrary contributor push access. Confirm the
   actual account, head repository and exact branch. Never infer access
   from the flag alone. Do not use the author's credentials.
5. Preserve the draft rules. New PRs open as drafts. An orange `conflicts`
   verdict requires the `resolve-conflicts` draft-before-push procedure,
   including its SAME exception. Ordinary review fixes keep a ready PR
   ready under **A review finding does not make a PR a draft again**.
   A takeover does not authorize an extra draft/ready cycle.

## 3. Make and check the bounded change

- Use [work-issue](../work-issue/SKILL.md) for the fix and its pinning tests.
  For a bug, load [repro-first](../repro-first/SKILL.md) before the fix.
- A takeover is not permission to refresh a branch. **Refresh only at merge
  time** still applies: the author's agents handle **Own blocked PRs
  first**, the PR is next in the merge batch, or the author chooses a
  refresh while pushing a review fix. Otherwise leave the base alone.
- For an allowed refresh, use
  [resolve-conflicts](../resolve-conflicts/SKILL.md). Merge, never rebase.
  Stop if both sides change the same logic or a safety rule can be lost.
  Compile affected code with `cargo check --tests -p <affected-package>`,
  one package at a time, including incoming base changes and shared inputs.
  A prose-only refresh gets focused documentation checks, not a code pass.
- Run the applicable checks on the final committed tree. Record the exact
  head, commands, results and exclusions. A zero-step quick plan is not
  proof that documentation or code passed tests.

## 4. Publish once, or provide the patch

1. Immediately before a push, repeat the ownership guard's pre-push reads.
   Re-read the claims, writer/reviewer records, PR head and remote branch.
   Do not build or edit between those reads and the push. A moved head
   means STOP; preserve the local commits and report both exact SHAs.
2. With confirmed maintainer-edit permission and a completed handoff, push
   plainly to the recorded author's branch, not to upstream `main`.
   A rejected push means STOP. Do not retry with another credential,
   rebase, reset, branch deletion or force-push.
3. If GitHub refuses the push, preserve the worktree and local commits.
   Make a repository-relevant patch with
   `git format-patch --stdout <starting-head>..HEAD`.
   Post it in a PR comment as a fenced patch, or attach the patch and link
   it from that comment if it exceeds the comment limit. Check that it
   contains only the intended commits and no private data. Do not silently
   open a replacement PR or push to a different branch.
4. Write one result comment that names the starting head, each added
   commit, the reason for each change, checks and any refusal. For a patch,
   include its application base and state that the remote did not change.
   For a successful push, verify remote and PR heads against local HEAD;
   record `Published head` and `Expected remote` against the writer URL.
5. Release writer ownership with its record URL, final exact head and the
   published result or stopped reason. Read back the comment and patch.
   Keep the issue open until its fix merges. Hand off for independent
   review; never count your own pre-check as a collaborator review.

## Completion check

- Approval or all required stale-claim evidence is linked.
- The author is not writing; the old claim and owners have valid releases.
- One writer, the exact remote head and refresh permission were checked.
- Original commits remain; no force-push or competing branch was used.
- The result comment traces commits, reasons, checks and any refused push.
- Remote publication or the patch was read back; ownership was released.
