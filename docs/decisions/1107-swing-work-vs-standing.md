## #1107 - The index swing judges the swept work against what stands still

**What was decided.** The index-swing clause now has two halves. The tool
half (#804) is unchanged: the retracted tool tip against the radius the
work sweeps. The new standing half (`posting_swing_standing.rs`) judges
that same swept work (stock, parts, the fixtures that turn with it)
against:
- **the face the rotary stands on**, at the machine file's new
  `axis_to_table` figure (the rotary's centre height over its own
  mounting face; the microARC 4 states 3.15 in). The swept radius plus
  5 mm must not reach it, or the post is refused naming the op, the
  index (letter and angle), the swept radius, the figure and the
  clearance;
- **every fixed fixture of the setup whose X span overlaps the work's**,
  by the distance of its axis-aligned box from the A line in YZ. A strike
  refuses by name ("standing fixture <name>").

What it cannot judge is named in the gate's not-judged line ("index swing
did not judge the work against the standing steel: ..."), which rides
`GateReport::not_judged` to the status line, the viewer header and the
script result: a machine with no `axis_to_table`, a fixed fixture with no
(or non-finite) facets, and a fixed fixture whose box reaches across the
centerline over the work's length (a tailstock, the rotary's own body).
These post; they are not refused.

**Why.** A table, a fixture plate or a subplate under the rotary only
lifts the line with it: the nearest standing face under the rotary is
always the rotary's own centre height below the line, whatever the stack.
So the check does not need #1082's stack model, and was built on main,
not stacked on #1082. Steel that stands beside the rotary and reaches
under the work (a plate the rotary does not stand on, a riser block) is
modelled as a fixed fixture and judged by geometry. The judgement is over
the full turn, like the tool half: the program does not know the A a G30
or section start leaves it at, so every index is judged as a full swing.

**Considered and dropped.**
- *Refusing an unstated `axis_to_table`* like an unstated `nose_to_axis`:
  the issue asks for a named not-judged note, and refusing would block
  every saved microARC project and every custom table-A file the day this
  lands. The note keeps it visible without blocking.
- *Judging fixed fixtures by vertex distance:* a plate's top face spans
  under the line with its vertices far out, so vertex distance overstates
  the clearance. The box distance is exact for a plate and conservative
  (it can only refuse more) for anything else.
- *Refusing a fixed fixture that reaches across the line:* a tailstock
  centre is meant to sit on the line inside the work's sweep; a radius
  cannot tell it from a strike. Named, not refused.
- *Using the machine's derived table box* (`fixture::table_of`): that box
  is topped at the Z travel minimum, which says nothing about where the
  rotary's centerline stands.

**Left open.** #1082's subplate is covered by the law above with no extra
work (it lifts the line with the rotary). The review's other gap (a
rotary standing on a plate with no subplate gets no note on the tool
side's `nose_to_axis`) belongs to #1082's stack model, not to this check.

**Cost to change later.** One clause file and one optional machine-file
key; a richer standing model (real table extent, per-angle swings) would
replace `fixed_strike` and the face test without touching the tool half.
