## #1449 - The "Entry positions (pre-drilled)" group on the 2D Pocket page (slice 2)

**What was decided.** The 2D Pocket page gets the group "Entry positions
(pre-drilled)" on its `Lead in / Lead out` tab, under "Horizontal arc
entry". The group has one header switch. The switch writes
`links.approach.predrilled` into the page draft. Apply commits the draft
through `EditOperationParams`, as each other row of the page does. The
open group shows one line: where the holes come from, when a hole is used,
and what stands in. When rest machining is on in the draft, the group also
shows `REST_PREDRILLED`, the refusal that Apply gives.

**Why.** Slice 1 (#1493) shipped the flag for MCP and project files only.
A GUI user could not set it. The issue asks for the group by this name and
for a structural UI test. The flag, its command, its refusals and the
generator law exist; the page only had to draw them.

**Which slice, and why this one.** The issue comment of 2026-11-13 lists
the owed items. This PR takes the first: the UI group and its structural
test. It is the smallest item and it changes no toolpath. The PR says
`Refs #1449`, not `Fixes`: the issue stays open for the other items.

**Considered and dropped.**

- *A row inside the "Entry" group.* Dropped. The issue names a group of
  its own, and the later items (typed points, model holes, "pre-drilled
  only") need a group to go into.
- *The group on 2D Adaptive, Slot and the 3D roughing pages.* Dropped for
  this slice. `PREDRILLED_POCKET_ONLY` refuses the flag on those kinds, so
  a switch there would only lead to a refusal. The test pins that those
  pages do not show the group.
- *The switch disabled while rest machining is on.* Dropped. The page
  pattern (#1815) is a visible consequence line plus the Apply refusal; a
  disabled switch does not say why.
- *Two caption lines (a hint and a consequence).* Dropped. `hint` attaches
  to the row label before it as hover text, and this group has no row. One
  consequence line carries the text.
- *Typed points, model holes, the "the hole exists" choice, "pre-drilled
  only", a user-set clearance, the golden CL file, the setup-sheet line.*
  Not in this slice. They need new parameters and new generator law
  (safety floor), each with its own tests.

**Cost to change later.** Low. The group is one function in
`crates/ui/src/egui_app/predrilled_page.rs` with one call site. New rows
go into its body. A page for a second kind calls the same function.
