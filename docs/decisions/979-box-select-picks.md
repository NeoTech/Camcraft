## #979 - Viewport box select into an operation's picks

**Decided.**

- **Gesture: Ctrl (or Shift) + left drag**, in Edge or Face mode with an
  operation open and no other pick draft collecting (boundary, locate,
  plane, axis, three points). Ctrl and Shift are the keys that already
  mean "add" to a feature click (`queue_feature_pick`), so the box adds
  to the op's picks; it never replaces them. A plain left drag still
  orbits, exactly as before - no existing gesture changes meaning.
- **Window and crossing, by drag direction** (the long-standing CAD
  convention): a drag to the right is a window and takes what lies
  wholly inside the box; a drag to the left is a crossing box and also
  takes what it touches. The rubber band shows which: solid with a light
  veil for a window, dashed for a crossing box.
- **What it takes follows the mode**: Edge mode takes feature edge
  chains (the tangent chain, `EdgeUnit::Chain` - what an op's click pick
  stores); Face mode takes welded faces. Vertex mode does not box
  (no op takes a box of points today).
- **The #692 one pick rule holds**: only feature chains are candidates
  (never a facet seam), and a candidate counts only if some part of it is
  unoccluded from the eye, by the click pick's own sight test
  (`feature_pick::occluded`), up to 16 sight rays per candidate, spread
  along it. **The box honours the Pick through toggle** (`state.pick_through`,
  #1204) exactly as a click does (`feature_click`): off, hidden chains and
  faces stay out; on, the sight test is skipped and they are taken too.
  A click and a box never disagree about hidden features.
  "Visible" is literal: in perspective straight over a through-cutout
  the eye sees down into it, so its walls' corners and bottom rim count;
  from the Top view in orthographic they are hidden and stay out.
- **Candidates are the user's parts only** - never stock, never a
  fixture - though both still hide what lies behind them (stock is
  translucent and never hides a model, as for clicks). A box is a part
  feature gesture; stock edges inside a crossing box were noise.
- **One command**: the release queues one `SetOperationGeometry` with the
  applied picks plus every new one (duplicates dropped), so it is one
  undo step. A box holding nothing new says so in the status and queues
  nothing.
- **MCP twin: `pick_in_box`** - a world X/Y box seen from straight above
  (+Z, orthographic: the Top view), same modes and kinds, adds by default
  (`replace: true` replaces), one undo step, refuses an empty, degenerate
  or already-picked box by name. The headless surface has no camera, so
  it takes the one view a 2.5D part is boxed from. It takes an optional
  `through` (default false): without it a headless box is visible-only,
  with `through: true` it also takes the hidden features - the toggle's
  twin. The tool description says so.

**Considered and dropped.**

- A box that ignores Pick through (always visible-only): a click with the
  toggle on takes a hidden rim the box beside it would leave out - the
  two gestures would disagree (NeoTech's review of PR #1078).
- MCP `pick_in_box` always visible-only, documented: workable, but the
  headless surface would lose a capability the GUI has.

- Plain left drag as the box (orbit moved to a modifier): changes the
  viewport's oldest gesture; every user's muscle memory and the drag law
  tests would move.
- Alt + drag: Alt already narrows an Edge click to one segment.
- Window only: crossing is the cheaper way to grab one side of a rim and
  costs one drag-direction check.
- Edge visibility by every sample (fully visible only): a rim chain half
  hidden behind a boss would never box, while a click on its visible half
  takes it - the box and the click would disagree.

**Cost to change later.** Small: the gesture arm is `box_pick::armed`,
the rule is `pick::box_bodies` (mode, kinds and the take filter are
parameters).
