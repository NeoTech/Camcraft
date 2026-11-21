## #1863 - DXF profiles in the UI: Stock page, drop, units prompt, browser row, stock-top overlay

**What was decided.**

- The DXF door is on the Stock page ("DXF profiles" group, "Add DXF
  profile…"), as the maintainer's comment on the issue asks. `Model > Open`
  does not read `.dxf`, and `pick_mesh_file` does not change.
- A `.dxf` dropped on the viewport goes through the same `ImportDxf` path
  (`State::import_dxf`), with the units prompt. With no stock on the active
  setup the drop is refused by name and says "make a stock first".
- The DXF import does not go through `finish_import`. That function handles
  the model worker (`ImportModel`). `ImportDxf` reads and checks the drawing
  at construction on the UI thread, so the typed `DxfUnitsMissing` error is
  caught where the command is built (`state_sketch.rs`). The issue's intent
  (catch the typed error, ask, then run `ImportDxf` onto the history) is kept.
- The units prompt shows the drawing size read both ways: as millimeters
  (with the size in inches) and as inches (with the size in mm). Cancel,
  Escape and a backdrop click place nothing. No unit is guessed.
- Placement on the Stock page is a draft (X, Y, angle). "Place profile"
  queues one `MoveSketch`, so a drag of a field does not write one undo step
  per frame. Delete queues one `DeleteEntity`.
- Hide / Show is view state in `hidden_models`, the same list as the model
  eye. It is not a command (AGENTS.md "Everything Is a Command": visibility
  is ephemeral view state).
- The browser shows each profile as a child row of its setup's Stock row
  (`Leaf::Sketch`), with the eye. A click opens the Stock page and selects
  the profile, so Del deletes it through the history.
- The viewport draws the profiles of the active setup on the stock top at
  `sketch::world_pose`. The free ends of open chains get a magenta X, a
  colour that no curve uses.
- The operation page's Source list offers the setup's profiles for a kind
  that `sketch_model::admits` (2D milling, plasma, wire EDM). The app's
  profile law still refuses a wrong pairing by name.
- `camcraft_app` re-exports `camcraft_geometry::dxf`, so the UI can name the
  `Primitive` variants of a sketch. This adds no crate dependency.

**Dropped.**

- A dropped `.dxf` in the browser build (wasm) is refused by name for now.
  The async drop path delivers model bytes only.
- No viewport drag of a profile. The Stock page fields and `MoveSketch` are
  the one placement door.
- No "move/setup alignment" verb on the browser row beyond opening the Stock
  page: a profile follows the stock, and the setup's part alignment does
  not move it (#1862 law).

**Cost to change later.** Small. The UI calls only the #1862 commands.
