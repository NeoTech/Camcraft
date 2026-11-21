## #2982 - Show the selected geometry source

**Decided:** Add and select a `Selection` entry when the operation draft has
source kind `2`. Keep the Stock and named model choices. Keep the existing
source edit commands and undo history.

**Why:** The old header forced a selection to the Stock index. The header
must show the saved source. This change does not change geometry or toolpath
generation.

Move `header_rows` and `coolant_row` to `op_page/header.rs`. This reduces
`op_page.rs` from 489 lines to 351 lines and keeps both files below 400 lines.

**Dropped:** Do not change the saved source schema or generation. Do not add
a command or redesign the source controls. The existing commands carry all
required source changes.

The regression test sends geometry and source commands. It checks the
rendered ComboBox and undo and redo results. Physical popup clicks are not
exercised.

**Cost to change later:** Another source choice needs a small local UI
extension and its display test.
