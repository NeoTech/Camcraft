## #1241 - a machine switch stays a live-gate input, not a stamp input

**What was decided.** Changing a setup's own machine (`SetSetupMachine`) or
the document's (`LoadMachine`) does not mark that setup's generated
operations Stale. The trace (`crates/app/tests/machine_switch.rs`) shows
every machine-dependent law is judged again at the post against the
machine the op's setup mounts NOW: the stroke and the spindle window
(`machine::check_ops` through `posting_mounts::machine_problems`), a
leaning path on a machine with no tilting rotary (`judge::pose_problems`),
and an op family the new machine's kind cannot run
(`posting_family::family_at_post`). Each refuses the post by name, writes
nothing, and the same holds after undo/redo and after save/reopen. So no
path cleared on machine A can post on machine B unjudged; the only gap the
issue saw was the display, and that is by design. No product code changed;
the issue's "Done means" is met by its "or the issue records why it need
not" branch.

**Why.** The op stamp deliberately leaves the machine out: the M2.3
roomy-one law ("load the roomy machine and the same program posts without
re-generating"), extended to workholding at M3.2 B (RECOMMENDED.md, the
"workholding never rides the stamp" entry), and pinned by
`fixtures.rs::workholding_never_stales_a_fresh_toolpath` ("loading a
machine must not re-generate the program"). Machine limits are evidence
computed at the post, never a cached verdict - so the post can never
trust a stale verdict, and a user who mounts the right machine is not
forced to regenerate a program that already fits it. Staling on a switch
would add a second, weaker net (a stale op is refused for freshness, not
for the law it breaks) and would break a pinned law.

**Considered and dropped.**
- Stamping the machine identity (or its judged fields) into the op
  stamp, as the dispatch suggested for the "stale display only" outcome.
  Dropped: it contradicts the recorded roomy-one law and its pinning test,
  and M3.2 B already tried a machine/steel digest term and reverted it
  when the UI's own tests caught the contradiction. That is a design
  reversal for the maintainer to make, not an unattended worker.
- Stamping only the machine fields generation reads (kind, rotary
  inventory, spindle body, Z ceiling). Dropped for the same reason: in
  practice every real machine swap moves the Z stroke, so it would stale
  on every switch, and the post re-judges each of those reads already.

**Cost to change.** If the maintainer wants a switch to show the ops as
needing regeneration, it is one digest term in `operation::current_stamp`
(the setup's effective machine) plus flipping
`workholding_never_stales_a_fresh_toolpath` and the `Ok` asserts in
`machine_switch.rs`; documents saved with a machine read Stale once on
open (the usual stamp-bump churn). A softer alternative that keeps the
law: a non-blocking "generated on <machine A>" note on the op.
