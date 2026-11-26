## #4176 - Path kernel K2, first part: the ring and the row guide planners

Stage 2 of the K0 record (`## #4024` in `RECOMMENDED.md`, "Stage 2: Guide
curves"), first of two sub-PRs. This part adds the ring planner and the
row planner, the two families that 2D Pocket (the first kind on the
kernel) needs. The medial line, the trace of a chain and the helix are the
second sub-PR. No old planner changes, and no planner calls the new code
yet: with the default switch the output is identical.

**What was decided**

1. **Files.** `crates/pathkernel/src/guide/` as the record names it:
   `rings.rs`, `rows.rs`, `mark.rs` (the named stretches), `cover.rs`
   (the shared checks and the completeness guard) and `all.rs` (`Guides`,
   the stage as one planner). `guide.rs` keeps the types. The kernel
   crate still depends on `camcraft-geometry` only. No geometry function
   was added: `Region::offset`, `sweep`, `difference`, `clip_ring` and
   `clip_polyline` are enough.
2. **The rings are level sets of a distance.** Ring `k` is the boundary of
   the area at distance `first_offset + k * stepover` or more from the
   reference walls (Held 1991). `RingRef::Boundary` is the erosion of the
   centre area (all its edges). `RingRef::Part` is the centre area less the
   swept band of the part walls; the stretches on the stock edge are cut
   away, so a ring round a part is open where it meets the stock outline.
   Where the region has no stock edge, `Part` and `Boundary` are the same
   computation (pinned). The step comes from the choice on each call; no
   step is fixed or read once (#3933 stays with K3). The rings are written
   new; no code path of the old ring planners is ported (the record lists
   their defects: #3788, #3933, #2232, the fixed `RING = 4` fill depth).
3. **The completeness guard and clean-up rings.** For a choice with no ring
   count, the planner adds clean-up rings until the tool discs cover the
   centre area: each uncovered piece of 1e-4 mm2 or more gets the boundary
   of the piece eroded by half of min(tool radius, stepover) (the piece
   itself when it is thinner), at most 6 rounds. A clean-up ring is of
   family `Ring` and has the index of the next deeper ring (one more than
   the deepest level area that holds it), and it is marked `Fill`. If more
   than 0.1 mm2 stays uncovered, the verdict is `Incomplete` with each gap
   named; a remnant at or under 0.1 mm2 is named in a note. Pieces under 1
   um2 are lattice noise. Dropped: a fixed number of sub-levels between two
   rings (the old `ring_fill`, `RING = 4`): its depth is a constant, not a
   law, and it does not stop at a covered area.
4. **A ring count asks for rings, not for a cleared area.** With
   `count: Some(n)` (Contour) the planner gives at most `n` rings, no
   clean-up ring and no guard.
5. **Full-width stretches are named, not hidden (#3788).** A stretch of a
   curve of index `k` is marked `FullWidth` where its tool disc overlaps
   no disc of a curve of index `k + 1` and no air within a tool radius of
   the stock edge. When the deeper curves are cut first, the tool cuts
   there at full width. A deeper curve than `k + 1` need not be read: it is
   inside the level area of `k + 1`, so a curve of `k + 1` is nearer. The
   test is exact in the plane: the stretches are the clip of the curve by
   the swept area of the deeper curves at twice the tool radius. Only the
   order and the load stages can say if the cut is really full width, so
   the guide names the place and does not change the curves. The deepest
   rings of a closed pocket are marked whole: in an order from the inside
   out they are the first cut, as a slot. On the waist pin (two lobes and a
   neck narrower than two steps), the middle 14.5 mm of each strand of ring
   0 through the 21 mm neck is named, and each mouth of the neck gets one
   clean-up ring. Dropped: the distance from a cut to the nearest earlier
   cut (the old `full_width_at`), because it needs an order and it missed
   the waist band (#3788).
6. **Marks are geometry, not point indices.** A `Mark` names a curve by its
   index in the set and holds a whole-curve span or the points of a
   stretch. `GuideSet::with_marks` moves each mark to the canonical index
   of its curve and refuses a mark on a missing curve or a stretch with a
   point off its curve. So the canonical rotation of a closed curve does
   not move a mark. `GuideSet::new` stays as it was (no marks).
7. **Small pieces are kept.** A piece of the centre area of any size gets
   its ring (pin: a 0.01 mm2 piece). A raw curve that the curve checks
   refuse is named in a note, never dropped silently.
8. **Rows.** Rows run at the choice's angle, evenly spaced at the stepover
   or less, from one extreme of the area to the other, so that a wall
   parallel to the rows is cut. Each row is clipped to the centre area
   grown by a quarter of the tolerance, so a row on the boundary is kept;
   a piece shorter than the tolerance (a row that touches a corner at one
   point) is not a row and is counted in a note. Each piece is an open
   curve of family `Row` with the row's index. The closing ring (when
   asked) is the boundary of the area, family `Ring`, index 0, marked
   `Closing`. Rows get no `FullWidth` marks: their engagement is a matter of
   the order (K4) and the load (K3). Rows do not get clean-up rings: a ring
   in a row set is another family, so the uncovered area makes the verdict
   `Incomplete` with the gaps named (pinned on a disc at a step of the tool
   diameter without a closing ring).
9. **Named refusals.** `NOT_RINGS`, `NOT_ROWS`, `CIRCLE_NOT_YET` (exact
   circle rings come with the helix guide), `BAND_NOT_YET` (the band
   reference comes with K4, #3934), `OFFSET_BELOW_CENTER`, `NO_RING`,
   `TOO_MANY_RINGS` (10000, checked before the work), `TOO_MANY_ROWS`
   (100000), `STEP_ABOVE_DIAMETER`, `BAD_TOL` (a tolerance under the kernel
   tolerance), `NOT_PLANAR`, `GUIDE_GEOMETRY`, and for marks `BAD_STRETCH`,
   `MARK_NO_CURVE`, `MARK_OFF_CURVE`. `Guides` refuses the medial line and
   the trace with `FAMILY_NOT_YET` until the second sub-PR: no family is
   planned as another one (law 4).
10. **The curves are in the frame of the region.** The planners read the
    2D coordinates of the region, which are along the axes of its work
    plane; no planner reads "XY". Each point has depth 0 (the level of the
    plan).

**Agreement with the old rings** (motion pins in
`crates/motion/tests/pass_start_levels/kernel_rings.rs`; two paths agree
where their points are within 0.02 mm):

- Dog bone, 2D Offset round the part (`RingRef::Part`, step 2, tool
  radius 4): the kernel gives 29 rings (4539 points). Each point is on each
  of the seven old plans with a known stock (`auto`, `inside`, `outside`,
  `inside_rings`, `outside_rings`, `inside_arc`, `outside_arc`): one ring
  set for each start side (#3750). Named difference: the old plan also
  has a pass along the stock outline (its band's outer edge, rounded where
  it meets the rings); each of those points is within one step of the
  outline. The kernel does not walk the outline: the stock edge is air
  (#4025 decision 4), and the last ring's tool disc already cuts past it.
  The guard confirms that the area is covered.
- `offset2d_dogbone_auto_blind.cl` is not compared: without the stock the
  old plan rings the part out to the band's outer edge, and an Outside
  kernel region needs the stock (#4025 decision 8).
- The band round the disc, 2D Pocket (`RingRef::Boundary`, step 4): the
  kernel's 8 rings (1112 points) and the old Auto plan agree both ways.
  The 3-D level rings on the same band: the kernel's 10 rings (1142
  points) and the old Auto passes agree both ways. Named difference: the
  old Inside and Outside plans build their own pieces for each start side
  (#3316; the defect of #3750). The kernel has one set, and it is the old
  Auto set.
- The entry (the helix and the point where it reaches the cut level) and
  the links are stages 6 and 5, so the pins do not compare them.

**Why**: the record fixes the guide signature, the laws and the guard,
but not how a full-width ring is named, how a gap is filled, or what a
ring count means for the guard. The choices keep the guide a pure function
of the region and the choice (laws 1 to 3), keep every substitution named
(law 4), and leave each order-dependent judgement to K3 and K4.

**Cost to change later**: low. The planners are internal to the kernel. A
different full-width rule or fill rule changes `rings.rs` and its pins.
`MarkKind` is `#[non_exhaustive]`, so a later stage can add a kind.
