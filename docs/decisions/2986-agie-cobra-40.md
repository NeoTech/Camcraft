## #2986 - Agie wire EDM: Cobra Cut D 0.25 qualities for one 40 mm punch

**Decided.**
- Two new machine conditions select two new work qualities:
  `AGIE_COBRA_RA030` requests `CCD030` (Ra 0.30) and `AGIE_COBRA_RA070`
  requests `CCD070` (Ra 0.70). The name `CCD` follows `CCB` (B for Berco,
  D for Cobra Cut D). Each complete tuple (wire, Ra, options) has its own
  name, so a resident quality name cannot bind the other recipe (#2984).
- The wire is a property of the quality (`Quality::wire`). The piece and
  each work write `ID_THREAD` from it. The Berco qualities keep the bytes
  of every existing golden.
- The offsets are the measured motion values of the issue: CCD030
  250/170/145/135 µm (main cut and three trims), CCD070 245/165/140 µm
  (main cut and two trims). The clearance and miter checks use the first
  (rough) offset of the selected quality.
- Only one closed polygon punch on a 40 mm plate is qualified. The setup
  encoder refuses other heights with `SetupError::UnqualifiedCobraHeight`.
  The post refuses a die, an open cutoff, a group and a DXF arc punch. The
  1 mm separation is unchanged.
- The request keeps Te 1.50 and Tkm 1.00 with three disabled options, as
  CCB030 does. The issue gives no other Te, Tkm or option values.
- The TECCUT review asks the operator to confirm a Cobra Cut D 0.25
  technology with the number of passes of the quality, and to stop if it
  differs. No technology name is known, so none is pinned.

**Why.** The issue asks for the two measured tuples on one 40 mm punch and
for a refusal of all else until it is qualified. The wire must agree on
piece and work level, so one source (the quality) writes both.

**Dropped.**
- A wire field on the wire tool or the condition row. The condition id is
  already the recipe key; a second key could disagree with it.
- A golden SBL for the 40 mm Cobra punch. The tests derive the expected
  SBL from the Berco 40 mm output with two exact replacements, which pins
  every other byte.
- Interpolated or other heights. No measurement exists.

**Cost to change.** A new height or work type adds one table row or one
refusal arm, and its own control check.
