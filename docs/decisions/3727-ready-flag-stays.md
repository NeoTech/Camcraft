## #3727 - a review finding keeps the PR ready

**Decided.**

- A `needs-rework` finding does not turn a ready PR back into a draft.
  The author pushes the fix to the same PR. A PR comment says which finding
  each commit closes and that the new head waits for a fresh review.
- A push moves the head. The earlier review and the `merge-ready` label do
  not count for the new head. The reviewer removes `merge-ready` if it is
  still on the PR.
- An orange `conflicts` verdict still makes one of our own PRs a draft. A
  merge-time refresh with hand edits is pushed to the ready PR and is
  reviewed as usual.
- The first `needs-rework` comment names the class of each finding and asks
  for one rule that closes the whole class (an allow-list, a guard on the
  whole input). A fix of the one case brings the next round.
- `AGENTS.md` ("Every PR opens as a draft", "Review labels"), the
  `review-prs` skill and the `resolve-conflicts` skill carry the change.

**Owner's decision** (live message, 2026-11-23): "Do your changes as a docs
changes, push it and merge it and announce it one go.. On my call." It
followed this question: "Analyze why ready to review gets switched back to
not ready." The independent review of this PR is waived by that order.

**Why.** The data covers 30 PRs from 2026-11-20 to 2026-11-23, with 36
times that a ready PR went back to draft:

- 27 of the 36 followed a `needs-rework` label. 17 came from our own
  reviews and 10 from the author's reviewer.
- 3 followed a `conflicts` label. 6 had no label cause (a ready flag set
  too early).
- The median time from ready to draft is 1.6 hours.
- #3283 and #3284 flipped three times each. #3061 returned four times for
  one class of defect (silent geometry changes).

Each round trip costs one more read and one more trial. The draft state
gave no protection: GitHub does not block a push to a ready PR, and the
head-pinned marker already makes an old review void for a new head.

**Dropped.**

- Reviewing draft PRs, with a new label for "please review". It reverses
  the maintainer's rule of 2026-11-08 (only ready PRs are reviewed). The
  ready flag stays the review request.
- Flagging ready only just before the merge for every PR. It does not fit
  contributors, who flag their own PRs.
- Keeping the draft round trip and asking for fewer findings. The findings
  come from independent reads and are real. The round trips came from the
  draft step, not from the findings.

**Cost of a later change.** Only text in `AGENTS.md` and two skills. No code
and no label changes.
