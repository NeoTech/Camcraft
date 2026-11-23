## #3469 - Part-off: the user states the piece kept and a back stock

**Decided.** The Part (part-off) operation gets one setting,
`kerf: PartKerf` (`motion::turning::parting_law`), with two values:

- `kept`: which piece is the finished part. `CutOff` (the default) is the
  #3452 rule: the piece cut off the free end is the part, and the whole
  kerf is on the chuck side. `Chuck` keeps the piece in the chuck: the
  whole kerf is on the free-end side, and a stub comes off the free end.
- `back_stock` (mm, 0 or more, default 0): the whole kerf stands this far
  off the parting plane, into the scrap side. The part keeps that much on
  its end face for a later facing pass.

The page shows them on the Part op's Passes tab as "The part is" (Cut-off
piece / Chuck piece) and "Back stock". The project file has the keys
`kept` (`cut_off` / `chuck`) and `back_stock`. Each key is written only
when it is not the default. MCP `edit_operation` patches them through
`EditOperationParams` (one undo step). A negative or non-finite back
stock is refused at the command door (`is_sane`), in the kernel and in
the verdict.

**The gouge law judges against the kept part.** `part_kerf_refusal_for`
reads the scrap side from `kept`, and it treats the back stock as part
material. The back stock zone has no model facets, so its radius is the
model's largest radius within one back-stock length of the end face. A
blade band in that zone, below that radius, is refused by name.

**The flipped-setup refusal of #3472 is narrowed, not removed.** Where
the model goes on past the parting plane on both sides in a setup that
holds the part end for end, the default (`CutOff`) is still refused, and
the refusal now names the setting: "Set 'The part is' to 'Chuck piece'".
With `Chuck` the part-off generates.

**Existing projects post the same program.** The default is the #3452
kerf. The stamp folds the setting only when it is not the default. An op
saved after #3452 keeps its program and its stamp. An op saved before
#3452 reads stale through the #3472 salt (see the refresh note below).
No golden or catalog file moves.

**Why.**
- The maintainer's scope (issue comments of 2026-11-22): one parameter
  with a default, "the piece that is cut off (default) / the piece that
  stays in the chuck", the kerf fully on the other side, and the back
  allowance kept in scope.
- One struct field instead of two keeps each `OpParams::Part` literal a
  one-field change, and the kernel and the verdict read one value.
- The flipped-setup default stays refused because the default there cuts
  into the finished part. The user has a way out: the setting.

**Dropped.**
- A "blade centred on the plane" option (the first text of the issue).
  It puts half the blade into the part, which the gouge law must refuse.
  A part that a later op faces to length is a back stock instead.
- Removing the flipped-setup refusal entirely. The default in such a setup
  would cut the finished part with no warning.
- A setup or machine field for the chuck side. The issue scope puts the
  side on the operation. A setup field can come later and set the default.
- Folding the setting into the stamp always. That would read every saved
  Part op stale once with no change to its program.

**The refresh of 2026-11-23 (merge of main).** The branch took
`origin/main`, which includes #3472. The safety mechanisms reconcile as
follows.

- #3472's `part-off-kerf-side-v1` stamp salt is in the branch. A Part
  cache made by the old centred-kerf algorithm reads Stale and does not
  post until the op is generated again. The test
  `part_off_saved_before_3452.rs` opens such a saved project and pins
  the refusal and the safe regeneration.
- #3472's `part_off_problem` and `plane_inside_model` refusal stays for
  the default (`CutOff`). When the user sets `Chuck`, the setting states
  the side and resolves the ambiguity that this refusal guards, so the
  ambiguity refusal does not fire. The gouge law still judges the path
  against the part that `kept` names.
- The refusal message now also names the setting: "Or set 'The part is'
  to 'Chuck piece' to state the side." The user has the way out.
- The #3469 branch's own blanket refusal by `reverses_axis` is not kept.
  #3472 replaced that refusal with the precise `part_off_problem` rule
  and removed `reverses_axis`. The precise rule covers the danger case
  (the model goes on past the plane on both sides in a setup that cannot
  say which end is free), and it is narrowed above only where the user
  states the side.

No safety rule is dropped. A default part-off in an ambiguous setup is
still refused before a move exists. An explicit `Chuck` states the side,
and the gouge law judges the path.

**Cost to change later.** The file keys are additive. A third `kept`
value, or a setup-level default, adds an enum arm and a DTO arm.
