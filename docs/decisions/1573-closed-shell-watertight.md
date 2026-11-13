## #1573 - a STEP closed shell that meshes open is refused by name

**Decided.** After a shell's kept tessellation pass, the importer
(`crates/app/src/geometry/import/step_closed.rs`, called from `step.rs`)
does two checks:

1. A shell the file declares closed (a `CLOSED_SHELL`, or the outer shell a
   `MANIFOLD_SOLID_BREP` names) must mesh closed: every edge shared by
   exactly two facets running opposite ways, vertices welded within 1e-6 of
   the file's length unit (truck's own `TOLERANCE`; copies of a shared
   boundary point differ by about 1e-7). Otherwise the import is refused:
   "closed shell #id (N faces) meshes open: K unmatched edges, the first
   from (..) to (..) mm; face #F (i of N) got no mesh from the kernel".
2. A face truck returned no polygon for is refused by name in any shell,
   closed or not: a face the file has and the mesh lacks is a missing wall.

The checks run on the pass the refinement KEPT, never on a pass it threw
away for the budget.

**Why.** truck drops a face it cannot place a boundary on (no polygon) and
trims wrongly one it places wrongly, with no error either way; the model
then imported with walls missing (the rounded cube at 57% of its volume)
and every verdict judged the program against it - the Prime Directive's
silent wrong answer. Downstream does not catch it: the default stock
backend takes an open mesh as a model, and only the SDF backend's
`validate_shell` refuses a non-watertight one.

**Considered and dropped.**
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
- *Moving the edge counter to `camcraft-geometry`.* It is small, used only
  by the importer, and a geometry change would push every gate to batch.

**Cost to change later.** The weld and the closed-shell set are one
constant and one function in `step_closed.rs`; relaxing either is a
one-line change, pinned by `import_step_closed_shell`.
