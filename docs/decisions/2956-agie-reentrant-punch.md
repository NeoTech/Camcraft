## #2956 - Post verified reentrant punch profiles

Check the entire model against a vertical extrusion of the nominal region. Both cap unions must equal that region on the geometry lattice. The cap triangles must cover it once. Side facets must lie on finite boundary faces with the proper winding. The mesh must contain one consistently wound, closed solid. Sampling a few horizontal sections could miss a hidden step or cavity.

Use the largest measured wire-centre offset for the selected stock height. At each reentrant vertex, compare the distance to the miter with this offset. Permit up to 0.002 mm of residual corner stock, as in the strict Wire Contour cover law. Reserve space for the complete swept offset on every unrelated face and for both joins on every edge. This conservative test can refuse short facets that a more exact calculation could accept. Tightening the estimate requires a proof for every resident pass and controller checks with short facets.

Keep the perpendicular entry on a supporting exterior face. Keep the original line vertices in the native ISO object. The controller owns the rough, trim and separation geometry. Fitting analytic arcs to mesh chords would change the nominal part without source geometry to justify it.

### Port onto the zero-step setup (#2984) and the measured heights (#2964)

**Decided.** Keep the setup writer of main (`agie_setup.rs`, zero erosion steps, `ID_PUNCH c_punch`). The reentrant work changes only the nominal ISO object and the checks before it. The reentrant reach and the feature spacing use the rough offset of the selected stock thickness from `QN2_OFFSETS_UM`. The rough offset is the largest of the four resident offsets at every measured height, and a test pins that. Move the feature spacing check to `agie/spacing.rs` and the reentrant tests to `agie/reentrant_tests.rs`, to keep each file under 400 lines. The refusal texts name the native wire EDM job, not the vendor (#3537).

**Why.** The old branch wrote its own SBL with erosion steps. Main replaced that writer, and the control must import the work with zero steps. A notched punch on the same stock and start hole gives the same SBL as the rectangular punch, and the app test compares it with the `CCSEP20` golden file.

**Dropped.** A separate per-pass reach check for each trim offset. Each trim offset is smaller than the rough offset, so the rough check bounds them. A die setup for reentrant openings belongs to #2969.

**Cost to change.** If a new measured height has a trim offset larger than its rough offset, the pinning test fails. Then pass the largest of the four offsets to the reentrant checks.
