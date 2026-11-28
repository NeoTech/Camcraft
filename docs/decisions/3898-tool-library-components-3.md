## #3898 / #3648 - a turning tool's component stack in the tool library window

**What was decided.**

- The Component stack group on the Tool tab now shows for a lathe-held
  tool too (Turning, Groove, Thread, Form). The old hide check
  ("A lathe tool's component stack is not resolved yet (#3643)") is gone:
  PR #4278 routes a lathe stack to the lathe resolver, so the group uses
  the same rows, Add, Remove, Apply, Detach and Swap as a spindle tool.
  Apply lands one undoable `SetToolAssembly`, as before.
- The group decides "lathe" by the stored tool record's kind
  (`tool_body::LATHE_HELD`), the same test that `tool_assembly::derive`
  uses. The window's draft kind is not used: the resolver reads the record.
- An empty stack offers the roots that the stack's resolver takes:
  - a lathe tool: a part with a rectangular shank or a round bar shank
    (`RectShank`, `Cylindrical`, `Weldon`) that is not a cutting item,
    the roots of the lathe resolver's station law;
  - a spindle tool: the parts as before, less a row with a lathe body or
    a rectangular shank (the spindle resolver refuses the first; the
    second is a lathe holder).
  After the root the offer is unchanged: `component_fit::compatible_children`
  of the last holding part, so a PCLNR offers the inserts that fit its seat.
- The values row of a lathe stack shows: the gauge length (0, as #4278
  decided for a lathe), "Inscribed circle (diameter)" (the record's
  diameter, which the lathe resolver sets from the insert's IC), the nose
  radius and the orientation code (the stated or derived code; the
  default for the kind when the record keeps none). It shows no corner
  radius: the lathe resolver does not derive one. A refusal shows by name,
  as for a spindle stack ("The stack does not resolve: ...").
- The turning-tool line in the holder area now also points at the stack:
  "... import its holder in the Body section, or build the tool from
  linked parts in the Component stack."
- No built-in lathe holders or inserts were added. The tools crate has
  the ISO 5608 / ISO 1832 parsers but no lathe holder table: a holder's
  seat position (f1, l1) is a catalogue value per holder, and the brief
  says "no new numbers without a source". A user puts lathe holders and
  inserts in the document from the My library page (the user's file), a
  project file, or the component editor.

**Why.** It is the smallest change that makes the existing window build a
turning tool end to end, with the existing look and commands. The offer
filter keeps the combo box from proposing a stack that cannot resolve at
its first step.

**Dropped.**

- Disable Apply while the stack does not resolve: Apply already refuses
  through the command with its message, and a spindle stack works the
  same today. A change there touches both kinds and is not in scope.
- Built-in PCLNR / CNMG rows from the app test fixture: those seat
  frames are test values, not a sourced catalogue.

**Cost to change later.** Small: the root filter is one function
(`starts` in `state_component_views.rs`) and the lathe labels are three
constants in `stack.rs`.
