## #1206 - Setup pages: per-setup machine and post pick is pinned by UI tests

**What was decided.** The per-setup Machine and Post pages were reproduced
through the real UI with two setups. Every control exists and is wired to
the setup's own undoable command, and a two-setup program posts one file per
mount through each setup's own post. No product code was changed; the
behaviour is pinned by four tests in `crates/ui/src/egui_app/tests/setup_mount_pages.rs`.
The PR is `Refs #1206`, not `Fixes`, because the path the maintainer hit has
not been found.

**Why.** The issue's "To do first" asks for a reproduction before any fix,
and the reproduction found no missing or wrong step. Inventing a fix for a
path that works would change behaviour without a defect behind it.

**Considered and dropped.**
- Changing the Setup page to show the setup's machine and post without the
  active-setup guard. Dropped: the guard keeps every edit on the active
  setup, and the browser already activates a setup before opening its node.
- Adding a new model or command for per-setup mounts. Dropped: the issue
  says the model and the commands already exist.
- Escalating to P1 or effort:high. Not taken: no setup posts through another
  setup's post and no setup is generated against another setup's machine in
  the flows driven.

**Cost to change later.** Low. The tests pin the current pages; if the
failing path is found, the fix lands next to them.

**Limits.** The tests drive clicks through egui_kittest and check state and
posted output. Nothing was looked at on screen.
