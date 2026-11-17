## #2174 - Turn a fixture by quarter turns or typed angles

**What was decided**

- A new undoable command `TurnFixture` turns one fixture about world X, Y
  or Z through the fixture's own origin. The Fixtures page has six buttons
  (Turn X+ / X- / Y+ / Y- / Z+ / Z-, 90 degrees each). The MCP tool
  `turn_fixture` takes any angle. The write is a `MoveFixture`, so plate
  riders turn with the fixture, as for a move.
- The place draft has the three angles of the Model page (roll, pitch,
  yaw) in place of the one "Place yaw" field. Apply writes all three.
- The turn premultiplies (`turn * old`): a turn about world Z stays about
  world Z when the fixture is tilted. This is the law of the Model page.
- No rotation noise (#2177). A quarter turn writes the exact quaternion
  components (`settle_rot`). A draft that keeps the angles the fixture has
  writes the old orientation. A turn that leaves the orientation as it is
  (0, 360 degrees) is refused with "nothing to do".
- Only a non-finite angle or an axis that is not X, Y or Z is refused.
  The collision box is the box of the transformed mesh, so each finite
  angle is modelled.
- A part or the stock that the fixture held does not turn with it. The
  project stores no mate relation, so the command cannot know what a
  "held" body is for certain. The command names each body whose box
  touched the fixture box before the turn (`fixture_clash::held_by`,
  0.1 mm), and the status line says to locate them again.
- The turn does not mark toolpaths stale. Fixtures stay live gate inputs
  (the live-gate law in `fixture.rs`): the post gate checks each toolpath
  against the fixture as it stands. A turn into the tool path blocks the
  post with "strikes a fixture" with no regenerate.
- The MCP tool is in its own file `mcp/fixture_turn_tools.rs`, so it does
  not overlap the open PR #2184 (`mcp/fixture_tools.rs`).
- The UI assertions are in `egui_app/tests/fixture_rows.rs`, the test file
  of the Fixtures page, not in `structural.rs` (592 lines, over the 400-line
  law).

**Why**

The maintainer asked for the Fixtures page to rotate a fixture "like the
models page can move a model using quarter turns or numerical input".
Re-use of `MoveFixture` keeps one write path for riders, the plate-mount
marker and undo.

**Considered and dropped**

- Carry held parts and stock with the fixture. Dropped: a box touch is a
  guess, and a wrong carry moves a part the user did not select. A named
  warning is visible and safe.
- Mark the toolpaths stale on a turn. Dropped: it breaks the live-gate law
  that each fixture action obeys, and the gate already refuses the post.
- Pivot about the box centre. Dropped: the Model page turns about the
  origin of the body, and a pivot at the origin keeps four quarter turns
  exact.

**Cost to change later**

A carry of held bodies needs a stored mate relation (a schema change). A
different pivot changes only `TurnFixture::new`.
