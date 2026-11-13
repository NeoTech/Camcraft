## #1153 - The merging session is the one writer of PROCESS_VERSION

**What was decided.** A process PR no longer edits `.claude/PROCESS_VERSION`; its body says `Process change: yes` and the reviewer checks that line (and that the file is untouched). After a merge batch that merged at least one process PR, the merging session runs `cargo xtask process-bump` (+1, once per batch) in a worktree off the new `main`, merges a one-line `chore(process): PROCESS_VERSION <n>` PR, and posts the #681 announcement with that number. What a fork compares is unchanged: its `.claude/PROCESS_VERSION` against upstream's. This PR bumps the file once itself (62 to 63), as the issue's "Done means" asks, being the last PR to carry its own bump.

**Why.** On 2026-11-12 (about 50 merges) every open process PR conflicted on that one line each time another merged, and two PRs could carry the same number (#1117 and #930 at 17). With one writer there is nothing to conflict on and a number can only be given out once. The window between the merge and the bump is harmless: a fork that syncs in it pulls the same files again at the bump.

**Considered and dropped.**
- Derive the version from a count of merge commits touching process paths (no file). Dropped: changes what a fork compares and how it detects change; the maintainer's call, and forks without full history would count wrong.
- Keep the file in the PR and add a `.gitattributes` "max + 1" merge driver. Dropped: needs a git config step on every clone; a fork that misses it fails silently.
- Let `merge-plan` or a GitHub Action bump it automatically. Dropped: an Action needs a push trigger (forbidden), and the merge-plan planner is read-only by design.
- One bump per process PR instead of per batch. Dropped: more one-line PRs for no gain; a fork syncs the whole batch at once anyway.

**Cost to change later.** Low: AGENTS.md bullet, four skill/agent lines and `xtask/src/process_bump.rs`. Forks pick it up through the PROCESS_VERSION sync (63).
