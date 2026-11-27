## #3913 - Contributor takeover and reproduction procedures

**Decided:** Add original `takeover` and `repro-first` skills and two routing
rows in `AGENTS.md`. Reuse the existing claim, ownership, fix, gate and
conflict procedures rather than defining another ownership mechanism.

**Why:** A stalled PR needs an explicit handoff, not an age-based push.
An open PR keeps a contributor claim live. A bug fix needs an actual before
reading on current `main`, not a successful connection or an assumed failure.
The procedures record refused pushes and unsupported reproduction honestly.

**Considered and dropped:** Do not copy the SolidCraft skills. Their ideas
are prior art; CamCraft's current rules supply the procedure. Do not add a
new timeout, a replacement-PR automation, a runtime framework or CAM changes.
Do not change `PROCESS_VERSION`; the merging session owns the bump.

**Cost to change later:** Edit these two skills and their routing rows.
Keep references consistent with `issue-claims`, its ownership guard,
`work-issue`, `resolve-conflicts` and `fork-sync` when those rules change.
