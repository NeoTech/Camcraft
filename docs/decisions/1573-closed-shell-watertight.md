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
     with the faces it meets (`step_orient.rs`, rules below). Said by name
     in the import result (`Graded::notes`, `ImportModel::notes`, the MCP
     / script `import_model` answer's `notes`, and the status line), and
     logged. Only closed shells are re-wound: an open shell's winding
     stays as the kernel made it.
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

**How the flips are decided** (#1576 second review, pullrequestreview-
5900373722: on `10208.STEP` three runs gave two flip sets, because hash
order drove a breadth-first walk; a cavity wound inward had all 6 faces
flipped, and a lone curved face was flipped by its own curvature):
- **Deterministic.** Edges, seam segments and shells are walked in sorted
  order, the votes kept in an ordered map; the result is a function of the
  facets alone, not of the hash seed or the facet order (pinned by
  `conflicting_votes_give_one_answer_on_every_run_and_in_every_order`: 64
  runs, each a new hash seed and a shuffled facet order).
- **Strongest evidence first.** Face pairs are joined in a parity
  union-find in the order: most exact shared edges (margin agree vs
  disagree), then T-junction votes, then face numbers. A pair that
  contradicts what is already joined is a contradiction, not a re-flip:
  one weak vote never overturns many shared edges.
- **The kernel's winding is the root.** In each connected group the side
  holding more faces keeps the kernel's winding (a tie: the group's lowest
  face's side). Nothing is flipped unless some vote says so.
- **Outward once, for the whole shell.** Only when that leaves some face
  flipped is the outward polarity decided, from the signed volume of the
  WHOLE shell, and every group turns together. A cavity wound inward and a
  lone face keep their winding; a connected shell gets the same answer as
  before (the corpus files, `the_flips_turn_the_shell_outward`).
- **What still disagrees.** A shared edge still running the same way in
  both faces after the flips (`UNRESOLVED_EDGES` = 0 allowed) is
  **refused by name**: "closed shell #id (N faces) is wound
  inconsistently: K shared edges still run the same way in both faces
  after the flips, most between face #A (..) and face #B (..)". No healthy
  closed mesh has one, and a face left wound against its neighbour turns
  inside and outside over for every check that reads its winding (the
  Prime Directive). A face pair met only across T-junctions that still
  disagrees is **said in the notes, not refused**: a T-junction vote is a
  nearest-boundary reading, not a shared edge, and every exact seam then
  agrees. The gap refusal comes first, so a shell with a gap keeps the gap
  message.
  On the two corpus files (second rework, release, 3 runs each, the same
  refusal every run) the solve cannot satisfy the shared edges either:
  `10208.STEP` shell #1319 keeps 31 shared edges reversed (one B-spline
  face disagrees with each of 24 small neighbours on all 32 of their
  shared edges while agreeing with face 48 on 807), and `32050.STEP` shell
  #4240 keeps 29 (the same pattern on one face). Both are refused for
  their gap first; without it they would be refused as wound
  inconsistently, not imported with a guessed winding. The first rework's
  "2 faces flipped" / "1 face flipped" was that guess.

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
- *Warn only for shared edges left reversed after the flips* (the first
  rework's rule). Dropped in the second review: the mesh would import with
  a face wound against its neighbour and only a log line saying so.
- *Refusing on T-junction-only disagreement too.* A T-junction vote reads
  the nearest other boundary within the gap tolerance; near thin walls and
  corners it can read the wrong neighbour, and no exact seam is then wrong.
  Said in the notes instead.
- *Outward per connected group* (the first rework's rule). It flipped a
  correctly wound cavity and a lone curved face; outward is a property of
  the shell, not of a group.
- *Breadth-first propagation in a fixed order.* Deterministic, but the
  first vote reached wins over stronger ones met later.
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
