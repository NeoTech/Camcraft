## #1424 - A P1/P2 filed after a cut has started does not restart the cut

**What was decided.** The "no minor/major cut while a P1 or P2 is open" rule is a start condition, checked once when the cut starts (release step 1). The session records the start time and the open P1/P2 list (none) in the release PR body. A P1 or P2 filed after that does not restart the cut and does not enter it. The cut finishes on the tree that passed the release gate; the late issue is named under "Known issues" in the release notes and in a comment on the release PR, and its fix merges after the tag and ships in the next minor. Only the maintainer can stop a cut for a late issue. An agent that finds a late P1 tells the maintainer in one line and goes on. Failures of the cut's own release gate are still fixed in the cut.

**Why.** The maintainer, in chat on 2026-11-13 after the v0.19.0 cut: "We should as a rule not let P1/P2s coming in after cut - force a restart as its a ~2 hour job". During v0.19.0, issue 1417 (P1) and issue 1398 (P2) were filed after the release branch existed. Folding them in meant a merge to main, a merge of main into the release branch and a second release gate, about two hours. The maintainer's question-dialog answer then was "Tag now as it is", with both named as known issues on the release PR.

**Considered and dropped.**
- Keep the rule as a continuous hold (restart the cut on every new P1/P2). Dropped: it is the cost the maintainer asked to stop.
- Hold the tag until late P1/P2s are fixed. Dropped: same cost, and it lets new issues keep moving the cut.
- Skip the start check and rely on the maintainer's judgement alone. Dropped: the start record in the release PR body makes the check auditable.
- Edit the `merge-ready` skill's "no release cut in progress" precondition. Not done: it tests for an open `release/*` PR, which stays true during a cut; it never repeated the P1/P2 rule.

**Cost to change later.** Low: a text edit to the Priorities bullet and the release steps of AGENTS.md. Forks pick it up through the PROCESS_VERSION sync (56).
