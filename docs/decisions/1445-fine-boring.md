## #1445 - Fine boring (G76): the shift is in the cycle choice and in the chain; only a post that declares the cycle writes it

**What was decided.**

1. Fine boring is a new cycle choice on the drilling operation,
   `CycleChoice::FineBore { shift_x_um, shift_y_um }`, and a new CL cycle
   kind, `CycleKind::FineBore`. The shift is a required part of the choice.
   There is no default shift.
2. The chain makes the shift. A hole's group is: feed in, a lateral rapid
   at the bottom (the shift), a rapid out at the shifted XY to R, a rapid
   back onto the hole axis at R. The `Cycle` and `CycleRec` records get no
   new field.
3. The post engine reads the shift from the chain and gives it to the
   script as `ShiftX` / `ShiftY`. It refuses the cycle by name for a script
   that does not declare `fineBoring = true`, and for a chain with no shift
   move.
4. Only the Haas post declares `fineBoring`. It writes
   `G76 X Y Z R I J P F`, one whole block for each hole, and never `Q`.
   Each of the other 17 shipped posts refuses the cycle by name.
5. A machine file can state `spindle.orientation: true|false`.
   `false` refuses a fine bore by name, with any post. An unstated value
   is "not limited", as for `spindle.rigid_tapping`.
6. The shift must be smaller than `(tool dia. - shank dia.) / 2`. The
   generator refuses a larger shift by name. The command refuses a zero
   shift, a shift above 5 mm on one axis, and a shift not smaller than a
   hole radius.
7. The verifier reads the three stopped-spindle rapids with the stopped
   bar. Law (d) and the solid law do not read the cutting-edge zone on
   these moves. The solid law still reads the shank and the holder at the
   shifted position.
8. The reach law holds a fine bore to the shank fit in the hole, not to
   the flute length. The hole is there already, as for a bore and a ream.

**Why.**

- #1479 asks for an explicit shift: "A guessed default could drag the tip
  across the finished bore, so no default is posted." A choice that cannot
  exist without its shift makes that a type rule. The integer micrometres
  keep `CycleChoice` `Copy` and `Eq`, and they are the lattice the chain
  uses.
- A shift held in the chain cannot disagree with the posted shift. The
  WCS flip, the WCS yaw and the axis mirror already transform the moves and
  the cycle centre together, so the posted `I` and `J` are correct in each
  frame with no new code. A new field on `Cycle` or `CycleRec` would need a
  transform in each of those places, and it would change about 50 struct
  literals in tests.
- The chain as plain moves is not safe on a control: the shift would move
  a turning tool into the finished wall. So the refusal is in the engine,
  for each post that does not declare the cycle. A future post is refused
  until it declares the flag. The author of a post does not have to
  remember the cycle.
- The Haas `G76` takes `I` and `J` as the shift along X and Y (Haas Mill
  Operator's Manual, G76 Fine Boring Canned Cycle). The `Q` form takes its
  direction from Setting 27, which a program cannot state. So the post
  never writes `Q`.
- The verifier models a tool as the cylinder that its turning edge sweeps.
  That cylinder, shifted by `s`, overlaps the bore wall by `s`, and the
  verifier refused each fine bore. With the spindle stopped, the bar
  reaches `bar radius + s` from the hole axis and the tip reaches
  `tool radius - s`. Both are inside the cylinder that the feed in cut
  while `s < tool radius - bar radius`. The feed in was read over that
  same cylinder. The shank and the holder are above the edge zone and are
  not inside that cylinder, so they keep their reading.

**Considered and dropped.**

- A `shift` field on `Cycle` and `CycleRec`, with a `SHIFT` word in the
  `.cl` and `.cnc` texts. Dropped for the reasons above.
- A separate `fine_bore_shift` field on `OpParams::Drill`. It permits a
  fine bore with no shift, and it changes about 70 test files that build a
  drill operation.
- `G76` with `Q` on the Fanuc, Mazak and Doosan posts. The direction of
  `Q` is a control parameter (Fanuc parameter No. 5101). A program cannot
  state it, so the posted shift could go into the wall. These posts refuse.
- Heidenhain `CYCL DEF 202`, Sinumerik `CYCLE86`, Mach4 `G76`. Each needs
  words that the operation does not have (a spindle angle, a disengaging
  direction code), or a word layout that this work did not read from the
  maker's manual. These posts refuse. The follow-up issue names them.
- An expanded chain with `M19`. The orientation word and its angle are
  different on each machine. #1479 asks for a refusal.
- A refusal when `spindle.orientation` is not stated. No shipped machine
  file states it, so each machine would refuse. The unstated-limit law
  already governs `rigid_tapping`; this capability uses the same law. The
  post is the second door.
- A removal of each `RapidInStock` warning of the three rapids. That hides
  a holder that the shift moves into a clamp or a wall. Only the edge zone
  is acquitted.

**What it trusts.** The tip points against the shift at the oriented stop.
This is the user's statement. No geometry in the program can check it. The
bar diameter is the tool's shank diameter. The shift is along the X and Y
of the hole list (the model frame); the post writes it in the frame of the
setup.

**Not done here.**

- No literature search ran. The session had no web access (the fetch was
  refused). The sources are the cycle formats as the worker knows them; no
  page was read in this session. A reviewer must check the Haas `G76`
  block against the Haas manual before a real cut.
- The Machines sheet has no row for `spindle.orientation`. No shipped
  machine file states it.
- Nothing ran on a real control.

**Cost to change later.** A shift field on the records can be added
without a file-format break: the `fine_bore` choice in the project file
already holds `shift_x` and `shift_y` in millimetres. A new post adds the
flag and its own word; no other code changes.
