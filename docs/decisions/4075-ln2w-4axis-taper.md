## #4075 - Sodick LN2W: a constant taper as a 4-axis G74 program

**What was decided.**

- The LN2W post writes a constant taper in the 4-axis form that the owner
  of an AQ600L uses (his input on #4075, 2026-11-25). The start order of
  #4065 does not change. After the `C` word the post writes `TP`, `TN` and
  `G74`. Then each cut is `G01 X Y U V`. `G75` comes after the last cut,
  before `T90`.
- `X Y` is the wire's point on the part's bottom. `U V` is the part's top
  minus its bottom. The stream states `WireU` `WireV` at the guide planes.
  The post moves them to the part's planes along the same straight wire:
  slope = U V / (upper guide - lower guide); bottom point = X Y + slope x
  (bottom - lower guide); U V on the part = slope x (top - bottom).
- The app sends four new PARAMs before a guide-offset leaning section:
  `WireGuideLower`, `WireGuideUpper` (the planes that U V use) and
  `WireBottom`, `WireTop` (the cut's resolved bottom and top). All are
  work Z, mm. Other wire posts ignore them.
- `TP` = the cut's bottom above the worktable. `TN` = `TP` + the cut's
  height. CamCraft knows no fixture, so the user states `TP` in a new
  declared post option, `partBottomAboveTable` (mm, default -1 = not
  stated). Unstated, each taper is refused by name. The option is set
  through the existing undoable `SetPostScriptOptions` command and its
  MCP tool, so no new command is necessary.
- The option holds one height for the program. A taper from a second
  setup is refused by name ("post the tapers of each setup as their own
  program").
- `TP` follows the section's own `WireBottom` (review of PR #4171). X Y
  are on that plane, and two Wire Contour operations of one setup can
  have different bottom heights. The option is tied to one work Z: the
  `WireBottom` of the program's first tapered cut. A later tapered cut
  gets `TP` = the option + (its `WireBottom` - the first `WireBottom`),
  and `TN` = `TP` + its height. A `TP` below the worktable is refused by
  name. Each section note names the work Z of its `TP` and the work Z
  that the option states, so the operator checks the plane and not only
  the number. Dropped: a refusal of each later taper with a different
  bottom. It is as safe, but it refuses a legal program that the post
  can write from the same numbers.
- Refused by name in a taper: an arc (the 4-axis upper-arc words are not
  confirmed), the control's wire offset G41 / G42, a cut that ends with
  the wire leaning (G75 needs the wire upright), a leaning move with only
  one of U / V, and a stream that does not state the four planes.
- The angle form (`G51` / `G52` `A`, `G50`) stays refused. The owner does
  not use it, and its corner motion is not confirmed.

**Why.** In the 4-axis form CamCraft decides each corner, and verify
checks the same surface that the machine cuts. This removes item 2 of the
issue (corner motion) for this route. Item 3 (lean side) becomes the sign
of U V, which follows from the geometry: the golden of the taper test part
shows U+3 on the -X wall and U-3 on the +X wall of a punch whose top is
3 mm smaller on each side. Item 4 (angle precision) does not apply: no
angle word is written, and U V use 0.001 mm.

**Considered and dropped.**

- A per-setup value with its own command and MCP tool (item 1 of the issue
  as written). That is a project-file field, a command, an MCP parameter
  and a UI field. The brief asked for the smallest change when the
  machine file does not state the heights (#2130). The post option gives
  the same safety for a one-setup program and refuses the multi-setup
  case by name. A per-setup value can replace it later; the post reads
  only `tableH`.
- A machine-file key (`edm.table_to_part_bottom`, removed in PR #4065):
  the review of #4065 refused it, because the fixture height changes with
  each setup.
- Mirroring the guide planes into the post's `machine` object: the stream
  already computes U V from them, so the stream states them with U V.
- 4-axis cuts with two profiles (`WireFourAxis`): the same conversion
  applies, but the pairing and arc handling are not tested on this post.
  They stay refused.

**Cost to change later.** A per-setup table height replaces the post
option read in `onOpen` with a PARAM. The PARAMs and the conversion stay.
