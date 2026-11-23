## #3453 - The untabbed levels of a tabbed 2D Contour start at its first tab window

**Decided.** On a loop with tabbed levels and no picked start, each
untabbed level starts where the first cut (window 0) of the tabbed levels
begins (`tab_clear::level_start`). The tab layout does not change: the tabs
stay centred between the spacing marks from the raw ring start, as on
origin/main. So the tabbed level enters window 0 at the point where each
level above cut the same lead-in, one stepdown or less below their floor.

**Why.** On origin/main the untabbed levels started at their own seat (the
axis start or the lead-room reseat, or the raw start), and the first window
of the tabbed level began after the last tab. Its lead-in stood in sheet
that no level above cut: an 18 mm feed plunge with a 2.5 mm crescent of
uncut sheet beside the cutter (`crates/motion/tests/contour_tab_entry.rs`,
tables in the PR). Only the start of the untabbed levels of a tabbed loop
moves. A contour with no tabs, a ring that walks no tabbed level, and a
picked start do not change.

**Exception: a shared level keeps its seam.** A ring that shares a level
with another ring of its band, or a band whose levels another band of a
whole-model contour also cuts (`PreparedBand::room` is set), keeps the old
start. Reason: two rings that cut the same path at one level must split it
at the same seam. A moved seam split a coincident corner arc differently,
and the path crossing law blocked the post
(`contour_whole_step.rs` case 1 and case 2: `Crossing { moves: [15, 59] }`).
On such a ring the first tabbed window still enters as on origin/main;
#3678 owns it.

**Scope split (coordinator, 2026-11-23).** This PR is part (1) only.

- Part (2) is #3678: a guard that pecks any contour dive deeper than one
  stepdown below the floor under the full tool disc. The later tab windows
  each enter at their own lead-in start, which no level above cut. On the
  fixture, windows 1 to 3 still plunge 18 mm with a 2.5 mm crescent, as on
  origin/main. The test pins those readings so that #3678 changes them on
  purpose.
- With the stay-down ramp preset, the levels below the first do not cut the
  lead-in. Window 0 then plunges 13.5 mm below the disc floor. #3678 owns it.
- The verifier law (a feed plunge deeper than one stepdown into uncut stock
  passes with a centre-cutting tool) is #3679.

**Dropped.**

- Seat the tabbed levels with the untabbed start (the context pack's first
  guess): on the fixture the raw start and the reseated start were the same,
  and window 0 still began after the last tab. No effect.
- End the tabs on the spacing marks from the start (the picked-start layout,
  #1473), so that window 0 begins at the start: tried and dropped. On a
  square with 4 tabs every tab moved onto a corner and grew past its width
  (`contour_tabs.rs`, `machining2d.rs` tab tests red; two loops "fill the
  whole loop"). On the L-shape, two windows then plunged with the full disc
  in uncut stock.
- Split the wrap window at the ring start, so that the first cut starts at
  the seam: it adds one more entry after a tab, so the number of dives into
  uncut sheet does not go down, and the window counts of every tabbed level
  change.
- The peck guard in this PR: split out by the coordinator.

**Cost to change later.** The untabbed levels of a tabbed loop start at a
different point than before (often a few mm along the ring). A project file
stores no start, so a later change moves it again with no migration. The
catalog case `2d/contour-tabs` carries the new program.
