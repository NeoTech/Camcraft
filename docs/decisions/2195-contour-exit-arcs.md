## #2195 - 2D Contour exit-point overlap re-traces the fitted walk and splits an interior hit on both laps

**What was decided**

1. **Project and extend along the fitted `walk`, not raw `poly`.** `contour_start::exit_at` (`crates/motion/src/machining2d/contour_start.rs`) now finds the exit pick's closest point on `walk` (handling both `Seg::Line` and `Seg::Arc`) and appends the prefix of `walk` up to that projection. Whole segments along the overlap — including rounded corner arcs from `fit_arcs_sided` and corner fillets from `fillet_corners` — are copied verbatim so the overlap emits the same `Arc` and `Line` moves as the first lap.
2. **Split an interior hit on the first lap too.** When the exit projection lands strictly inside a walk segment (whether a `Seg::Arc` or a `Seg::Line`), `extend_walk` splits that segment at the projection on the first lap as well and stops the overlap at that same projection. Hits within `1e-5` mm (`SNAP_EPS`) of a segment endpoint snap to that endpoint so lattice rounding of arc centers (`1e-6` mm) cannot invert the angle order or produce sub-lattice slivers.

**Why**

The verifier's crossing check (`camcraft_sim::verify::crossing`) tessellates every `Feed` arc into equal-angle chords at `gouge_tol` and applies `proper_cross_touch`. Previously `exit_at` appended raw `poly` chords (`Seg::Line`), whose vertices stood inside and out of phase with the first lap's fitted `Seg::Arc`, causing `verify` to convict the overlap as crossing and refuse post. Emitting only a partial `Seg::Arc` on the overlap when the exit lands inside an arc has the same problem: a shorter arc divides its sweep into different angles whose chords cross the full arc on the first lap. Splitting an interior hit on the first lap makes both laps emit identical `from`/`to`/`center`/`ccw` moves, and ensures `lead_off` starts from an explicit shared vertex on oblique edges too.

**Considered and dropped**

- **Exempting exit overlaps in `verify::crossing`.** Rejected: the triage explicitly requires keeping the crossing check intact and making the overlap path match the first lap instead of hiding findings.
- **Taking whole segments only (like `lead_ext::retrace` for `overlap`).** Rejected: a picked exit point must leave at the pick's projection, not overshoot to the next segment endpoint (#1473).
- **Re-fitting arcs on the extended polyline (`fit_arcs_sided` after extending `poly`).** Rejected: the seam split would alter where `fit_arcs_sided` or `fillet_corners` starts/splits an arc, so the overlap's arc endpoints or center could still differ from the first lap.

**Cost to change later**

Contained inside `crates/motion/src/machining2d/contour_start.rs` (`nearest_on_walk`, `arc_foot`, `extend_walk`); no parameter, CL format, or public API change.
