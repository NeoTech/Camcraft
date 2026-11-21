## #2956 - Post verified reentrant punch profiles

Check the entire model against a vertical extrusion of the nominal region. Both cap unions must equal that region on the geometry lattice. The cap triangles must cover it once. Side facets must lie on finite boundary faces with the proper winding. The mesh must contain one consistently wound, closed solid. Sampling a few horizontal sections could miss a hidden step or cavity.

Use the largest measured wire-centre offset for the selected stock height. At each reentrant vertex, compare the distance to the miter with this offset. Permit up to 0.002 mm of residual corner stock, as in the strict Wire Contour cover law. Reserve space for the complete swept offset on every unrelated face and for both joins on every edge. This conservative test can refuse short facets that a more exact calculation could accept. Tightening the estimate requires a proof for every resident pass and controller checks with short facets.

Keep the perpendicular entry on a supporting exterior face. Keep the original line vertices in the native ISO object. The controller owns the rough, trim and separation geometry. Fitting analytic arcs to mesh chords would change the nominal part without source geometry to justify it.
