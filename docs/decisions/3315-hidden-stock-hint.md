## #3315 - Simulate with a hidden Stock row: a hint, not a forced stock body

**What was decided.** While the active setup's Stock row is hidden, the
line under the simulation transport says: "The stock is hidden, so the
material removal is not shown. Show the Stock row in the Browser to see
it." (`play_precalc::STOCK_HIDDEN_NOTE`). The simulation still draws no
stock body while the row is hidden. The line goes when the stock is shown,
and it never shows outside the simulation.

**Why.** The issue gave the maintainer two choices: say why, or draw the
stock body whatever its visibility. The hint keeps what the user asked for
(the stock hidden, for example to see a fixture or the tool under it), and
it removes the silent case the issue reports. It is display state only: no
command, nothing undoable, and nothing that the verdict or the post reads.

**Considered and dropped.** Draw the stock during playback even when the
row is hidden. Then the eye icon does nothing in the simulation, and a user
who hid the stock on purpose cannot hide it there. The line is also placed
under the transport and not on the viewport, because the transport already
owns the simulation lines (#2181, #2280).

**Cost to change later.** Small. To draw the body instead, skip the
`active_stock_hidden` test in the playback paths (`world.rs`,
`view_pass.rs`) and delete the line and its test.
