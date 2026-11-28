## #4453 - Recipes tab: surface speed and chip load as inputs

**What was decided.** Each row of the Recipes tab gets two inputs under the
table: the surface speed (Vc) and the chip load (fz). They replace the
read-only derived line, which is now the hover text of the two inputs. The
table keeps its Spindle rpm and Feed columns. A Vc edit sets the row's
stored rpm (n = Vc x 1000 / (pi x D)). An fz edit sets the row's stored
feed (F = fz x z x n) and keeps the rpm. The stored numbers do not change
type, so the project file does not change. The edits write the window's
draft. The editor's one Apply lands them as one `ChangeTool`, as for each
other number on the tab.

**Refusals.** The fz input is disabled when the tool has no flute count or
the row has no rpm. The Vc input is disabled when the tool diameter is not
above zero. The disabled input names the reason in its hover text. The
conversion functions (`recipes_speeds::rpm_for`, `feed_for`) also return the
reason as an error, so no path divides by zero.

**Why.** Both values are visible at the same time, so no mode switch is
necessary. The issue offered "a switch in the tab header" or "both
fields". Both fields are simpler and need no extra state. The inputs stand
outside the shared table (`tool_cutting_rows`), which Properties and the
non-rotating tools also draw. That keeps the change in the Recipes tab.

**Dropped.**
- A header switch between fz and F: one more state, and the user sees only
  one of the two values.
- Two new columns in the shared table: that table also serves the torch,
  waterjet and lathe rows, and other PRs touch it.
- Rounding a derived rpm to a whole number: the rpm stays exact, so the Vc
  that the user typed reads back unchanged. The post rounds the S word.

**Frame cost.** The derived lines stay cached (`RecipesView`). The two input
values are two multiplications for each row on each frame. They are not
derived geometry.

**Cost to change later.** Low: one UI file (`recipes_speeds.rs`) and its
tests.
