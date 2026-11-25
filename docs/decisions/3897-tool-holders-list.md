## #3897 - The Holders page of the tool library window, and the mock-up gaps

**What was decided.**
- The window gets a page switch at its head: "Cutting tools | Holders".
  The mock-up says "Tools | Holders". The label "Tools" is already the
  ribbon cell that opens the window, and two buttons of one name break
  each test and each MCP-driven flow that clicks "Tools". "Tool list" is
  already a browser tree row. So the tools page is "Cutting tools".
- The Holders page lists `holder_library::library()` grouped by taper,
  with the taper of the mounted machine (`tooling.spindle_mount`) first
  and marked. The straight-shank chucks are the group "Straight shank".
- The holder view is read-only. A built-in holder is fixed, and user
  holders and holder edits need the user library store of #3645 (U7,
  #3898). The view says so in one line.
- The used-by query is in app: `tool_body::holder_users` and
  `tool_body::tools_using_holder`, keyed on `HolderRef::entry` (case and
  spacing ignored, as `holder_library::find`). It is a read of the world,
  so it belongs in app (plan decision of #3663). The holder key moves to
  the `library_ref` of #3645 in U7.
- The five mock-up gaps of the scope comments on #3897 are in this PR (the
  user's decision: fewer PRs). Each is display only, with no command:
  1. the editor's rows on the left, the preview on the right;
  2. the preview's Dimensions and Holder toggles (view state in
     `EditorFace`, both on by default). The Holder toggle shows the model
     without the holder in 3D, and the side view without the holder bands
     and the holder dimensions;
  3. a small drawing in each Document row (cached by tool name and keyed
     on the tool value) and a "Gauge length" column (the holder's gauge
     length plus the stick-out, for a tool in a library holder; "-" for a
     tool with a drawn body, a mesh body or the stand-in cylinder, because
     their gauge line is not stated);
  4. the categories on the left of the list, from the tool kind (U2 did
     not add them). A category with no tool in the shown source is not
     listed, so a mill machine shows no wire category;
  5. the window's default and minimum width: the rail, the rows column
     (780 px, wide enough for the Recipes table) and the preview (320 px).
     The window stands left of the Properties pane. On a screen that is
     too narrow for that width, the window takes the space left of
     Properties (it then covers the browser), and the Recipes table
     scrolls across as before. Review fix (#3943): on such a window the
     rail folds from 170 px to 120 px, the preview keeps 250 px while the
     rows keep 280 px, and the side view's dimension room gives way
     before the drawing: a thin 8 px border with Dimensions off, and a
     room that shrinks in proportion so the drawing keeps 96 px. The
     painter clips a dimension value at the area's edge.

**Why.** The mock-up of #3663 (pictures 1, 2, 4, 5) and the scope comments
on #3897.

**Dropped.**
- Stacking the preview under the rows on a narrow window (the review's
  first option): the editor's column height is already bounded by the
  screen, so two stacked panes would each get half of it. A folded rail
  and a shrinking dimension room keep the two columns side by side.
- "New holder", "Import" and "Export" on the Holders page, the holder
  fields as edit boxes, Duplicate and Apply on the holder view: these need
  the store of #3645 (U7).
- A cache for the used-by count: the count is a read of the document's
  holder references (no geometry), and the frame-cost rule caches derived
  geometry. The holder outline and the row drawings are cached.
- A separate PR for the five layout items: the user folded them into U6.
  The one-page-per-PR rule of a UI refit is kept by commit: the mock-up
  items are the first commit, the Holders page is the second.

**Cost to change later.** Display only. Nothing reaches the project file,
the posted output or a collision check.
