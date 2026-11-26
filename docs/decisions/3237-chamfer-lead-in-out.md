## #3237 - Configurable 2D Chamfer lead-in and lead-out

**Decided:** 2D Chamfer uses the shared five-shape `Lead` vocabulary and
the existing `arc_leads` constructors. Runtime and project `lead` use an
outer optional value as the pre-v83 compatibility marker. An absent lead
keeps the old fixed-width open-chain extension. A stored
`Lead::None` means that a newly stated operation adds no XY lead.
`lead_out` keeps the existing "absent means same" convention.
New 2D Chamfer operations seed a 3 mm perpendicular lead, with the lead-out
mirroring it. Existing project files still use the absent-key compatibility
marker. Users can state `Lead::None` to remove the added XY move.

The Chamfer planner may halve each end independently through four half
steps. It names the used value. It refuses when neither end fits. The
candidate is checked for self and prior-run crossings and is swept with
the existing full `solid_hits` and `stack_hits` verification surface,
including the cone and the modelled body. No one-millimetre radius model
is added. The verifier-only path splits the final lead at the authored
outer cone-band reach. It excuses a `Gouge` only on that exact seam move
and on the selected cut transition. Body and holder warnings are never
excused.

Chamfer exposes only lead-in and lead-out shape plus the selected length
or radius. It does not add Contour Entry, links, tabs, compensation,
overlap, extension or vertical-lead controls.

**Why:** This reuses one lead engine and its DTO vocabulary. It preserves
old saved paths and project bytes while making the difference between a
legacy fixed approach and a new explicit no-lead choice visible in the
file and digest. Perpendicular is the default because it moves directly
onto the already-classified air side; a linear lead follows the selected
tangent and can leave material or fixtures less room at an endpoint. Three
millimetres follows the existing lead-in length scale and leaves room for
the planner to halve the request. The full verifier already reads the exact
tool stack, so a second safety model would drift.

**Considered and dropped:** Replacing absent fields with `Lead::None`
during load was dropped because it changes old open-chain motion.
Reusing Contour's `lead_fit` was dropped because its cutter-region and
bare-wall fallback laws belong to Contour, while Chamfer must refuse.
A sampled one-millimetre clearance disc was retained only for the
legacy path; new requested leads use the full verification surface.

**Cost to change:** A different file compatibility state requires
another project rung and a new migration pin. A new lead shape must be
added to the shared enum, DTO, selector, digest and `arc_leads`
constructor together. Replacing the exact collision rule requires
motion, app and simulation review because it can change posted safety.
