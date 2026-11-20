## #2561 - a contour tab is checked against every cut of its level

**Decided.** After `tab_bridge::fit_tabs` fits each ring, the new
`tab_clear` reads each tab's line on the ring against every other cut of
the tabbed levels: the other keeps of the same ring, the far part of the
two keeps beside the tab, and every other ring.

- Another cut nearer than the tool diameter `D` cuts into the tab. The
  loop is held: its tabbed levels cut nothing, and a `tabs:` note names
  the tab and the distance. This is the response that `fit_tabs` already
  gives when no bridge fits.
- Another cut between `D` and `W + D` away leaves the tab at its width,
  with less than the tab width of material beside it. The loop is cut as
  before, and a `tabs:` note names the tab and the width of the strip.
- A picked start that a grown tab covers gets a `tabs:` note that names
  the pick and the point where the tabbed levels start instead.
- `SLACK` stays `1e-6` with no margin for the chord error. A test reads
  the bridge on the posted arcs in chords of `1e-5` mm and pins it to
  `1e-4` mm.

**Why.** The shape-mask measurement on main found tabs with a neck of
0 mm and no note: a tab on the wall of a notch 10 mm wide (6 mm cutter),
and a tab that faces a second part 10 mm away. The return run, or the
other ring, passes 4 mm from the tab and removes it from its stock. A
part can then come free with fewer tabs than the user asked for (Prime
Directive: correct, or refused by name).

The strip case is a note and not a hold. The strip between two kerfs has
the full cut depth, and the tab has only the tab height, so the strip is
not the weak point. The same reading also occurs across a thin wall of
the part itself, which is not a tab defect. A hold there would refuse
common nests.

**Dropped.**

- Move the tab away from the other cut. This changes where the user's
  tabs are, and needs a placement law. Not in this fix.
- Hold only the loops that have no good tab left. The user asked for a
  count; fewer tabs without a refusal is the silent case again.
- Remove a held ring's cuts before the other rings are checked. The
  check uses all cuts as first fitted. This is conservative: a hold only
  removes cuts.
- A margin on `SLACK`. A margin of some microns grows each tab window
  on a straight wall and moves the recorded router example program. The
  measured error on the posted arcs is under `1e-6` mm on the fixtures of
  the test. The flatten step (0.05 mm, 8 samples or more for each arc)
  limits the sagitta to about 0.005 mm in the worst case; the tests
  already accept 0.02 mm.
- The meter in `shape_mask.rs`. That file has 409 lines, so the meter is
  `crates/motion/tests/common/tab_neck.rs`.

**Limits.** The check is local to the tab's line on the ring. It does
not follow the stock strip farther along (a strip that narrows away from
the tab). The two keeps beside a tab count only from where they have
left the tab (`W + D` of walk, then the first point `W + D` from the
line); `tab_bridge` owns the part before that. Tabs of two rings that
face each other at the same place join the two parts and not the stock:
this reads as "cuts into" only when the other ring cuts there.

**Cost to change.** `tab_clear::fit_band` is one function with two
thresholds. A later placement law (move the tab) replaces the hold arm.

## #2561 - review fixes of PR #2705 (takeover by the owner account)

**Decided.**

- `tab_clear::departed` drops only the monotonic exit of a keep beside
  the tab: the pieces whose distance from the tab line keeps growing. A
  piece that comes nearer to the line than the farthest point before it
  starts the counted part of the cut, also inside `reach` and before the
  cut is `W + D` away. A U-turn in a shallow notch beside the tab is then
  read and holds the loop.
- With one keep on the ring, its lead-in and lead-out join the tab at
  both ends. `nearest_other` now reads them through `departed` too, so a
  lead that turns back toward the tab counts.
- `tab_clear::fit_loops` takes the tabbed levels of each loop from the
  depth ladder (`Ladder::ring`). A loop with no tabbed level is not
  checked, and a ring is read against the tabs of ring `i` only when the
  two rings share a tabbed level. With no tabbed level in the band, no
  fit is made.

**Why.** The review of head `edb566d33` found a neck of 0 mm with no note
(an 18 mm deep notch), and a false hold of a deep island beside an island
that stops on a shelf above the tabs. The first is a silent thin bridge.
The second refuses a legal program.

**Dropped.**

- Check the pieces past `reach` against `D - SLACK` only. That reads the
  turn-back but not a strip narrower than the tab width, so the note of
  the strip case would depend on the notch depth.
- Check the adjacent leads for each count of keeps. With two or more
  keeps, `tab_bridge` measures those leads; the brief asked for the
  one-keep case only.

**Limits.** The distance is to the tab line as a segment. Near the ends
of the line this is conservative: a turn-back past the end of the tab can
read as nearer than it cuts. The result is a hold or a note, never a
silent bridge.

## #2561 - the two cuts beside a tab against the tab core (`tab_kerf`)

**Decided.**

- New module `crates/motion/src/machining2d/tab_kerf.rs`. `nearest_other`
  reads the two cuts beside a tab also against the core of the tab line:
  the line without its first and last `D / 2`. The two end kerfs (the
  tool disc at each tab end) remove those parts.
- A point of a cut beside the tab is skipped only while the cut stays in
  the disc of radius `D` about the tab end and goes away from that end.
  In that part, the cut is read against the core outside that disc. From
  the first point where the cut leaves the disc, or turns back toward the
  end by more than `TURN = 1e-3` mm, all of the cut is read against the
  full core.
- A distance under `D - SLACK` holds the loop with a `tabs:` note. A NaN
  distance or a NaN limit holds the loop. The new read adds a hold only,
  never a note. `departed` stays for the "narrow strip" note.

**Why.** The review of head `57756c69a` (comment 7007645147) found a
return run along the tab at a constant distance. `departed` dropped that
run because it does not come nearer to the tab line. The meter read a
neck of 0 mm with no note: a silent thin bridge.

**Dropped.**

- Read the full cut against the full tab line. Each cut touches the tab
  end where it starts, so each tab would hold.
- Skip a fixed length of each cut (`reach`). A short notch puts the
  return run inside that length, and the run is not read.
- Make `departed` stricter only. It measures "comes nearer"; a run at a
  constant distance never does.

**Limits.** The tool is a disc and the cut is its centre line. The read
can hold a loop where the true neck still keeps the tab width (an acute
V-notch, a tab end near a tight fold). Such a hold is safe: the tabbed
levels cut nothing, and a note says so. A bore or a slot narrower than
about two tool diameters keeps its tab-height ring of material uncut,
with a note.
