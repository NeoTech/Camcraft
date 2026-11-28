## #4338 - Tool library U12: the Holders page search and the holder form

**What was decided.**

- **The shape is the holder library's.** A user holder's editable shape is
  `holder_library::HolderShape` (new, in `crates/tools`): the taper, the
  clamp (an ER size, a Weldon bore or a shrink-fit bore), the bore, the
  gauge length, the nose diameter and the largest diameter. Its outline is
  `HolderEntry::points` of those values - the same audited builder as the
  built-in holders - so the law "never smaller than the steel" holds for
  the values the user states: an ER nut at its diameter over its whole
  length, a Weldon body `WELDON_SCREW` wider, a shrink nose and body
  `SHRINK_MARGIN` wider, the flange a full cylinder. The test
  `an_edited_outline_is_never_smaller_than_the_steel` samples the outline
  of a grid of edited shapes against the stated steel.
- **No new project-file field.** The shape is read back from the row's
  machine side, workpiece side and outline (`HolderShape::of`), and is
  accepted only when the outline it makes is the row's outline, point for
  point. A holder with any other outline (a DXF profile, an adapter) opens
  in the generic component editor as before. Dropped: a stored shape record
  on `DocumentComponent` - a project-file change and a hot `schema.rs` edit
  for data the outline already holds.
- **Largest diameter.** It is the widest steel of the body below the
  flange. An ER or Weldon body is as wide as its nose, so for those clamps
  the field follows the nose diameter and is disabled with that reason;
  only a shrink-fit chuck has two body diameters. The drawing's top
  dimension still shows the flange when the flange is wider.
- **Clamps from / to.** An ER chuck clamps the collet range of its size
  (ISO 15488): the two fields show it and are disabled. A Weldon or
  shrink-fit bore clamps one size: either field sets the bore. Dropped: a
  narrowed ER range - the workpiece side code `ER32` cannot state it.
- **Refusals.** A value that cannot describe steel is refused by name
  before Apply (Apply disabled with the reason, the text under the
  fields). An edit that breaks a tool's fit is refused by
  `EditDocumentComponent`, which derives each tool that uses the holder
  again; the refusal names the holder and what no longer fits, and nothing
  changes.
- **Units.** The form edits in mm, as the component table and the library
  facts store them. Dropped for now: inch input on this form.
- **Search.** AND over terms; a term matches the holder's words (name,
  taper, clamp) or, when it is a number, a holder that clamps that shank
  size ("BT40 ER32", "shrink 12", "er 16"). The result is kept for its
  input (text, taper pick, the library rows and the component rows,
  each an `Rc`), so a still frame searches nothing. The user holders
  section shows only when the document has one.
- **New holder** copies the machine taper's ER32 chuck (or its first
  holder; BT40 ER32 without a machine taper) as a user holder and opens
  it. **Duplicate** copies the applied user holder under a new `user:` id.
  Both are the existing `CopyAsUser` action, one undo step.
- **Not in this PR, filed:** Import / Export of holder files (#4389, the
  buttons are disabled with that reason), Edit outline and Import model
  (STEP) (#4390, same), the MCP tool for a shape edit (#4391; MCP can
  already replace the row with `upsert_tool_component`). The holder rail
  on the left of the mockup's form is not drawn: "‹ List" returns to the
  searchable list.

**Cost to change later.** The shape lives in `holder_library_shape.rs`;
a stored shape record or a new clamp kind changes that file, the form and
`HolderShape::of`'s read-back.
