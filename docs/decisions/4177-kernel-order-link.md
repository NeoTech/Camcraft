## #4177 - Path kernel K4: the order stage and the link stage

Stage 4 (order) and stage 5 (link) of the K0 record (`## #4024` in
`RECOMMENDED.md`). Sub-PR of `side/path-kernel`, on top of K2 part 1
(#4176). No old planner changes and no planner calls the new code yet:
with the default switch the output is identical.

**What was decided**

1. **Files.** `crates/pathkernel/src/order/` (`side.rs`, `chain.rs`,
   `stage.rs`; the planner is `OrderStage`) and
   `crates/pathkernel/src/link/` (`proof.rs`, `route.rs`; the proof is
   `LinkStage`). `order.rs` and `link.rs` keep the types. The kernel still
   depends on `camcraft-geometry` only. Geometry got two public functions
   with their tests: `geo2d::segments_cross` (a proper crossing, the
   verifier's crossing test) and `geo2d::segment_gap` (the distance of two
   segments), and `geo2d::point_segment_distance`.
2. **The marks go with their curves.** `OrderedSet::new` now moves each
   mark of the guide set to the index of its curve in the ordered set, and
   gives a stretch of a reversed curve in the new direction. No planner
   can drop or move a mark: the constructor does it (valid by
   construction). The load stage (K3) reads `OrderedSet::marks`.
3. **A full-width stretch is cut last.** Of the curves that are ready
   to cut, a curve with a `FullWidth` stretch comes after the others, and
   a closed curve starts where its longest full-width stretch ends, so
   that the heavy stretch is the last part of that curve. Why: by then all
   that the other curves can clear round it is cleared, and the tool
   leaves the heavy stretch into cut ground; the link into the curve
   reaches cleared material, not the heavy cut. The walk's sense is not
   changed for a mark: the start side owns it (law 3). Dropped: "cut
   first" (the tool enters the heavy cut straight after a link, with less
   cleared round it); moving the stretch to another level (it would change
   the Inside/Outside walk). K3 still reads the load and names or slows
   the stretch.
4. **Auto** resolves to Outside when the region has a stock edge with air
   beyond it, else to Inside (the rule of `ring_side::resolve`, read from
   the region). The `stock` argument of `order` is not needed for it.
5. **The walk's sense.** Ring `k + 1` is deeper than ring `k`. For a
   pocket the deeper rings lie inward; round a part they lie outward
   (toward the stock edge). The stage reads it from the region: outward
   when the deepest rings lie nearer to the stock edge than the first
   ones. Inside starts at the part wall (round a part) or in the middle (a
   pocket); Outside starts at the stock edge or at the pocket wall (the
   maintainer's rule of #3741: the sides differ only in where the tool
   starts). A single level counts as deepest-first.
6. **Climb and conventional.** Climb keeps the uncut material on the
   right of the travel (a clockwise spindle). The uncut side is the deeper
   side when the deeper rings come later, else the shallower side. A
   closed ring has the deeper side on its left (a loop of a level area);
   an open piece of a ring round a part has it away from the part walls.
   So Inside on a pocket runs counter-clockwise, Outside clockwise, and
   conventional reverses each ring. Rows have no side: each row starts at
   its end nearer the tool (a zigzag).
7. **The sequence.** A ring waits for its neighbours on the side the
   walk comes from: the rings of the level before it in the walk that
   come within a tool diameter of it (the order of the offset tree, Held
   1991). Of the rings that are ready, the walk takes the first start, in
   this preference: a curve without a full-width stretch; the run that
   goes on the chain of the last run (the chain order of the Adaptive air
   plan, `air_chain::chains`, ported with its reach of 3 tool radii: the
   runs of one open region form a chain, the #3934 decision); the
   nearest start. A closed ring may start at one of its 8 vertices nearest
   the tool. The walk takes the first of these starts (at most 16) to
   which the link stage proves an in-plane link; if none is proved, it
   takes the first start and the link lifts with its reason. Dropped: a
   strict level-by-level order (it gave about 3 lifts on each level of the
   disc band), and the plain chain order without the proof (13 lifts on
   the disc band, Outside).
8. **The link proof.** A hop stays at depth only when one in-plane path is
   proved. Each chord of the path must:
   - stay in the cutter-centre area or the air (else `CUTS_PART`, or
     `HITS_FIXTURE` within a tool radius of a keep-out area);
   - cross no cut of the level: a planned cut must not be touched away
     from the two ends of the link; a made cut must not be crossed (the
     verifier's crossing test), but the path may run along it
     (`CROSSES_CUT`);
   - run over cleared stock: each sample (every quarter tool radius)
     within 0.9 tool radius of a made cut, in the air, or within 0.1 tool
     radius of the stock edge, where half of the tool is in the air (the
     `AIR` margin of the ring links) (`UNCUT_STOCK`). Without that margin
     the disc band had 13 lifts on Outside: each step from one piece to
     the next along the stock edge ran over the strip that the next piece
     cuts.
   The tries: the straight chord; back along the open cut just made, then
   a chord (the chain link of `air_chain::link`); a walk along a loop of
   the centre area that both ends stand within a tool radius of, joined at
   the loop's vertices, the shorter way first (`ring_links::edge_route`,
   #3824). If no path is proved, the hop lifts with the reason of the
   straight chord. `LinkPolicy::Retract` lifts with `USER_RETRACT`. Every
   hop goes through the tally. The transition table of `Plan::new` stays
   the one gate (a lift needs an entry).
9. **The link trait reads the level.** `LinkProof::link` takes a
   `Level { made, planned }` in place of one list: the proof needs the cuts
   already made (cleared stock) and the cuts still planned (no crossing).
10. **Not ported.** The join of `air_links` (a lifted traverse, a retract
   hop): the kernel lifts only with a named reason, and the microlift
   becomes a `Hop` with its own reason in a later step. The stay-down
   length limit (`TOO_LONG`): `LinkPolicy` has no limit yet. The held
   level blend of #3934 (35 lifts on the disc, 24 crossing feeds on the dog
   bone): not ported; the pins show 0 crossings.

**Readings.** Motion `kernel_laws` (the #4154 measures; tool radius 4,
step 2, the 80 x 50 stock box), the kernel plan made into a toolpath in
the test, against the old planners' tables:

| Fixture | Kernel lifts / crossings | Old 2D Pocket | Old 2D Offset |
|---|---|---|---|
| disc, Inside | 0 / 0 | 18 (none named) / 0 | 0 / 0 |
| disc, Outside | 0 / 0 | 18 (none named) / 0 | 0 / 0 |
| dog bone, Inside | 0 / 0 | 26 (none named) / 0 | 0 / 0 |
| dog bone, Outside | 0 / 0 | 26 (none named) / 0 | 0 / 0 |

The held level blend of #3934 had 35 lifts on the disc and 24 crossing
feeds on the dog bone (Outside). The law suite pins the same 0 lifts on
the disc pocket, the disc band and the dog-bone band (Auto, Inside,
Outside), and the lifts of the pocket with an island: 7 Inside (and
Auto), 5 Outside, each with its reason. Inside, the four corners of that
pocket are four separate deepest areas: the walk starts in each, and the
link between two corners runs over a strip that is not cut yet
(`UNCUT_STOCK`) or across a ring still planned (`CROSSES_CUT`). Outside,
the walk must go from the wall rings to the island rings across the rings
still planned between them (`CROSSES_CUT`). No in-plane link is proved
there, so each lift is correct and named.

The engagement readings of the kernel plan are higher than the old ones
(for example `over 3504` on the disc Inside: the rings run outward from
the part wall, and each move of the dense rings counts). Load control is
K3: this step does not change the load.

**Cost of a change later.** The full-width rule is one function
(`order/stage.rs`, `started` and the level pick). The sense rule and the
side test are in `order/side.rs`. The proof's swath (0.9 r) is one
constant.
