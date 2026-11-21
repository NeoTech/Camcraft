## #3184 - Post-merge gate: one independent run per merge

**Decided.** `post-merge.yml` uses one concurrency group for each commit,
`post-merge-${{ inputs.ref || github.sha }}`, with `cancel-in-progress:
false`. Each push runs on a free `camcraft-post-merge` runner and always
finishes. The `gate` job does not change. The `report` job has its own
group, `post-merge-report` (`cancel-in-progress: false`), and checks out the
full history (`fetch-depth: 0`). `.github/scripts/post-merge-report.sh`
keeps the last reported commit in a hidden marker in the rolling issue body,
`<!-- post-merge-last-sha: <sha> -->`. A run whose commit is a strict
ancestor of that commit (`git merge-base --is-ancestor`, not the same
commit) prints a notice and reports nothing.

**Why.** With one group for `main`, a second push waited and a third push
cancelled the waiting run. A red run then covered several merges, and the
breaking merge needed a bisect (#3111). With one run for each commit, a red
run names one commit. Runs can now finish out of order, so the report must
not let an older green run close the issue of a newer red run.

**Details chosen.**
- The marker is read from the newest rolling issue, open or closed. A green
  run with no open issue writes its commit into the closed issue's marker.
  Without that, an older red run that finishes later would open a new issue
  after a newer green run.
- The same commit (a re-run) is not older than itself, and it reports.
- If git cannot resolve the marker or this commit, the run reports as
  before (fail open to a report, never to silence).
- A new red issue carries the marker in its body. A red comment and a green
  close write the marker into the issue body (the old marker line is
  replaced).

**Dropped.**
- A repository variable or a separate state file for the last sha: the
  issue asks for no new secrets or variables, and the rolling issue is
  already the one shared state.
- Comparing commit dates instead of ancestry: dates of merge commits are not
  an order guarantee.

**Known limit.** GitHub keeps at most one pending job in a concurrency
group. If three report jobs wait at once, GitHub cancels the middle one.
That report is lost; the next run on `main` reports the newer state. With
two runners and a gate of about 20 min, three reports at once are rare.

**Cost to change later.** Low. The marker is one line in the issue body;
removing the guard returns the old behaviour.
