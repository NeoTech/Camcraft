## #1558 - Circular rings post as arcs; the ring ramp rides helical arcs

**Decided:** the Circular kernel (`crates/motion/src/machining2d/circular.rs`)
cuts each ring at each level as two half-turn arcs about the authored centre
(G3 for climb, G2 otherwise: the travel the old polygon had, starting at the
same +X point). The ring polygon is kept for what the ladder plans with
(descent placement, links, the zig-zag ramp's length), but its chord count is
now the fewest power of two (8 to 512) whose sagitta stays within
`ARC_FIT_TOL` (0.01 mm), not a fixed 512. When the entry stands down to the
zig-zag ramp along the first cut (the helix that does not fit the ring), the
ramp is flown as helical arcs on the ring itself (`ring_zig::push_zig_ring`):
the same legs, the same travelled length and so the same ramp angle, each leg
in at most two half turns. Only Circular opts in (`Motion::with_ring_zig`);
every other ladder's zig-zag is unchanged.

**Why:** the issue's Ø6 hole was 74,131 moves (0.006 mm feeds); it is now 417.
Half-turn arcs are what the helix entry already posts (`entry::helix_about`),
so the post and the verifier read nothing new. A power-of-two count divides
the 512-gon the cutter-centre disk certificate is built from, so every ring
vertex is still one of the certificate's vertices and descents certify as
before.

**Tolerance:** Circular has no tolerance parameter. `ARC_FIT_TOL`, the
constant the waterline rings and the depth kinds without a tolerance field
already use, sizes the polygon. Adding a `tolerance` field to Circular
(project file, MCP, UI) was dropped as more than the fix needs: the cut is
exact arcs; the polygon only plans.

**Considered and dropped:** arcs for the ring cut only, with the ramp still
walking the polygon's chords (measured: 2,749 moves for the repro with a
32-chord polygon; the 512-gon ramp would be far more); fitting the 512-gon
with `fit_arcs` the way Bore does (same ramp problem); gating helical arcs on a post capability (the helix entry
already posts helical arcs on every post, and the post engine handles them).

**Breaking?** No. It is a `fix:` that changes the posted output of existing
Circular operations: the same circles, depths, direction and entries, in far
fewer blocks. No parameter, command or file format changes.

**Cost to change later:** a Circular `tolerance` field would replace
`ARC_FIT_TOL` in `ring_loop`; nothing else reads it.

**Review follow-up (the M3 review on f3cb9d2):**
- *The ring ramp proves its own ring, the flag only opts in.* `push_zig_ring`
  (moved to its own module, `crates/motion/src/ring_zig.rs`) flies arcs only
  when every vertex of the planned prefix is within `ON_RING_TOL` (2e-6 mm:
  the 1e-6 lattice's half-diagonal plus float) of the ring, the prefix's last
  point (cut part-way along a polygon chord) is inside the ring by at most
  `ARC_FIT_TOL`, its first step turns in the arcs' sense, and the leg's arc
  end lands on the prefix end within `4 * ARC_FIT_TOL` (the arc-versus-chord
  length difference of a polygon held to `ARC_FIT_TOL` is under
  `2 pi / 3 * ARC_FIT_TOL`, plus the sagitta). Anything else posts
  `push_zig`'s chords, as before #1558. Dropped: a 1e-6 tolerance on every
  point (rejects the legitimate ring prefix whose end lies mid-chord); relying
  on a pin of the flag alone (a future opt-in would still fly a wrong circle).
- *No arc a post can round into a full circle.* `MIN_ARC_CHORD` = 0.01 mm:
  the coarsest length a shipped post writes is four-decimal inch (2.54 um per
  axis); a chord of `sqrt(2)` units (3.6 um) guarantees one axis differs by a
  whole unit, so the rounded endpoints differ. 10 um is that with margin. A
  ring-ramp piece under it posts the chords; a ring narrower than it walks its
  polygon. Considered: a guard in the post engine or `PathBuilder::push_arc`
  (no such guard exists in the repo to reuse; a shared one would change every
  op's output and its goldens - a separate decision, not this fix's). The
  helix entry's half turns share the class only for a helix radius under
  5 um; that path is not changed here.
- A golden (`tests/golden/circular_ring_arcs.cl`, 417 moves) now records the
  issue's repro, so any later change to the shape is a diff.
- Found and filed, not fixed here: Circular with `entry: ramp` descends at
  76-90 deg (#1609, pre-existing on main).
