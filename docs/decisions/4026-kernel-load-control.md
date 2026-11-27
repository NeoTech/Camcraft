## #4026 - Path kernel K3: the load control stage

Stage 3 (load control) of the K0 record (`## #4024` in `RECOMMENDED.md`).
Sub-PR of `side/path-kernel`, on top of K4 (#4177). No old planner changes
and no planner calls the new code yet: with the default switch the output
is identical.

**What was decided**

1. **Files.** `crates/pathkernel/src/load/`: `law.rs` (the engagement
   law), `stage.rs` (`LoadStage`, the planner), `rings.rs` (constant
   engagement), `trochoid.rs` (trochoidal loops). `load.rs` keeps the
   types. The kernel still depends on `camcraft-geometry` only; no
   geometry function was added.
2. **One law for every mode.** The stage makes a raster of the stock
   reader (`StockRead`, 20 cells across the tool radius, at most 4
   million cells; a cell that the reader does not know is material, so a
   read can only be too high). It walks the curves in the cut order, 8
   reads across the tool radius, and reads each place before it clears
   the tool disc there. The read is the contact angle phi on the front
   half of the cutting edge, as ae/D = (1 - cos phi) / 2 (Altintas 2000;
   Stori & Wright 2000). The read comes after the tool disc at the place
   is cleared, three quarters of a cell outside the edge, as the #4154
   meter reads. The first form read half a cell inside the edge before
   the clear: the disc of the read before had then cleared the side of
   the arc, and an outward ring read 54 degrees where the geometry gives
   64 (the meter of #4154 found 2297 unnamed moves on the disc). The
   stage does not see the entry (stage 5 comes after it), so the first
   cut is read without the bore of its entry: it can be named where the
   entry clears its start. A read can be too high, not too low. Every curve is read, the clean-up rings too: the
   law of #3849 (PR #4008 checks the clean-up passes as the lanes) holds
   by construction, because no curve is skipped.
3. **A stated limit for a choice without one.** `LoadControl::None` and
   `Trochoidal` carry no engagement. `LoadStage::new(limit)` states the
   limit that the stage names cuts against; the default is 0.5 (a 90
   degree contact, the old default trochoid limit). A kind sets it to its
   planned load (for rings: stepover / D). Dropped: a limit field on
   `LoadControl::None` (it changes a public type that the other stages
   and the kit build).
4. **None is the identity.** The set is the input set (marks included);
   each read above `limit x (1 + PLAN_TOL)` gets an over-limit record with
   the reason `NO_CONTROL`, and the verdict has one status line note for
   each reason (the count, the worst load and its place). `Controlled::new`
   stays the single gate: an unnamed read above the limit is refused.
5. **Constant engagement on rings inserts passes.** Where a ring reads
   above the load, the stage finds the nearest curve cut before it at
   the hot place. When both are rings of one distance field (the walls of
   the centre area, or the part walls of a region with a stock edge: the
   distance of each point agrees within 1 % of the tool radius), it
   inserts the mid level of that field between them, only the pieces
   within the gap of the hot ring, turned to run as that ring runs. It
   reads the plan again, at most 12 rounds and at most 8 times the curves
   (and 64). An insertion is kept only when it lowers the load of its ring
   (no hot read left, or the worst excess halved, or a quarter of the hot
   reads gone); else it is taken out and the ring is named `NOT_FIXED`.
   That is the sharp corner of offset rings (the #3933 rectangle read
   93.9 %): more passes do not help there, the corner is named, not
   hidden, and the note names Adaptive. A ring with no earlier curve
   within one diameter is `FRESH_CUT` (the first ring of a pocket), a
   curve that is no ring is `NOT_A_RING`, a gap below 2 x 15 % of the load
   width is `AT_FLOOR` (the floor of the held #3933 design). A
   `FullWidth` mark with no hot read left is resolved and removed; a mark
   that stays is named by its reads.
   Dropped: the outward spiral and the morph of #3933 (Held & Spielberger
   2009). A spiral is a link and an entry matter (record: load control
   adds no link). Dropped too: a new spacing of the whole ladder by the
   Stori & Wright law (fewer passes than a mid-level insertion, which
   halves a gap). The insertion reads the true stock at each step, so it
   is correct for any ring shape; the cost is more passes than an
   optimal ladder.
6. **The stock reach stays with the HSM planner.**
   `ConstantEngagement { reach: Stock }` (Adaptive) is refused by this
   stage with `STOCK_REACH_ELSEWHERE`: the K0 record revision 2 keeps the
   HSM planner in `camcraft-motion` behind the load trait. So Offset
   rings (Rings) and Adaptive (Stock) stay two behaviours, as the
   maintainer answered (3).
7. **Trochoidal replaces each full-width stretch.** Each `FullWidth`
   stretch is cut as loops that walk the stretch itself (#3932: the path
   follows the contour and does not slot to a wall). A loop is
   `q + rho n (1 - cos t) + rho d sin t`: it leaves the stretch and comes
   back to it, one loop a step. Its radius is the room on the side with
   more room (the largest disc that touches the stretch there and lies in
   the centre area), at most half the loop width; so a loop cannot leave
   the area, and a point outside it is refused (`LOOP_LEAVES`). The step
   is the trochoid pitch, at most the load width of the stated limit
   (the front arc of a loop cuts about one step: Otkur & Lazoglu 2007,
   Rauch, Duc & Hascoet 2009). A loop with less room than 5 % of the tool
   radius is cut as the stretch: where no loop fits, the stretch stays and
   its reads are named `NO_ROOM`. Reads above the limit on the loops are
   `LOOPS_OVER`, on other curves `NOT_MARKED`. The marks of a changed
   curve go (their reads name what stays). A closed curve keeps the
   start that the order stage gave it.
   Dropped: a microlift return between loops (a link matter; the kernel's
   hop is stage 5); the loop sense from climb or conventional (the loop
   runs forward on the stretch, as the ring runs).

**Readings** (measured, not tuned).

The law suite (`tests/laws/load_stage.rs`; the check grid of the suite
walks the plan; hot = a read above `load x 1.05`):

| Fixture (load) | None: curves, hot reads, worst | Constant engagement: curves, hot reads (unnamed), worst |
|---|---|---|
| disc pocket (0.25) | 4, 3, 0.413 | 9, 0 (0), 0.179 |
| pocket with an island (0.20) | 12, 286, 0.999 | 29, 146 (0), 0.998 |
| two lobes and a neck (0.40) | 9, 249, 0.998 | 27, 128 (0), 0.998 |

The island and the neck keep their full-width first rings and their
sharp corners: named (`FRESH_CUT`, `AT_FLOOR`). Trochoidal on the neck:
the worst read in the marked stretch is 0.941 (was a full-width 1.0);
the loops stay in the centre area.

Motion `kernel_laws::k3_load` (the #4154 measures, limit 60 degrees,
25 % load, the band round the part): feed moves above the limit, the
worst in degrees, and the moves above the limit with no named place
within one tool radius.

| Case | Old 2D Pocket | Kernel, None | Kernel, constant engagement |
|---|---|---|---|
| disc Inside | 20, 178 | 3507, 173, 0 | 833, 173, 1 |
| disc Outside | 20, 178 | 402, 145, 13 | 334, 118, 74 |
| dog bone Inside | 32, 180 | 2587, 173, 0 | 581, 173, 1 |
| dog bone Outside | 32, 180 | 724, 167, 3 | 678, 143, 66 |

The kernel counts every short move of its dense rings (the old planner
has fewer, longer moves), so the counts are not one to one. The worst
Inside (173) is the first ring round the part: full width, named. The
unnamed moves Outside are on the stock edge line, where a piece starts
after a lift: there the kernel read and the #4154 meter differ. That is
follow-up #4260 (P7). The one unnamed move Inside is a link feed (stage 5).

**Cost of a change later.** A new spacing law is one function in
`rings.rs` behind the same reads. A tool shape other than a disc changes
only the edge read in `law.rs`.
