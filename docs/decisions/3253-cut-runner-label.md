## #3253: the release gate gets its own runner label

- **Chosen:** `cut.yml` runs on `[self-hosted, camcraft-cut]`, and only the cut runner carries that label (today `wopr`).
- **Why:** `[self-hosted, linux]` matched every self-hosted runner. Since 2026-11-21 there are eight post-merge runners, several of them 8-thread or 24 GB machines. A 5-hour release gate could land on any of them. Labels are matched case-insensitively, so `Linux` on a post-merge runner satisfied `linux`.
- **Considered and dropped:**
  - Removing `self-hosted, Linux, X64` from the post-merge runners: it needs a re-registration of each runner, and any future workflow that asks for `linux` hits the same problem.
  - Stopping the other runners during a cut: kept only as a stop-gap for the cut of 2026-11-21. It depends on a script on one collaborator's machine.
- **Merge order:** the label goes on a runner first, then this merges. A `cut.yml` that asks for a label no runner has queues forever.
