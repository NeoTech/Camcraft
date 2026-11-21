## #3133 - Own blocked PRs first: who fixes, and which PRs are skipped

**What was decided.**
- AGENTS.md has the new law **Own blocked PRs first**, in the Issue Law
  before **Claim before any code**.
- A listed PR whose issue has `claimed` is skipped: it is already in work.
- An issue worker does not fix a listed PR itself. It stops before the
  claim and reports the PR numbers; the coordinator sends a worker to each
  one (work-issue step 0). The coordinator runs the same check before each
  dispatch for a new issue.
- "Refresh only at merge time" keeps its rule for maintenance and
  reviewers; the author's agents refresh only under the new law.
- The #681 comment format gets a `reload` part: each running session
  reloads `AGENTS.md`, `.claude/agents/` and `.claude/skills/`, and a fork
  runs `fork-sync`.

**Why.** Without the `claimed` skip, a second worker sees a PR that a
first worker is fixing and stops or starts a second fix of it. A worker
holds one issue and one worktree; a fix of a different PR inside the same
run breaks that and the effort tier of the brief.

**Considered and dropped.** The worker fixes the listed PRs itself before
its issue: dropped, because the PR may be above its tier and it doubles
the worktrees of one run.

**Cost to change later.** A few lines in work-issue and the coordinator
agent file.
