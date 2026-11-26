## #4178 - Path kernel K5: the entry and exit stage

Stage 6 of the K0 record (`## #4024` in `RECOMMENDED.md`, "Stage 6: Entry
and exit"). The stage is in `crates/pathkernel/src/entry/` (the record's
folder: `air`, `hole`, `column`, `material`, `exit`, with `proof`, `room`
and `stage`). The planner is `EntryStage`. No old planner calls it, so
the output with the default planner switch does not change.

**What was decided**

1. **The entry returns a verdict, so a fallback has a name** (law 4). The
   trait is now
   `fn entry(&self, first: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal>`.
   This is the smallest change that the skeleton allows: the guide and
   the load stages already return `Result<Verdict<_>, Refusal>`, and
   `Verdict::Done` holds the fallbacks and the notes. `exit` keeps
   `Result<Exit, Refusal>`: an exit has no user choice to fall back from.
   `named(entry, choice, why, notes)` builds the verdict, and
   `needs_fallback(asked, used)` is the one rule: another kind than asked
   is a fallback, except that an air entry and an arc lead are one family,
   and an entry in air, in a hole or in a column where an entry into
   material was asked is not a fallback (the asked kind applies only where
   the tool must enter material: the record's order of tries; the same
   reading as the L4 check of #4027). A planner returns `Err` when no
   entry fits, never `Verdict::Refused` or `Verdict::Incomplete`.
   Dropped: a new `EntryOutcome` type (a second result type for one
   stage); a `fallback` field on `Entry` (an entry would then carry a
   record of the stage, not of its moves).
2. **The order of tries.** The asked kind first when it is an entry
   outside material (so that its fallback reason is true), then air, a
   pre-drilled hole, a cleared column. An entry into material only when
   the user asked for a helix, a ramp or a plunge: the asked one, then the
   other one of helix and ramp, with a named fallback. A plunge is never a
   fallback: it is tried only when asked, and `Entry::led` refuses it
   without the user's choice and a tool that can plunge. When nothing fits
   the stage refuses by name: `NO_SAFE_ENTRY` (no entry into material
   chosen; fix: pre-drill a hole or select a helix or ramp) or
   `NO_MATERIAL_ENTRY` (the chosen one does not fit). A cut that starts
   above the material top is entered straight down in air.
3. **One gate: `Entry::led`.** `Entry::new` keeps its signature and calls
   `led` with no lead. An entry has a new optional `Lead`: the moves in
   the plane at the cut depth from the end of the descent onto the cut
   (the #3746 arc, the move from a hole or a column). `Lead::new(pts,
   room)` checks two or more points at one depth and no stretch outside
   the room that it names; `led` checks that it starts where the descent
   ends. The descent check of the skeleton is unchanged.
4. **The legal room of the tool centre** at the cut depth is read from
   the region of K1: the centre, the air band, and the band of one tool
   radius outside each stock edge loop (there the disc overhangs the stock
   but touches no part: a loop all on the stock outline has no part wall),
   less the keep-out grown by the tool radius, grown by `ROOM_TOL`
   (0.005 mm, the arc tolerance of the polygon offset). The region has
   no stock outline of its own; a stock edge loop is its best reading.
   Where a part touches the stock edge, that loop is a part wall and the
   band is not added there: the lead then must stay in the centre or the
   air (safe, it can refuse more).
5. **The air entry (#3746).** The descent is in the air zone: the air of
   the region less the places within the approach distance of the centre
   and the stock band. K1's air band is one tool radius wide (stock + r
   to stock + 2r), so the approach distance is limited to half the tool
   radius, with a status line note when it is limited. Then the arc of
   `ArcChoice` (radius, sweep up to 180 degrees) tangent to the cut at its
   start, on either side, at the asked radius and then halving (5 tries,
   as the old code); a straight lead in air back along the arc's tangent
   where the arc start is not in the zone. The straight lead must not
   enter the centre area: so an Inside start deep in the band is not
   entered through uncut material. A smaller radius gives a note. The
   arc is kept as chord points within the chord tolerance; the exact
   circle (G2/G3) is not recorded yet.
6. **The hole entry.** The nearest hole whose centre is in the legal
   room and whose straight lead to the cut start stays in it. A hole
   narrower than the tool needs a tool that can plunge (the tool cuts the
   ring between the hole and its radius). Holes carry no depth: the caller
   gives only holes that reach the cut depth.
7. **The cleared column.** `cleared` (the trait's argument) is the area
   where no stock stands down to the cut depth; the caller fills it from
   its stock reader (`StockRead`). The column is `cleared` less the tool
   radius (the whole disc is in cleared material), in the legal room. The
   cut start serves when it is in the column; else the nearest corner of
   the column with a straight lead in the legal room.
8. **Helix and ramp limits are the user's** (`MaterialChoice` on
   `EntryChoice`, set with `with_material`): one descent angle (above 0
   and below 90 degrees; default 3, the old `DEFAULT_RAMP_ANGLE`) for the
   helix and the ramp, and the helix radius (default half the tool
   radius; at most the tool radius, so the core is cut). The helix passes
   through the cut start, tangent to the cut, and its circle stays in the
   legal room; 4 radii (halving). The ramp runs along the whole first cut
   (laps of a closed cut, forth and back on an open one) at no more than
   the angle; at most 128 passes and 1024 helix turns (the old caps).
   Dropped: a separate helix pitch (the angle gives the pitch,
   2 pi R tan a).
9. **The exit.** `Exit::proved(last, lead_out, retract, top)`: the
   lead-out starts at the end of the last cut at its depth, and the
   retract is at or above the material top (no rapid in material). The
   lead-out is an arc (the lead-in reversed) in the room where the tool
   cuts nothing: its disc stays in the cleared area, the band the last
   cut swept, or the air round the stock. Where none fits, the tool goes
   straight up. `Exit::new` keeps its signature and builds only an exit
   with no lead-out; `Some(points)` is refused (`UNPROVED_LEAD_OUT`), so
   no unproved lead-out can exist.
10. **The heights are the planner's**, not the choice's: `Heights { top,
    clear, retract }` along the plane normal, in the depth units of the
    plan. The descent starts at the top plus the clearance.
11. **Geometry in `camcraft-geometry`**: `geo2d::arc_points` (an arc as
    chords within a tolerance, at most 4096 chords) and
    `Region::holds_path` (no stretch of a polyline outside the region).
    The kernel keeps no private copy.

**For the law suite (#4027, PR #4175)**: when the two meet, the second to
merge changes these: each stand-in `impl EntryPlanner` returns
`Ok(Verdict::done(entry))` (or `named(...)`); `kit::run` takes the entry
out of the verdict (`live(...)`) and keeps the verdict for L4;
`check_l4` counts an entry as silent when `needs_fallback(asked, used)`
is true and the entry verdict has no `Fallback` of `Stage::Entry`
(today it treats `ArcLead` as an entry into material and cannot read a
fallback).

**Why**: the record fixes the order of tries and the laws, but not how a
fallback is carried, how the room is read from K1's region, or where the
user's limits live. Each choice keeps every value valid by construction
(#4055 decision 12) and keeps the old planners as they are.

**Cost to change later**: low. The stage is internal to the kernel; no
project file and no posted program changes until a kind PR calls it.
