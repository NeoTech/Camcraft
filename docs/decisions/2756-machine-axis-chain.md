## #2756 - Machines file editor: add, remove and move the axes of a machine

The questions on #2756 (2026-11-20) are not answered yet. Each choice below
that waits on one is marked, takes the option the question recommended, and
says what it costs to change.

**What was decided**

1. **The chain is edited in the file's own JSON.** `MachineDraft` adds,
   moves and removes axis parts in `kinematics.default.parts`
   (`crates/app/src/machine/chain.rs`), like `add_coolant` and
   `remove_coolant`. Every key the editor does not know is kept, and a
   refused edit changes nothing. The validator judges the result as before.
2. **A new axis goes on the table or carries the head.** On the table it
   sits beside the table's axes, before the head: a rotary there turns the
   work. Carrying the head, it takes the head's slot and the head rides on
   it: a rotary there tilts the tool. A second rotary that carries the head
   becomes the child of the first.
3. **Removing an axis keeps what it carried.** Its `parts` take its place
   in its list, in order, and its controller row goes with it.
4. **A move trades places with the neighbour row; the tree keeps its
   shape.** The rows are the axis parts depth-first, as `parse_mch` reads
   them. Each axis keeps all its own keys (`pivot`, `offset`, `zero_offset`,
   `rollover`, `sign_convention`, unknown keys); `parts` stays with the
   slot. So a rotary that moves gets the `parent` and `chain_order` (#1962)
   of its new slot.
5. **A move trades only with an axis of the same kind that moves the same
   thing** (Q2, waiting). A rotary in a linear's slot describes another
   machine: an A that rides on Y, with X no longer riding on Y. In the same
   way, an A on the table that trades with a B that carries the head gets
   the head's slot and tilts the tool. So both must be rotary or both
   linear, and both must move the work or both the tool. A rotary turned on
   the table changes its `axis` and letter, not its slot. To put an axis
   elsewhere, the user removes it and adds it again. The refusal says so;
   the editor greys the button.
6. **An `offset` with no parent rotary gets a note, not a refusal** (Q1,
   waiting). `offset` is measured from a parent rotary (#1962). The row
   says "states offset, but no rotary carries it"; the file still
   validates, because `mirror_rotaries` writes the offset for any rotary
   that states one, and a new refusal changes what every file may hold. We
   recommend the refusal (Q1 option 2) and leave that call to the
   maintainer. Whether a child rotary may state `pivot` is not judged.
7. **A linear is reversed with `"direction": -1`** (#373). Back to
   standard removes the key: unstated is standard. A rotary is not
   reversed; the user turns its `axis` instead, and the editor refuses the
   reverse by name.
8. **A rotary's axis is ±X, ±Y, ±Z or any three numbers.** The editor
   suggests the RS-274 letter for the axis (about X is A, about Y is B,
   about Z is C) and notes a row whose letter disagrees. It does not refuse
   it: an A about Y is a legal file.
9. **The work lengths sit on a rotary that turns the work.** `nose_to_axis`
   (#804) and `axis_to_table` (#1107) are rows under "Rotaries and limits"
   for a rotary with no head below it. A rotary that carries the head shows
   them only when the file states one, so it can be cleared. The validator
   now keys these refusals by axis and key (`A nose_to_axis`,
   `work_length_refusals` in `crates/post/src/machine/swing.rs`, the
   parser's own rule), so each shows at its row instead of as a file
   refusal above the grid.
10. **The "Kinematics" group is left for #1970.** `pivot`, `offset`,
    `zero_offset`, `rollover` and `controller.rtcp` as fields are #1970's,
    in the same files (`fields.rs`, `sheet.rs`). This PR keeps those keys on
    every edit and does not draw them.
11. **The group draws in one property grid, as the editor's other
    groups** (`prop_grid` in `layout/form.rs`; #1122): one setting per
    row, the label in the first column and the control beside it in the
    second, help as a hover tooltip. Controls pushed to the right edge
    (`prop_row`) drifted away from their labels in a wide pane. Each axis has a row with its
    letter and kind and the buttons Up, Down and ✖ (their accessible names
    stay "Move A up", "Move A down", "Remove A"), then "A moves" (the work
    or the tool), then its own fields. The direction and the new axis's
    place are dropdowns. The first version wrapped the buttons and the six
    direction choices over three lines per rotary in a narrow pane.
12. **The rows are built once per edit** (the frame-cost rule, #2238; the
    review of 2026-11-24). `MachineDraft::chain_view` holds the rows, each
    row's notes and the refusal keys that show at a row
    (`crates/app/src/machine/chain_view.rs`). The draft is the file's one
    writer: `value` is private, and every write, `value_mut` included,
    drops the view. A still frame reads it back. `chain_view_builds`
    counts the real builds for the pins. The draft's `refusals()` and
    `groups()` still run on each frame, as on `main` before this PR; they
    are not this group's and are left as they are.

**Why**

The editor's law is that the file is the truth: an edit that drops a key
the editor does not know loses a vendor's data. Keeping the tree's shape on
a move means a move never changes which slot carries the head, so a move
cannot turn a table rotary into a head rotary by accident; a trade across
the head's slot is refused for the same reason (decision 5).

**Considered and dropped**

- A move that carries the axis's `parts` with it: it changes the tree's
  shape, so a move up could put the head on another slot.
- A move across kinds (the first version, `651ada63b`): valid files that
  describe another machine; dropped for decision 5.
- A refusal for `offset` with no parent: see decision 6.
- Offering `nose_to_axis` / `axis_to_table` on every rotary: the parser
  refuses them on a rotary that carries the head.
- Keeping `value` public and keying the view by a copy of the JSON
  (decision 12): each frame would still walk the whole file to compare it,
  and a write through the public field could not drop the view.

**Cost to change later**

- Decision 5: remove the kind check in `move_axis` and the button's guard;
  two tests change. The work-or-tool check stays: without it a move
  changes which axis tilts the tool.
- Decision 6: a refusal is a rule in the validator plus a test; the note
  then goes. No shipped machine file states `offset`.
- Decision 10: none here; #1970 adds its rows.
