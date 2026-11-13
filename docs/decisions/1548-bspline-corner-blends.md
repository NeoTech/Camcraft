## #1548 - A three-fillet corner blend imports: its pole keeps parameters inside the knot domain, and surface-curve edges are read from their 3D curves

**Decided.** Two changes at the importer's wrapper around truck, and no new
tessellator.

1. `crates/app/src/geometry/import/step_pole.rs`. The metered surface
   (`step_guard`, #1536) now post-processes truck's parameter searches for
   B-spline and NURBS faces. An answer inside the knot domain is returned
   untouched. An answer outside it is replaced by a parameter inside the
   domain: the previous boundary point's value for each coordinate that left
   the domain, else the nearest bound. The replacement is taken only when
   the surface there is the searched point (within truck's own 1e-6).
   Anything else is left as truck found it.
2. `crates/app/src/geometry/import/step_edge3d.rs`. Before the shells are
   converted, every `EDGE_CURVE` whose geometry is a `SURFACE_CURVE` is
   pointed at that curve's `curve_3d`, and every surface curve's master
   representation is set to the 3D curve.

**Why.** The issue assumed truck cannot subdivide the corner blends because
they are not continuous. They are continuous. The #1536 model was rebuilt
here from the issue's description with solidcore (boxes united, all 30 edges
rounded R3.125, all opt-in fixes on, written by its step-export): 1 shell, 62
faces, 126 edges, 66 vertices, as the issue lists, and the same refusal from
PR #1546. Its corner blends are rational Bezier patches of degree (4, 2) with
one side collapsed to the corner vertex (a pole). Two separate faults were
measured on it:

- **The pole.** Every parameter along the collapsed side is a pre-image of
  the vertex. truck finds a boundary point's parameters by Newton iteration;
  at the pole the Jacobian is singular and the free parameter came back as
  -1,195,342,517 (knot domain 0..1). truck accepts it, because the
  extrapolated rational surface still passes through the vertex. It then
  subdivides the bounding box of the boundary's parameters, here a billion
  knot spans of extrapolation, which never meets the tolerance. On the knot
  domain itself the same subdivision ends in 3 to 5 rounds at every
  tolerance of the schedule. That is the hang of #1536 and the refusal of
  #1546.
- **The edges.** The file's rounded edges are `SURFACE_CURVE`s whose master
  is `.PCURVE_S1.`. truck then rebuilds the edge as a 2D curve on the
  neighbouring surface, in its own parametrisation of that surface and
  between parameters it searches for without a hint. With the pole fixed
  alone the model imported in 2.4 s with no refusal and was wrong: volume
  13,981 mm3 against 19,549.7, every planar face missing a corner. Switching
  only the master to `.CURVE_3D.` is not enough either: for an edge with
  `same_sense = .F.` truck 0.3.0 inverts the curve twice on that path and
  the edge runs backwards (pinned by the reversed-edge test).

With both changes the rebuilt model imports closed in 1.3 s (debug): 33,073
facets, volume 19,545.21 mm3 against the stated 19,549.71. The 4.5 mm3
shortfall is the inscribed mesh's chord error over the rounds, inside the
0.005 mm promise. The same holds with every edge reversed and with the face
senses turned.

**Literature.** ISO 10303-42: a `b_spline_surface` is defined on its knot
domain; `surface_curve.curve_3d` and its pcurves are one curve, and
`master_representation` names a preference. Piegl & Tiller, "The NURBS
Book", 2nd ed., ch. 4-5, for rational patches with a collapsed side. Nothing
from any CAM vendor.

**Considered and dropped.**
- *Our own B-spline tessellator, knot-span splitting, or a fallback grid*
  (the issue's three options). They would replace a subdivision that works
  once it is given the right range, change the mesh of every B-spline face,
  and leave the pcurve fault in place.
- *Clamping the subdivision range to the knot domain.* It hides a boundary
  that really runs off its surface: the grid would stop at the domain and
  the rest of the face would be meshed from boundary points alone, with no
  chord bound and no refusal. Left as before: such a face is subdivided as
  truck finds it, under #1536's meter.
- *Always replacing an outside answer by the nearest domain parameter.*
  Without the same-point check a boundary that leaves the surface would be
  moved onto it silently (pinned:
  `an_outside_answer_that_is_not_a_pole_is_left_as_truck_found_it`).
- *Patching truck.* Same reasons as #1536: the pin on truck 0.3.0/0.4.0
  stays, and both faults are reachable from our side of its traits.

**Not decided here.** A closed shell whose mesh is not closed still imports.
Before this change a plain rounded cube from the same writer (no pole
escaped, so nothing tripped) imported at 57% of its volume, 14,993 of
26,286 mm3, with no refusal; and some bent-blend variants of the new fuzz
test still import with the blend face dropped. That law (refuse a `CLOSED_SHELL` that does not mesh closed) is a
separate issue, named in the PR.

**Cost to change.** Both modules are private to the importer and touch no
project file. Removing either brings back its fault and fails
`import_step_corner_blend`. The fixture `testdata/models/fillet_corner.step`
is generated (solidcore step-export, face senses turned outward); the recipe
is in the PR body.
