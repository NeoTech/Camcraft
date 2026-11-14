## #1536 - A STEP face whose tessellation never converges is refused by name, not hung on

**Decided.** Every face of a STEP shell is wrapped in a metered surface
(`crates/app/src/geometry/import/step_guard.rs`) before truck tessellates it.
The wrapper counts the points truck evaluates on that face in one pass. B-spline
and NURBS faces are divided by truck's own generic `parameter_division`, called
on the wrapper. That is the same algorithm their own impls run, so the division
and the mesh do not change. Other surface kinds keep their specialised
division. When a face spends more than `FACE_EVAL_CAP` (25,000,000)
evaluations in one pass, the wrapper does two things. It answers every
further evaluation with one fixed point, so no cell is "far" and the
subdivision ends within one sweep. It also hands truck the bare range as
the division, so truck does not triangulate the millions of grid points the
runaway grew. The import is then refused with the face's STEP entity number,
its index and kind, its shell, the cap and what to do: "re-export the model,
or import an STL". A trip refuses even when a coarser pass of the same
shell had met, because a surface that jumps converges at any tolerance
wider than its gap, and keeping that mesh would smear the gap into a wall
that is not there.

**Why.** truck's subdivision splits every cell whose midpoint stands off the
bilinear patch of its corners and recurses until nothing splits. A surface
that is not continuous keeps one seam "far" at every depth. An example is an
interior knot repeated past its degree, which the pinning fixture uses. The
recursion then never ends: past float resolution a split at an unchanged
midpoint still grows the list. It runs inside a rayon task we cannot
interrupt, and it produces no facet, so the facet budget never fires. The
issue's sample matches this shape: the stack is `sub_parameter_division` over
`NurbsSurface<Vector4>`, and memory grows slowly. Metering the evaluations
bounds the loop without a thread, a timer or a patch to truck. It is
deterministic, so the pin is a plain test. The cap was measured. A
400 mm doubly curved bicubic patch at the 0.0025 mm chord cap needs 1.5 M
evaluations (a 475 x 475 grid). A discontinuous face reaches 25 M in about
1.7 s (release) or 4.5 s (debug, the whole import). A face needing 25 M
would be a ~3.7 M-point grid, or ~7 M facets from one face.

**Considered and dropped.**
- *Worker thread with a deadline (option c).* A thread inside truck cannot be
  killed. The stranded one keeps a core busy and its memory keeps growing (the
  issue measured 175 -> 322 MB in 4 min). That is a leak to cap and document,
  where the meter leaves nothing running. It would also make the refusal
  depend on wall-clock time and machine speed.
- *Tessellate face by face and skip the bad face (option b).* This gives a
  partial model, which must be marked incomplete. It also still needs (a) or
  (c) to stop the bad face. A refusal is the Prime Directive's default, and
  the user has the STL workaround.
- *Patch or vendor truck's `sub_parameter_division`.* This would be a
  new-dependency-shaped change that needs the maintainer. The wrapper reaches
  the same bound from outside.
- *Keep the coarser mesh when only a finer pass trips (as the facet budget
  does).* Tried, then dropped: on the pinning fixture it imported the jump as
  a 0.31-tolerance smear with no refusal.
- *A wall-clock budget per import.* Not deterministic, and a large healthy
  model can legitimately take longer than any fixed limit.

**Limits, and the cost to change later.** Only B-spline and NURBS surface
divisions are metered. Elementary and swept surfaces keep truck's own
division, and edge curves are divided unmetered, so a runaway in those
(none is known) would still hang. Covering them means a metered curve
wrapper and a decision on swept surfaces, which is filed as a follow-up.
The cap is one constant. Raising it costs only time on a broken face.
The real `sc_logo.step` was not available to this worker. The fix is pinned
on a synthetic face with the same failure mode, not on the user's file, so
the issue's own repro should be re-run on a build with this change.
