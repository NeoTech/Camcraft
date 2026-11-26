## #4042 - AGENTS.md: the Side branch rule for side/path-kernel

**What was decided.**
- The rule **Side branch** goes in "Issue Law", directly after
  **Overlapping issues stack**. Both rules are about a PR whose base is not
  `main`, so a reader finds them together. "Pull Request Law" gets only
  short pointers in **Done on a branch**, **When PRs merge** and **When a
  PR merges**.
- A sub-PR merges into the side branch when its review is clean, also when
  it is a `feat`. The feat-at-cut rule applies to the switch-over PR to
  `main`. Without this, no kernel sub-PR could merge before a cut.
- The rule names the generic form `side/<name>` and says that
  `side/path-kernel` is the one that exists. A new side branch needs the
  maintainer's word, so the generic form gives no agent a new permission.
- The rule states the GitHub protection as the maintainer approved it: no
  force-push and no deletion. It does not say that GitHub requires a PR.
  The rule itself says that each change goes through a sub-PR.
- Three small edits remove contradictions in other rules: the start
  branch (**Start from the latest main**), `--delete-branch` (**When PRs
  merge**), and `in-review` after a merge (**Done on a branch**, **When a
  PR merges**).
- The skills `work-issue` and `merge-ready` get one or two lines each:
  the base branch, `Refs #N`, no `--delete-branch` on the switch-over, no
  retarget of a sub-PR to `main`, and the issue comment after a sub-PR
  merge.

**Why.** The design record `docs/decisions/4024-path-kernel-design.md`
("Side-branch plan") and the maintainer's answer of 2026-11-25 ("Yes,
both") ask for the rule before the branch is made.

**Dropped.**
- A separate section in "Pull Request Law": the rule is about branches
  and issues as much as about merges, and **Overlapping issues stack** is
  the nearest rule.
- The full side-branch plan in `AGENTS.md` (sync branch names, the switch
  function, the proof list): it stays in the design record. The rule
  points to it.

**Cost to change later.** Low. A process PR edits the rule text. The rule
name is stable; the skills cite it.
