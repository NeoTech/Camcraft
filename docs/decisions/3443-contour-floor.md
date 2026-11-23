## #3443 - Prove an exterior contour clear at every depth

**What was decided:** Build a conservative XY convex hull from all model
facets. Use the existing `derive_extruded_from_facets` API with
`ExtrudeAxis::Z`. Prove that every segment of the closed cutter-center ring
stays outside this hull. Keep the tool radius and stock allowance, with the
existing 0.008 mm clearance slack. Add a 0.00001 mm margin to this optional
proof. The margin covers the hull API's 0.000001 mm point deduplication and
polygon quantization, and makes the proof slightly stricter. A ring that
passes this proof keeps every requested depth. If the hull is unavailable
or the proof fails, use the existing section and overhang checks. Add no
dependency.

Use the hull proof only for rings that the user picked
(`contour_on_picked`, through `ladder_picked`). The whole-model contour and
the unpicked region contour keep the plain `ladder`. A whole-model contour
cuts the model's own outline. Its refusal of an unreadable section is the
signal that the mesh is broken (`contour_whole_step::whole_model_contour_that_cannot_reach_bottom_is_refused_with_the_stopped_level`).
The hull proof skips each section, so it hid that refusal and gave a clean
partial path. The premise of that test is correct: a broken mesh must
refuse by name.

Reject a picked contour when a required section is unreadable, even when
earlier levels are clear. Name the section error and the skipped levels.

**Why:** Issue #3443 reports that an outside contour stops at Z 7.061
instead of Z 0 and loses its tabs. A section error can become a false floor
note after earlier levels succeed. The hull contains the XY projection of
every triangle at every height. Clearance from that bound proves exterior
clearance without repeated interior slices. It does not treat an unreadable
section as empty material.

**Considered and dropped:** Do not apply the hull proof in the shared ladder for each contour kind. That hides the unreadable-section refusal of a whole-model contour. Do not ignore section errors, weaken wall
clearance, increase slice retry offsets, or give every Outside contour its
full depth. These choices do not prove the cutter clear. A general rewrite
of slicing is larger than this repair and changes checks used by other
operations.

**Cost to change later:** Replace the conservative hull proof with a tighter
bound if concave exterior features need it. Keep complete segment clearance
and the unreadable-section refusal. The existing depth ladder remains the
fallback, so this change does not require a new project format.

**Fixture placement:** Keep the saved mesh at
`crates/motion/tests/contour_outside_floor/fixtures/sign_plate_3443.json`.
Only this pin uses it. The gate supports this test-local layout and still
checks for other consumers. Shared placement under `tests/fixtures` was
dropped because it unnecessarily selects every motion suite.
