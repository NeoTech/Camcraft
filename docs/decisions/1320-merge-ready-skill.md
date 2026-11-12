## #1320 - merge-ready skill in the repo, deterministic planner, collaborator rule

**Decided.** (1) The maintainer's local merge-ready skill is ported to
`.claude/skills/merge-ready/SKILL.md`, calling `cargo xtask merge-plan`.
(2) Every ordering choice in the planner is a named key ending in the PR number
(`wave1_pick_key`, `order_key`, `wave2_key`, `shared_first`); the plan prints
the base sha and each head sha, and `--json` prints the same plan for tools.
The planner's JSON is written by hand (no serde in xtask: no new dependency).
(3) Maintainer, in the question dialog (2026-11-12): "Any collaborator review
counts" and "Both collaborators" may merge - this supersedes the owner-only rule
of #1207 / PROCESS_VERSION 31. Markers and `merge-ready` labels count when made
by an account with push permission; any collaborator's session runs
`/merge-ready`; cuts and tags stay with the maintainer.

**Why.** Other agents and forks must run the same procedure and get the same
order; two people merge on the same code base.

**Dropped.** Calling the Python planner (not in the repo, needs Python); a
serde dependency for JSON; making the pairwise-clash probe symmetric (the
reference tries b on top of a for a < b, kept, so plans match it).

**Cost to change.** The collaborator list is read at run time with
`gh api .../collaborators`; a different trust rule changes those four skill and
agent texts and the AGENTS.md paragraph. xtask reads no marker author, so no
code depends on the rule.
