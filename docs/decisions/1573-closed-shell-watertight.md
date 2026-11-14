## #1573 - a STEP closed shell that meshes open is refused by name

**Decided.** After a shell's kept tessellation pass, the importer
(`crates/app/src/geometry/import/step_closed.rs`, called from `step.rs`)
does two checks:

1. A shell the file declares closed (a `CLOSED_SHELL`, or the outer shell a
   `MANIFOLD_SOLID_BREP` names) must mesh closed. Its seams are judged face
   by face (`step_seams.rs`), vertices welded within 1e-6 of the file's
   length unit (truck's own `TOLERANCE`), and three faults are told apart:
   - **Reversed face** (an edge two faces share runs the same way in
     both): the geometry is closed, one face is wound against its
     neighbours. **Fixed, not refused**: each face is flipped to agree
     with the faces it meets, propagated face to face; in each connected
     group is then turned to enclose a positive volume (outward). Logged as
     a warning naming the face. Only closed shells are re-wound: an open
     shell's winding stays as the kernel made it.
   - **T-junction** (neighbouring faces divide their shared boundary into
     different segments): closed when every boundary edge's ends and
     midpoint lie within 2 chord tolerances of another face's boundary
     (each side's polyline is within one tolerance of the shared curve).
     Not refused, logged.
   - **Gap** (a boundary edge with a point farther than that from every
     other face): refused, "closed shell #id (N faces) meshes open: K
     boundary edges lie more than G mm from any other face, most on face
     #F (i of N), up to W mm; the first from (..) to (..) mm[; face #D got
     no mesh from the kernel]".
The checks run on the pass the refinement KEPT, never on a pass it threw
away for the budget.

**Why.** truck drops a face it cannot place a boundary on (no polygon) and
trims wrongly one it places wrongly, with no error either way; the model
then imported with walls missing (the rounded cube at 57% of its volume)
and every verdict judged the program against it - the Prime Directive's
silent wrong answer. Downstream does not catch it: the default stock
backend takes an open mesh as a model, and only the SDF backend's
`validate_shell` refuses a non-watertight one.

**Revised after review** (#1576, NEEDS-REWORK 5899733965): the first
version counted every edge not used exactly twice in opposite directions
as open, and refused two healthy CAD exports of the maintainer's corpus
(13,227 and 9,762 "unmatched edges"; no face dropped). About 62% of those
were reversed faces, the rest T-junctions. Both are closed geometry: a
healthy CAD export is not refused for them.

**Prime-Directive call on the two corpus files (for the maintainer to
confirm).** With reversed faces fixed and T-junctions closed, both files
are still refused, for a gap this law exists to catch:
- `10208.STEP`: 2 faces flipped, 4,755 T-junction edges closed, and 211
  boundary edges of face #1073 (a `B_SPLINE_SURFACE`) lie up to 1.95 mm
  from any other face; a brute-force scan put most of them 16-64 chord
  tolerances from any other face's facets, not only its boundary.
- `32050.STEP`: 1 face flipped, 555 T-junction edges closed, and 4,435
  boundary edges, most on face #3350 (a `B_SPLINE_SURFACE`), lie up to
  4.16 mm from any other face.
The files are healthy; the mesh truck makes of their B-spline faces is
not (the faces #1602 already suspects: about 2 M facets each). On main they
import with those holes and nothing says so. Refusing them by name is the
Prime Directive's answer (a wrong model is worse than a refused one); the
fix that lets them import is #1602's, in the face meshing, not a looser
seam law. Loosening the gap to let them through would need over 800
chord tolerances (4.16 mm against 32050's 0.005 mm), wider than the 2.2 mm
hole the dropped-face fixture leaves.

**Considered and dropped.**
- *Keeping the kernel's winding for the side with more facets.* Tried
  first: on `10208.STEP` the two reversed faces are B-spline faces with
  4 M of the shell's 6.1 M facets, so the rule flipped the other 50 faces
  inward. The enclosed volume's sign is the outward test instead.
- *Refusing a shell whose face pairs still disagree after the flips.* Both
  corpus files show a few (8 and 1, around their B-spline faces); it is
  logged as a warning. No healthy closed mesh can have one, but no file
  reaches it without also having a gap, so it adds no refusal today.
- *Refusing a reversed face by name.* The fix is safe and local (reverse
  the facets of one face; the facets do not move), downstream reads a
  face's outside from its winding (`pick/winding.rs`, wall recognition),
  and the file is healthy: a refusal would block a good model.
- *Stitching T-junctions (splitting facets at the other side's points).*
  It would make the mesh edge-manifold, but it changes the facet set on
  every such model and no downstream consumer on the default backend
  needs it; the SDF backend's `validate_shell` still refuses such a mesh
  by name, as it does on main today.
- *A looser weld instead.* Measured worse in review: at 1e-4 the count on
  the first file rose to 396k (fine facets collapse into fans).

- *Volume shortfall in the message.* The true volume of the B-rep is not
  known without integrating the surfaces, and the signed volume of an open
  mesh is meaningless; the unmatched-edge count and the first edge's
  position are what the message carries instead.
- *Refusing a dropped face as soon as any pass shows it.* A coarse pass the
  refinement discards is not the model; only the kept pass is judged.
- *Repairing (filling the hole, re-meshing the face).* A guessed wall is a
  silent wrong answer of its own; the user re-exports or imports an STL.
- *An `OPEN_SHELL` / surface model meshing open.* Allowed: it may be open
  by definition. Only its dropped faces are refused.
- *Moving the seam check to `camcraft-geometry`.* It is used only by the
  importer, it needs the per-face facet ranges only the STEP pass knows,
  and a geometry change would push every gate to batch.

**Cost to change later.** The weld, the gap factor (`GAP_CHORDS`) and the
closed-shell set are constants and one function in `step_closed.rs`;
changing either is a one-line change, pinned by `import_step_closed_shell`
and the `step_seams` unit tests.
