## #1119 slice 3 - Every input on the operation pages has a name

**What was decided.**
- `prop_row` opens its row while it draws the row's controls
  (`layout::named_by`). The seam's fields drawn inside it (`num` and its
  variants, `num_plain`, `value_with_unit`, `combo`, `combo_owned`,
  `tool_combo`, `drag_row`) take the row's label as their accessible
  name (`layout::name_by_row`). Outside a row, the fields get no name
  from the seam, so a field never takes the name of the row drawn before
  it.
- The operation header is a `Grid` with its own labels. Its Tool, Source,
  Setup and Coolant combos are named through `named_by` with the label of
  their row.
- The Spindle speed field (`spindle_row.rs`) is named by its row.
- `num_row`, `num_feed_row`, `num_plain_row` and `num_unit_row` already
  named by the row. They are now the same field as `num`, `num_feed`,
  `num_plain` and `num_unit`, so that no field has the same label twice.
- A guard test walks every kind of the registry and every tab that the
  kind has. It fails on an input with no name, or with one label named
  twice (`tests/op_page_names.rs`).
- The branch is stacked on the 2D slice of #1122 (PR #4221). That slice
  names five of the fields that the guard finds.

**Why.** On `main` 773736856, 1,840 of 1,861 input occurrences on the
operation pages had no name: `num` and `combo` set no `labelled_by`, and
pages call them, not the `*_row` variants. Naming in the seam fixes them
all at once and keeps a new page named.

**Considered and dropped.**
- Name each call site with `num_row` or `num_by`. That is about 280 edits,
  and a new page would forget it.
- Use `row_label_id` (the last row of the frame) in the base fields. A
  field drawn after a row, outside it, would take that row's name.
- Name the fields in `ring()`. `int_range_row` names its field itself, so
  it would have the same label twice.

**Cost to change later.** Small: the naming is in two helpers of
`layout/form.rs`.
