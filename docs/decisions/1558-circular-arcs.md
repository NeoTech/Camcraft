## #1558 - Circular rings post as arcs; the ring ramp rides helical arcs

**Decided:** the Circular kernel (`crates/motion/src/machining2d/circular.rs`)
cuts each ring at each level as two half-turn arcs about the authored centre
(G3 for climb, G2 otherwise: the travel the old polygon had, starting at the
same +X point). The ring polygon is kept for what the ladder plans with
(descent placement, links, the zig-zag ramp's length), but its chord count is
now the fewest power of two (8 to 512) whose sagitta stays within
`ARC_FIT_TOL` (0.01 mm), not a fixed 512. When the entry stands down to the
zig-zag ramp along the first cut (the helix that does not fit the ring), the
ramp is flown as helical arcs on the ring itself (`links::push_zig_ring`):
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
