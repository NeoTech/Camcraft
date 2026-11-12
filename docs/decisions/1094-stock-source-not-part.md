## #1094 - A model that is only the stock's source is not judged as the part

**Decided.** The program verdict's part set (`stock_sim::part_facets_as_modelled`
and `setup_align::parts_facets_in`, which every verdict reader goes through:
`simulate_document`, the carry's per-setup `judge`, the field view, judged
playback) leaves out at most ONE model - the one `stock_source::stock_only_model`
names - and only when all three hold:

1. the stock of **every** setup the program machines in (the unsuppressed
   operations' setups; with no operation, the active setup) is a `FromModel`
   copy of that one model;
2. **no** operation of the document - any setup, suppressed or not - names it
   in its source (`SrcRef::Entity`, or any entity of a `SrcRef::Selection`);
3. **some** operation names another non-fixture model in its source - there is
   a separate part to judge instead.

Otherwise every non-fixture model is judged, exactly as before.

**Why this reading.** The Prime Directive: the change must never take real part
geometry out of the verdict, so where several readings were defensible the one
that keeps more geometry judged was taken:

- *Several setups.* "The source of ANY setup's stock" was dropped for "the
  source of EVERY program setup's stock": a setup cut from other stock still
  judges the casting.
- *Picked for an op later.* Any op, in any setup, suppressed included, takes it
  back - not only the ops of the setup being judged, and not only ops that post.
- *Stock made from the part itself* (every op on `SrcRef::Stock`, no other
  model): clause 3 keeps the only model judged, so a user who made the stock
  from the finished part still sees the gouge.
- *`edge_break_gate.rs`'s brick* (the stock's source AND the op's source):
  clause 2 keeps it judged - today's verdict.

**Undo/redo and old files.** Nothing is stored: the rule is read from the live
document on every call, so undo/redo of `SetStock`, of `CreateOperation` /
an op's source edit, and a project file saved before this change all land on
the same answer with no migration. A pre-#1094 file whose casting blocked the
post now posts when the rule holds.

**Considered and dropped.**
- *An explicit "stock source only" mark on the model* (a command, an MCP tool,
  a schema field and a project-file field). It would judge the most geometry
  (nothing leaves the verdict until the user says so), but the issue's "Done
  means" is that a casting left in the tree simply posts; a mark the user must
  find first leaves the bug in place for everyone who does not. Worth revisiting
  if the automatic rule is ever found to drop a real part.
- *Per-frame exclusion* (leave the casting out only of the verdict read in the
  frame of a setup whose own stock it is). It would keep slightly more judged in
  multi-setup documents, but the as-modelled verdict has no single frame;
  clause 1's "every program setup" covers the same cases conservatively.

**Other readers of "every non-fixture model", unchanged on purpose.**
- `model_fidelity::verdict_tol` (#864/#911 model-excess law): still reads every
  non-fixture model. A too-coarse casting still tightens or refuses the verdict -
  a false refusal at worst, never a missed gouge. A follow-up could align it.
- Per-op collision, rapid and cross-drill laws read the op's own source or the
  stock (`operation::job`), never the document part set - unaffected.
- Rest material reads the stock walk, not the part set - unaffected.
- Part alignment, picks, fixture clash, `part_centre`, the tree and the GPU
  part list use `setup_align::is_part` / `model_entities` for placement and
  display, not judgement - unaffected.

**Cost to change later.** One module (`crates/app/src/stock_source.rs`) and two
call sites; a stored mark would replace `stock_only_model`'s body.
