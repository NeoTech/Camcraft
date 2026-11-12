## #1094 - A from-model stock's source is judged as the part unless the user marks it raw stock only

**Decided (after review of PR #1097).** The program verdict leaves a model out
only when the user said so: a checkbox on the stock page, "The source model is
raw stock only (not judged as the part)", shown for a stock made from a model.
**Off is the default** and every older document's state: every non-fixture
model is judged exactly as before. The mark is one undoable command
(`SetStockSourceOnly`), with an MCP tool (`set_raw_stock_only`) running the same
command, a component on the stock entity (`schema::StockSourceOnly { source }`)
and an optional project-file key (`raw_stock_only`, absent = false).

Even when on, `stock_source::stock_only_model` keeps it honest:
- the mark names the source it was said of; it holds only while every stock the
  program starts from (the own stock of each setup the unsuppressed ops machine
  in; a setup taking what the one before leaves is skipped) is a copy of that
  same model and carries the mark. `SetStock` to another source drops its hold;
  undo of that `SetStock` restores it.
- an operation of the document - any setup, suppressed or not - that names the
  model in its source (`Entity`, any `Selection` entity) or in a boundary pick
  of its area controls puts it back in the verdict. Chosen over refusing the
  command while such an op exists: a refusal at tick time cannot stop an op
  added later, the reader-side rule holds whatever order things happen in.
- a mark that leaves no other non-fixture model (nothing judged as the part)
  refuses the post by name ("nothing is judged as the part: ...") - the
  stricter of refuse or warn.

**The hint is the only inference.** A gouge refusal whose verdict read the
program stock's source model adds one sentence: how to mark it (when no op
names it), or why the mark did not hold (an op machines it). It changes no
verdict.

**Why explicit, not inferred (orchestrator's direction, taken unattended).**
The first version inferred "only the stock's source" from three clauses (the
source of every program stock, named by no op, while some op names another
model). The review reproduced a real 5 mm gouge into the finished part posting
with an empty gate: a stock made from the part itself plus any unrelated model
an op touches (live or suppressed) met the clauses, and boundary picks were not
seen. Any rule that guesses which model is the part can be met by a job it did
not foresee - the suggested envelope test is defeated by a deburr or engrave
job on a finished part with a small reference body inside its envelope - and a
wrong guess posts a gouge, the worst failure this project has. Per the rule
that a CAM choice is a user parameter with a safe default, the user decides.

**Considered and dropped.**
- *The three-clause inference* (above) - removed.
- *Patching the inference clause by clause* (unsuppressed ops only, picks,
  an envelope containment test) - each patch narrows the hole, none closes it.
- *Refusing the mark while an op names the model* - kept the reader-side rule
  instead (see above).
- *A named warning instead of a refusal when nothing is left to judge* - the
  stricter reading was taken.

**Project file.** No version bump: the key is additive and optional, and an
older reader that drops it judges the casting - a false refusal at worst,
never a missed gouge. An unmarked document saves byte-for-byte as before.

**Other readers of "every non-fixture model", unchanged.**
- `model_fidelity::verdict_tol` (#864/#911 model-excess law) still reads every
  non-fixture model, marked or not: a coarse casting can still tighten or refuse
  the verdict - a false refusal at worst.
- Per-op collision, rapid and cross-drill laws read the op's own source or the
  stock (`operation::job`), never the document part set.
- Rest material reads the stock walk, not the part set.
- Alignment, picks, fixture clash, `part_centre`, the tree and the GPU part
  list are placement and display, not judgement.

**Cost to change later.** One module (`crates/app/src/stock_source.rs`), the
command, two verdict call sites and the gate clause in `posting_sim.rs`.
