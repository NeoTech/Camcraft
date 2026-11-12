## #821 - The rotary on a fixture-plate stack: a subplate on the grid, the swing judged from the raised centerline

Corrected scope (the issue's 2026-11-11 comment): the microARC subplate
does not turn with A. It is a base plate that mounts the rotary on a
fixture plate (two locating pin bores) or on the table T-slots. Sources:
behaviour only - plain stacked-height arithmetic, the #804/#818 swing law
(`804-a-swept-radius.md`, `818-swing-any-horizontal.md`), the #819 plate
model. No maker's dimension is committed.

**The subplate is a built-in accessory, not a new fixture kind.**
- **Decided:** `Accessory::RotarySubplate` (length, width, thickness,
  bore spacing, first bore from the -X end, bores from the -Y edge, bore
  diameter), inserted by `InsertAccessory`, resized by `SetAccessory`,
  saved in the existing `accessory` key (kind `rotary_subplate`), offered
  by the Fixtures page's accessory dialog and MCP `add_accessory` /
  `set_accessory` with no new code paths. Its mesh is a solid slab (bores
  drawn solid - never less steel). It lands `fixed` (it never turns), so
  it is never in the swept radius.
- **Frame:** origin on the first pin bore at the underside, +X toward the
  second - exactly what `PlaceOnPlate` does with a dowel pair, so the
  subplate is located on the #819 grid by its pins with the existing
  command.
- **Defaults are generic and named so** (10 x 6 x 3/4 in, 1/2 in bores 6 in
  apart): the dialog says to enter your own figures; the user's own
  subplate's numbers stay the user's.
- **Dropped:** a separate `InsertSubplate` / `SetSubplate` pair - it would
  duplicate the accessory path (insert, resize, file key, dialog, MCP) for
  one more slab.

**The pin law.** A rotary subplate goes only on a dowel pair whose spacing
is its bore spacing (within 0.05 mm); a single hole, or a pair at another
spacing, is refused by name at `PlaceOnPlate`, at a plate edit that
re-seats it (`SetPlate`), and at a resize of the placed subplate
(`SetAccessory`, which also keeps its footprint on the plate). Dropped:
seating it on two tapped holes - pins do not go in tapped holes.

**Who raises the rotary, and by how much.**
- **Decided:** every rotary subplate tagged in the op's setup raises the
  rotary by its thickness plus the thickness of every plate it stands on,
  followed through the plate locations (a subplate on a plate on a riser
  plate counts all three). With more than one subplate, the tallest stack
  is used - the nearest any centerline can come. A setup without a
  subplate has no stack: judged exactly as before (pinned word for word).
- **A plate alone does not raise the rotary:** nothing says the rotary
  stands on it. Placing the subplate is what states the stack.
- **Unknown thickness refuses by name:** a plate with no stated thickness
  (an imported plate - its grid detected from holes), a location naming a
  plate the document no longer holds, a looped stack, or a non-finite
  figure. Measuring an imported plate's mesh was dropped: its Z extent may
  include anything modelled on it; the user states a measured
  `nose_to_axis` instead.
- **An effective figure at or under zero refuses by name**, never judged.

**The setup's stated `nose_to_axis` wins.**
- **Decided:** `SetupDef::nose_to_axis` (command `SetSetupNoseToAxis`, MCP
  `set_setup_nose_to_axis`, a checkbox and field on the setup's Machine
  page), a finite length over zero and at most 5000 mm, used as it is in
  place of the machine's figure less the stack - also when the machine
  file states none, or the stack is unknown. A measured figure is the best
  evidence there is.
- **It rides a datum transfer** (`CreateSetup` from another setup) with the
  machine and the post (#343: the same clamping system moved).
- **File:** an additive `nose_to_axis` key on the setup row; absent - every
  earlier file - states none. A saved value at or under zero (a hand edit)
  loads as written and refuses the swing by name.

**Messages.** A judged op on a stack ends with "(nose_to_axis less the N mm
stack under the rotary: 'Sub' t mm on 'Plate' t mm)", or "(the setup's
stated nose_to_axis N mm)". The no-stack message is unchanged.

**Not done here (stays for the user):** the check on the user's own
microARC subplate (#821's "checked on the user's microARC subplate"), and
the camcraft-mcp skill's tool table (a process file; the tools describe
themselves in `tools/list`).

**Cost to change later:** the stack rule is one function
(`rotary_stack::rise`); a geometric rise (the subplate's top over the
machine table) would replace it without touching the swing clause.
