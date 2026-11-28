## #2227 - Klartext reader, third slice: RND, CHF, polar LP / CP

**What was decided.** The reader reads the contour elements of a
hand-written program that its motion model can show exactly:

- `RND R<r> [F<f>]` (rounding arc) and `CHF <l> [F<f>]` (chamfer) between
  two straight feed lines (`L` or `LP`) in the working plane.
- `LP PR.. PA.. [Z..] [R0] F../FMAX` with `IPR` / `IPA`: a straight line to
  polar coordinates about the pole `CC`.
- `CP PA.. DR±` and `CP IPA.. DR±`, with `Z` / `IZ` as a helix: an arc about
  the pole at the radius of the tool position.

`RL` / `RR`, `APPR` / `DEP`, `CTP`, `LBL` / `CALL LBL` and inch programs stay
refused by name and line.

**Why, item by item.**
- *Look-ahead, not a rewrite of moves.* A transition changes the element
  before it. The reader looks one block ahead: a line that an `RND` / `CHF`
  follows waits, and the line after the transition sets the cut-back points.
  The toolpath then gets the shortened line, the transition and the next
  line. No move that is already in the toolpath changes.
- *Geometry.* The corner point is not machined. A rounding arc of radius R
  on a turn of angle phi starts and ends R tan(phi/2) from the corner, on
  each line, with its centre on the inner side; it is tangent to both lines.
  A chamfer cuts the side length from each line (the cut-off triangle has two
  legs of that length). A transition that needs more of a line than the line
  has after an earlier cut-back is refused, as are two lines in line (no
  corner). A transition that takes exactly the whole line leaves no line:
  the reader writes no zero-length feed move for it.
- *The F of RND / CHF acts only on the transition.* Without F, the
  transition uses the modal feed. The modal feed does not change.
- *Only line-line transitions.* A transition next to an arc needs the fillet
  between a line and a circle or two circles. It is refused by name; it is a
  follow-up. A transition next to an `FMAX` line, a line with a Z change, a
  line with an M function that acts at the block end (before) or with any M
  function (after) is refused too: the order of these effects around the
  transition is not settled in our sources.
- *Polar reference.* A polar coordinate that `LP` does not write, and the
  base of `IPR` / `IPA`, is the last programmed value about the current
  pole. Our sources do not say what the control keeps as that value after a
  Cartesian move (`L`, `C`, `CR`, `CT`) or after a `CC` block that moves the
  pole: it can be the old programmed value, which is not the polar radius or
  angle of the tool position. The sources say no more for the angle than for
  the radius, so one rule covers the two. The reader keeps the last `PR` and
  `PA` only while it knows them exactly: from an `LP` (or a `CP` that writes
  `PA`, or a `CP IPA` from a known angle) about the same pole, with no X/Y
  move by another block and no move of the pole in between. A Z-only move,
  an M block or a `CC` at the same point keeps them. `CP` takes its radius
  from the tool position, so it keeps the last `PR` as it was. When the
  reader does not know the value, it refuses the `LP` by name and line
  ("LP without PR: the last polar radius about this pole is not known").
  `LP PR+0` needs no angle: its end is the pole. `CP` itself is not
  affected: the manual starts the arc at the tool position.
- *CP direction.* `CP IPA` needs the sign of `IPA` and `DR` to agree, as in
  the manual's helix table; a mismatch is refused, not guessed. `CP PA` with
  the start angle is refused (write `IPA+360` for a full circle). `|IPA|` is
  limited to the control input range 99999.9999 degrees.
- *CP pieces.* The toolpath arc model holds less than one turn per move, so
  each `CP` is split into equal pieces of at most 90 degrees on the same
  circle, with Z linear over the sweep (a helix). The motion is the same.
- *`RL` / `RR` stay refused.* The compensated path needs the tool radius of
  the tool table, which is not in the program, and the control's corner
  rules. `R0` is read. `APPR` / `DEP` always come with `RL` / `RR`, so they
  do not fit the tool-centre model and stay refused.

**Dropped.** Reading a transition next to an arc as a transition next to its
chord (wrong path); taking `CP IPA` as `|IPA|` in the direction of `DR`
(guessed sweep); the polar radius and angle of the tool position as the
value of a polar coordinate that is not written (wrong after a Cartesian
move or a moved pole, if the control keeps the programmed value); a tool-radius option to compute `RL` / `RR` (needs the
control's corner and approach rules, a separate design).

**Cost to change later.** A transition next to an arc is one new geometry
function in `klartext/corner.rs` (offset-curve intersection) and the
look-ahead accepts `C` / `CR` / `CT` / `CP`. `CTP` is one function in
`klartext/polar.rs`.
