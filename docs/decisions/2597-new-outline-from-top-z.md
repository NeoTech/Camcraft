## #2597 - Whole-model 2D Contour: a new, larger outline starts at Top Z

**Decided.** In a whole-model 2D Contour, a ring of a deeper band whose tool
sweep reaches outside every sweep cut so far gets every flat level above its
first level. So it is cut from Top Z down, one stepdown at a time, and not
first at the level where its outline appeared. The check is in
`crates/motion/src/machining2d/contour_whole_reach.rs` (`reach_up`). A ring
whose sweep stays inside the cut sweeps (an outline that does not grow) is
unchanged. If such a ring cannot be proven clear of the part at the added
levels (a slice error, or material over it), the operation is refused by
name, with no toolpath, so nothing posts.

**Why.** The contour has no stock outline. The material outside the part can
stand up to Top Z. A first pass at the new outline's level buried the tool by
the full height above it (20 mm on the #2597 part with stepdown 5). The
user's stepdown is the axial limit of the operation, so this is not a new
choice and not a new parameter: each pass is now at most one stepdown under
the pass before it on the same ring. The radial load of the added passes is
not more than that of the first band's passes: each pass engages at most the
tool width, as any contour pass does.

**Dropped.**
- Refuse every growing outline. Safe, but it refuses common parts (a boss
  on a plate) that can be cut correctly.
- A new operation parameter ("start new outlines at Top Z" on/off). The
  other value would post the buried pass again, which the Prime Directive
  forbids. So there is no choice to give.
- Read the stock outline to start each ring where the stock really starts.
  The contour generator gets no stock region today; that is a larger change
  (app and motion), and the Top Z rule is safe without it. It costs air
  passes where the stock is narrow. A follow-up can add the stock region.

**Cost of the rule.** A wall that grows a little at each level (a chamfer)
re-walks the new ring from Top Z at each band: more passes, mostly in air.
A strip narrower than 0.05 mm is taken as offset noise and does not count as
growth.

**Stock over Top Z (review blocker B1 of PR #2659).** When the stock top is
over Top Z (`stock_over`, the predicate by which `cut_levels_under` keeps Top
Z as a level), the material stands up to the stock top, not to Top Z. Then
the flat levels under Top Z are not enough: on the #2597 part with the stock
top at 28, the base ring was first cut at Z 20, 8 mm under the stock top
(stepdown 5). The rule (`upper_levels`) has two cases:
- The stock top is less than one stepdown over Top Z: the growing ring takes
  levels one stepdown at a time down from the stock top, until its first
  level. On that part the base ring is cut at Z 23, 18, 13, 8, 5, 0, and the
  boss ring (band 0) at Z 25, 20, 15, 10. These levels are off the flat grid
  on purpose (see "Dropped").
- The stock top is one stepdown or more over Top Z: band 0 bites that much
  at Top Z (the operation already says "Top below stock"). Only a pass at Top
  Z keeps the bite of the growing ring as small. So the ring takes Top Z and
  the flat levels under it.
- Without stock over Top Z, a level at Top Z removes no material, and no
  level at Top Z is added (unchanged).
- Dropped: add the Top Z level in every case (the review's first proposal).
  Top Z is the plane of the ring that band 0 cut there. On the chamfered
  island of `contour_whole_step` case 1 (stock top 17, Top Z 15) the lead-in
  of that ring crossed the grown ring 1 mm outside it at Z 15. The
  `Crossing` warning blocked the post of a correct part. With the levels from
  the stock top, the grown ring takes Z 13.5 there and shares no plane with
  the ring above it.
- Dropped: refuse the ring when the Top Z level crosses. It refuses a
  correct part that #2436 pins.
- Known limit: in the second case the two rings share the Top Z plane. On a
  part like case 1 with the stock top at 18.5 or higher, the verifier finds
  the same `Crossing` and blocks the post. The result is visible and safe,
  but a correct part does not post. #2677 tracks it.
- The added levels are proven by the same `ladder` as before. A ring that
  cannot be proven clear at them is refused by name.
