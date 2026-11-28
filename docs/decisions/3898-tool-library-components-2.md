## #3898 - Tool library window: built-in ER collets, stack values and swap (U7 follow-up)

Stacked on #4271 (`docs/decisions/3898-tool-library-components.md`). This
file records the choices of the follow-up slice only.

**What was decided (catalogue).**
- The built-in component catalogue (`component_library::builtins`) now
  holds metric ER collets, between the holders and the solid tools. Ids are
  `builtin:collet:<series> <bore>` (for example `builtin:collet:ER32 12`),
  names are `<series> collet <bore> mm`.
- A collet is an adaptive item: machine side the ER seat (`ER32`), tool
  side a plain bore of the nominal size (`CYL 12`). The existing fit rule
  (`ConnectionCode::takes`) then gives the wanted pairs with no new rule:
  a built-in `BT40 ER32` chuck takes every ER32 collet and no other, and
  the `ER32 collet 12 mm` takes every straight shank of 12 mm (within the
  bore slack of 0.02 mm).
- The collet geometry is one axial port at depth 0 and no body. The collet
  sits in the chuck's nose: the chuck's body (with its nut) is already the
  drawn holder, and the stick-out is measured from the chuck's port. This
  is the same model that the resolver's equivalence pin uses
  (`a_collet_chuck_assembly_is_judged_as_the_library_holder`).
- The numbers are not new: the bore range of each series comes from the
  one ER table that the holder library already has
  (`holder_library::ER_SIZES`, ISO 15488 series, DIN 6499 collet ranges and
  nuts). One collet for each whole millimetre from the series' smallest
  bore (1 mm at the least) to its largest:

  | Series | Bores (mm) | Count |
  |---|---|---|
  | ER11 | 1-7 | 7 |
  | ER16 | 1-10 | 10 |
  | ER20 | 1-13 | 13 |
  | ER25 | 1-16 | 16 |
  | ER32 | 2-20 | 19 |
  | ER40 | 3-26 | 24 |

  These are the whole-millimetre nominal bores of the common DIN 6499
  collet sets of each series (89 collets).

**Why.** Before this slice no built-in holder took a built-in solid tool:
the ER chucks need a collet and the catalogue had none, so "Show
compatible parts" on a built-in tool found nothing. A collet as its own
part is the ISO 13399 model that #3644 chose, and it keeps the fit rule
the one rule (no special case "an ER chuck takes a shank").

**Considered and dropped.**
- A clamping range on the collet (an ER collet clamps from its nominal
  bore down by 0.5 mm (ER11) or 1 mm): dropped here. It changes the fit
  law of `ConnectionCode`, which the safety floor puts at effort:high on
  its own, with its own tests. With exact bores, a 6.35 mm (1/4 in) or a
  3.175 mm shank and the odd drill shanks of the starter catalogue find no
  built-in collet. Follow-up issue #4274.
- Inch collets (1/8, 1/4, 1/2 in): dropped. DIN 6499 is metric, and the
  clamping-range follow-up covers the inch shanks better than a second set.
- Half-millimetre bores: dropped. The common sets step by 1 mm, and 89
  parts already make the "Adapters and extensions" group long.
- A "Collets" group of its own in the list: dropped for now. The UI puts
  each adaptive item that is not on a spindle taper in "Adapters and
  extensions" (`group_of`), and a collet is that. A group of its own is a
  small later change if users ask for it.

**What was decided (UI).**
- Under the stack rows the group shows three read-only labels, "Gauge
  length: … mm", "Cutting diameter: … mm" and "Corner radius: … mm", from
  `tool_assembly::derive` on the shown stack. On a draft that is not
  applied it adds "(from the stack edits; not applied)". When the stack
  does not resolve, it shows "The stack does not resolve: <the resolver's
  refusal>" in place of the values. The look stays the group's existing
  look (plain labels in a wrapped row).
- Each row has a "Swap <part>…" menu next to "Edit <part>". The menu lists
  only the parts that fit where the row's part sits
  (`stack::candidates` of the rows above it) AND with which the whole stack
  still resolves (the resolver is run on each trial stack). The row's turn
  and its stated stick-out are kept. A pick lands the whole new stack as
  one `SetToolAssembly`: one undo step, the tool record and body follow.
- The swap is offered on the applied stack only. With stack edits that are
  not applied the menu is off ("apply or undo the stack edits first"), so
  that one undo step never lands edits that the user did not apply.
- Frame cost: the "Add component" offer, the values and the swap offers
  are built in one cache that State owns (`state_component_views`, from
  the review of #4271), keyed on the component table and stacks as read
  for the document version, the tool name, the tool's record and the
  shown links. A still frame builds nothing
  (`a_still_frame_builds_no_stack_view`). The page-owned `stack_view`
  thread-local of the first version of this PR is gone.

**Not done here (open on the issues).**
- A turning tool stack: it needs the lathe resolver caller of #3643.
- MCP tools for the component commands: #3647. The swap uses the existing
  `SetToolAssembly` command, so it adds no new command.

**Cost to change later.** Low. The collets are generated from the ER
table at run time and are not written to any file (a document stores the
row that it took, with its own copy). A different bore list changes
`builtin_collets` and its pin only. The stack values and the swap are
display and one existing command.
