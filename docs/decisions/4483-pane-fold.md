## #4483 - Side panes fold to title chips

**Decided.**
- The fold flag is egui_dock's own `LeafNode::collapsed`. The dock's fold
  arrow, the saved panel setups and the fold law read one fact. The arrow
  stays on each leaf: it now folds a side pane too, so it is the fold
  control (no new button).
- A pane stacked top over bottom folds natively (egui_dock gives it the
  tab-bar height). A side pane (Browser, Properties) is held at the kit's
  `CHIP_WIDTH` (28 points) by the split fraction. The open fraction is
  stored on fold and put back on unfold. The fraction is set again on each
  frame, so a window resize or a separator drag cannot widen a folded pane.
- The shell draws the kit's chip (the title turned to read top to bottom)
  in the folded leaf, under its tab bar. One click on it unfolds the pane.
- The viewport never folds: the fold law clears its flag. Its fold arrow
  stays visible and a click on it does nothing. egui_dock 0.21.1 has only
  the global switch `show_leaf_collapse_buttons`; it cannot hide the arrow
  on one leaf. To hide it we must hide all arrows and draw a fold control of
  our own, or paint over the arrow (a fragile pixel trick). We keep the
  arrows: the side panes need them, and the dead arrow on the viewport is
  the smaller cost. When egui_dock gets a per-leaf switch, hide it there.
- A drag on the separator of a folded side pane snaps back. egui_dock moves
  the separator and the pane for the frame of the drag, and the fold law
  puts the chip width back on the next frame. The drag cursor still shows.
  The pane does not unfold. We accept this: a folded pane has no width to
  drag.
- Each chip keeps the safety state as text with a glyph (the #4245 E2
  rule), until the issues pane (PR I) carries the counts. The Browser chip
  shows the counts of blocked, stale and not generated operations, or
  "Up to date". The Properties chip shows the state of the open operation:
  "Up to date", "Needs regenerate", "Not generated" or "Blocked" (the
  reason in the tooltip). The glyph is a check mark or a warning sign, so
  the state never depends on colour. The state is read from `status_of` and
  `tool_problem_of` on each frame, but only for a folded pane. The status
  bar reads the same sources on each frame already.
- The chip's hover is a 2 px accent frame and a full-ink title. The hover
  fill step alone is about 1.03:1 in the dark theme. A kit test checks the
  step with `contrast::ratio` in the dark and the light theme. The tooltip
  follows the pointer, because the chip is as high as the pane.
- The stored open fractions live in `State::pane_folds`, keyed by the
  first tab of the leaf. A closed pane forgets its fold. Reset layout and a
  setup load clear the store.
- A saved setup writes the OPEN fraction of a folded side pane into the
  split, and a top-level mark `camcraft_pane_folds: 1`. A setup without the
  mark is from an older build: it loads with every pane unfolded, because
  the old arrow set a flag that did nothing to a side pane.
- Only the main area folds side panes. A floating window keeps egui_dock's
  own behaviour.
- The chip, the chip rect and the fraction arithmetic are in
  `crates/ui-kit/src/fold.rs` (no dock, no shell state). The fold law over
  the dock is in `crates/ui/src/egui_app/layout/fold.rs`.

**Why.** One flag keeps the persistence free: the setups already carry the
dock's serde form. The mark is the cheapest way to tell an old setup from a
new one without a change to the setups file format in `camcraft-app`.

**Dropped.**
- Hide the dock's arrow and add a fold button of our own: more shell code,
  and the arrow already sits where a user looks for it.
- Restack the side panes in vertical splits so that all folds are native:
  it changes the default layout that #4245 keeps.
- An envelope object around the dock JSON: an older build would refuse a
  new setup by name instead of loading it.

**Cost to change.** Small. The chip look is one kit function. A different
fold control only calls `layout::fold::fold` and `unfold`.
