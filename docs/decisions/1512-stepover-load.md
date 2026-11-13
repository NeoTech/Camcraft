## #1512 - one radial-step field per pocket strategy

**Decided.** The maintainer saw Stepover go from 3 mm to 25 mm with no change
to the path. The path was right: Adaptive and Offset rings step by Optimal load
(`load_step(D, optimal_load)`) and never read Stepover, while Zigzag,
Waterline and Auto space their rows by Stepover. The page showed both fields
regardless. Now the Passes tab shows only the field the strategy reads. That
holds for Pocket, 3D Pocket, Rotary offset, Multi-axis rough and the 2D
Adaptive kind; 2D Offset keeps Stepover. One predicate,
`PocketStrategy::steps_by_load()`, drives both the generator and the page.
Every Optimal load field carries a hint saying it sets the step and that
Stepover is not used. The Leads tab's Engagement group gets a hint that it
scales the entry feeds only, and the trochoid's Forward step a hint that it
is the most a loop advances: the loops shorten where Max engagement would be
exceeded.

**Why.** An inert field that looks live gets edited, read as "ignored" and
reported as a bug. Hiding it states the law where the user looks. The 3D
diameter check refused a stepover wider than the tool even under Adaptive,
where the value is never read, so a stale value blocked a legal job. That
check now runs only for the strategies that read Stepover. A positive stepover
is still required for all strategies, so switching back to Zigzag can never
find a zero. The refusal past reach now names the field and both values.

**Considered and dropped.**
- Make Adaptive read Stepover (stepover as a cap on the load step). Two
  numbers would fight over one distance, which is the confusion the issue
  reported. It would also change every Adaptive path and golden.
- Rename the Leads group to "Entry engagement". The name is pinned by the
  structural tests of every pocket kind. The hint says the same thing without
  that churn; the rename stays open if the maintainer wants it.
- Grey Stepover out instead of hiding it. An `egui` disabled row still reads
  as a field that should do something. The other guard-rendered rows on the
  page (Retract Z, Helix radius) are hidden, not greyed.

**Cost to change.** The page guards are a few lines each in `mill2d_page.rs`,
`mill3d_page.rs`, `rotary_page.rs` and `kind_editors.rs`. The validate gate is
in `machining3d/params.rs`. Nothing in the project file or the posted output
changed.
