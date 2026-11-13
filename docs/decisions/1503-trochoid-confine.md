## #1503 - a Trochoidal slot stays inside its selection, or is refused

**Decided:** when a Trochoidal slot has a selection picked (faces or edge
chains), generation checks that the slot's capsule - centre line A-B,
half-width = width / 2, the band the generator already keeps the cutter
inside (`contained`, motion `trochoid_slot`) - lies inside the picked
region: both endpoints inside, and no boundary edge (holes included)
nearer the centre line than width / 2 (slack 1e-4 mm, so a slot exactly
as wide as its pocket still fits). A slot that does not fit is refused
by name with the numbers (clearance needed vs found, where, and by how
much it would overrun). No selection: unchanged - the typed endpoints
rule, and the post's stock-sim gouge clause judges the path against the
model (pinned here).

**Why:** the issue's step-0 check showed the post already refuses such a
slot (stock-sim gouge clause), but generation reported it OK while its
path ran across the pocket wall. Refusal is exact, simple and cannot cut
the model; clipping the slot to the region would invent a different slot
than the one authored.

**Considered and dropped:** clipping the centre line to the region
(changes what the user typed, and is only provably safe for convex
regions); judging the generated moves against the region (arcs would
need sampling, and the capsule test is exact given the generator's own
band law); a depth law against the picked floor face (the verify pass
and the post's sim already judge depth, and a face pick need not be the
floor). The stand-alone operation stays; trochoidal motion as a 2D
Adaptive method (Expected item 2) is split into its own issue.

**Shared:** the law is `operation::slot_fit::refuse_unfit_slot(kind, a, b,
width, region)`, written so the plain Slot (#1504) can call it
unchanged once its arm resolves the picked region.

**Cost to change:** one call site in `operation/generate.rs`; the helper
is pure geometry with unit tests.
