## #1056 - the reduced-fidelity flag: a stock copy carries it, the refusal wins over the sim, the colours are tightened; older files left open

**What was decided.** Three of #1056's four points are fixed; the second
(projects saved before #911) is left open on the issue.

1. *A stock copied from a coarse model carries the chord error itself.*
   The flag describes facets, and a `FromModel` stock holds a copy of
   them. `SetStock` writes the source's chord onto the stock (and its
   undo puts back the stock's old one); `UpdateModel`'s propagation trades
   it with the copy's facets both ways; the datum-transfer clone
   (`CreateSetup`) and `DeleteSetup`'s undo carry it in `StockSnap`;
   `DeleteEntity` already restored any entity's flag. The project file
   saves it on the stock row (`coarse_mm`, the same key as the model's)
   and the loader restores it. Deleting or renaming the source no longer
   changes what the laws read.
2. *Older files.* A `FromModel` stock from a file written while the flag
   lived only on the model (#911's window) adopts its source's flag on load
   (`model_fidelity::adopt_copies`), and `coarse_source` still falls back
   to the source by name. Files from before #911 are **not** solved here
   (below).
3. *The named refusal wins.* The program sim and the eye judge at
   `model_fidelity::program_tol`: the gate's own refusal line when any
   program op is refused for its model's fidelity, else `verdict_tol` as
   before. The sim then reports "did not judge" with that line, and the
   gate shows the refusal alone - never beside a gouge line read off the
   same material. The case found: a fine part in a stock copied from a
   model too coarse for the op's laws, once that model is deleted (or, in
   general, any op whose own floor sits below the sim's 0.01 mm, or whose
   coarse model is a stock the part list does not show). Before, the sim
   judged at the plain 0.01 mm and added a gouge line next to the refusal.
4. *Display tolerances.* `FieldView` carries the tolerance its colours are
   gated at (`tol`), filled from `program_tol` by `field_view` and by
   playback (`PlaySeed`/`FieldPlay`); `machined_classes::mesh_classes`, the
   GPU eye (`trace/eye.rs`) and the CPU eye (`trace/cpu_eye.rs`) read it
   instead of `DEV_TOL`. When the program is refused, the picture has no
   target and its note is the refusal (no colours off a mesh that cannot
   carry them).

**Decisions taken unattended.**

- *A refusal names the model the facets came from,* also after it is
  deleted: "model 'blank' was imported too coarsely ...". That is what a
  person re-imports; naming "Stock" would point at a solid nobody imported.
- *Only a `FromModel` stock row is read for the key on load;* a box or
  cylinder stock is meshed by the loader and is never coarse, so a
  hand-edited key there is ignored.
- *Point 2 is left open, not guessed at.* A project saved before #911 has
  no way to say whether a STEP model in it was kept coarse, and the file
  version did not move with #911, so such a file and a later file with
  only fine models are indistinguishable. The honest options are all
  file-format work: bump `PROJECT_VERSION` and mark every model of an older
  file "fidelity unknown" (saved as such, so re-saving cannot launder it
  into "fine"), then either refuse its operations by name until the model
  is re-imported (safe, but refuses every existing project once) or report
  it as a named, non-blocking line. Re-measuring on load cannot be done:
  the chord error needs the B-rep surface, which the file does not keep.
  That choice changes every older project's post and is above this slice
  (`effort:max`: a file format); it stays on #1056 with this summary.

**Considered and dropped.**

- *Keep looking the source up by name, and refuse when it is missing.*
  Refuses every stock whose source was deleted, coarse or not, and still
  lies after a rename to a different model of the same name.
- *Two tolerances in `FieldView` (sim and display).* One number, the one
  the binding verdict reads, keeps "the mesh and the traced picture cannot
  disagree by a threshold" (`machined_classes`' law).

**Cost to change later.** Low. The flag is one optional key on a stock
row; `program_tol` is one function the sim and the eye share.
