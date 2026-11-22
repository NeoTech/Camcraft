## #3354 - "Pass starts at" on 2D Pocket

**What was decided.**

- 2D Pocket gets the shared field `pass_start: machining2d::PassStart`
  of #3316, on `OpParams::Pocket`. It uses the same project key
  (`pass_start`, `auto` never written), the same Passes-tab row
  (`pass_start_row`), the same MCP door (`edit_operation`) and the same
  digest bytes (written only off Auto). No new type.
- The generator calls `pocket_started` (from #3316) instead of
  `pocket_entered_with_relief`. With Auto, `pocket_started` is that
  function byte for byte, so every golden stays the same.
- The ring strategies (Waterline, Offset rings) read the value through
  `plan_passes_from` and `waterline_passes`. Auto resolves to one of the
  two strategies first; a resolved ring strategy reads the value.
- Zigzag has no ring order: the rows sweep across the area and have no
  inside or outside. With Inside or Outside, the planner writes the path
  note `PASS_START_ZIGZAG_NOTE`. The choice is never dropped silently.
- Rest machining walks its own rest areas, not the pocket's rings. With
  Inside or Outside, the generator writes the path note
  `PASS_START_REST_NOTE`.
- The row is shown for every strategy. The hint and the notes say where
  it has no effect.

**Room for the #1448 start point.** Unchanged from #3316: the start and
exit points go beside `pass_start` as their own fields; `pocket_started`
takes them when #1448 lands.

**Why.** The maintainer asked for one parameter with one shape on every
kind with a pass order (#3316, 2026-11-22). 2D Pocket's ring strategies
use the same planner as 2D Offset, so the value threads through with no
new law.

**Considered and dropped.**

- An order for Zigzag rows (start at the wall or at the core). Rows have
  no inside or outside; a row order is a different parameter (a start
  side or a start point, #1448).
- Hide the row for Zigzag. The strategy can be Auto, which resolves at
  generate time, so the page cannot know. A note on the path is honest
  in all cases.
- Pass the value into the rest planner. The rest areas are many small
  regions with no family order; that is a separate design.
- Move `pass_start` into `PocketParams`. The #3316 entry functions take
  it as an argument already; 2D Adaptive can move it in its own issue.

**Cost to change later.** Low. A Zigzag row order or a rest order can
replace the notes without a change to the file key.
