## #1056 - projects saved before mesh fidelity was recorded load "fidelity unknown" and are refused until re-imported

**What was decided.** Project version 59. Loading a v58-or-older file
marks every mesh the file holds no fidelity record for - every model with
facets (fixtures included) and every from-model stock copy without a
`coarse_mm` - "fidelity unknown", with one load note per marked entity.
A recorded `coarse_mm` (a file saved after #911) is kept as it is. The
mark is the sentinel chord `model_fidelity::UNKNOWN_MM` (1e12 mm) on the
existing `ReducedFidelity` component, so it rides every path the #864/#1056
flag already rides (stock copies, setup clones, delete/undo, `UpdateModel`
propagation). Where it is read:

- the gate refuses every operation judged against it by name ("... was
  saved before CamCraft recorded mesh fidelity (project v58 or older), so
  a gouge check against it cannot be trusted; re-import the model to
  verify its operations"); an unknown part no operation names refuses the
  program through `verdict_tol`; an unknown fixture refuses it through a
  `gate_problems` line of its own;
- the program sim and the eye judge nothing and say that line
  (`program_tol`);
- the model page, MCP `get_job` and the script API say "Fidelity unknown"
  in words (`summary`, `facts` with `fidelity_unknown: true`), never the sentinel as a number;
- a fixture of unknown fidelity keeps its plain box at generation (no
  honest growth exists, and a box grown by the sentinel would collide with
  everything); the gate refusal above is what blocks the post.

The file saves the mark as `"fidelity_unknown": true` on that row (never
as a `coarse_mm` number), so re-saving cannot turn it into "fine"; a row
with both keys is a corrupt file, refused before any world write. Only a
fresh import of the model (`UpdateModel`, which records the measured
chord or none) clears it, and its undo puts the mark back.

The two committed projects whose meshes we generated ourselves
(`examples/first-part.camproj`, from its own OBJ generator, and the
microARC kit's project, built in code by its test) are re-stamped v59 by
hand: their meshes are the exact geometry, not a tessellation, so they
are known full fidelity.

**Why.** The Prime Directive: a gouge check against a mesh of unknown
coarseness is a silent possible wrong answer. Re-measuring on load (the
preferred option) is not possible: the chord error is measured against the
B-rep, and the file keeps only the facets. Nor can the mark be narrowed:
#911 did not bump the version, so a v58 file cannot tell a pre-#911 save
from a later one whose models all met the promise, and the file does not
record whether a mesh came from STEP or STL/OBJ. Tightening instead of
refusing would need a bound on the unknown chord, and the pre-#911
importer had none worth using (its schedule could keep a mesh up to 16x
the shell's extent when the budget bound, `step_refine`).

**Considered and dropped.**
- *Leave it and say so* (a non-blocking line): posts programs whose
  verdicts may be looser than shown - not safe.
- *Re-measure on load*: impossible without the B-rep (above).
- *Mark only STEP models*: the file does not say which models were STEP.
- *A separate `FidelityUnknown` component*: every copy, clone and undo
  path that carries the chord would need a second carrier, and missing one
  would silently drop the mark; the sentinel cannot be dropped by a path
  that carries the flag.
- *Grow an unknown fixture's box by the sentinel*: every move would read
  as a collision - a misleading verdict where a named refusal is honest.

**Cost to change later.** The price is that every project saved before
v59 refuses its operations once, until each model is re-imported (an STL
or OBJ model too, though its mesh is exact). A "vouch for this mesh"
command (undoable, with its MCP tool) that clears the mark for a model
whose source file is gone or known exact would remove that cost without a
format change; it is filed as a follow-up rather than built here.
