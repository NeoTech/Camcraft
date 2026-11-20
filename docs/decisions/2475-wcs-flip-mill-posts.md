## #2475 - Each mill post carries a flipped WCS axis and an inverted machine axis

**Decided:** each mill script declares `axisInversion = true`: Mazak, Doosan
DNM, Siemens 840D, Heidenhain, Mach4, grblHAL, Tormach PathPilot, TinyG,
Carvera, and LinuxCNC. LinuxCNC is not in the issue's list (the script is
newer than the list), but the issue's "Done means" says "each mill script".
No script has a recorded reason why it cannot.

**The evidence.** `crates/post/tests/wcs_flip_dialects.rs` posts the mirrored
stream of `wcs_flip.rs` through each script (Fanuc and Haas too, to calibrate
the reader). A new reader, `tests/mirror_trace`, reads the program as a
controller whose X, Y or X and Y run reversed. It compares the physical cut
runs with the CAM path point by point at equal fractions of the run length.
It also compares the physical compensation side, and it checks that no rapid
ends over an X Y that the CAM path does not visit. The standard program read
by the reversed controller must stray, for each script and each case.

**What each dialect changes under a mirror, and how it is read.**

| Dialect | Arc words | Compensation | Drilled hole |
|---|---|---|---|
| Mazak, Doosan DNM, Mach4 | I/J/K increments, G17/G18/G19; R words with `useRadiusArcs` (not Mach4) | G41/G42 | G81 |
| Siemens 840D | I/J/K increments, G17/G18/G19 | `G41 NORM` / `G42 NORM` | G81 |
| Heidenhain | an `I J` block as the absolute centre, then planar G02/G03; XZ and YZ arcs as 1 um chords; native XY helices refused | refused by the script | G200 with an M99 call |
| Tormach, LinuxCNC, grblHAL | I/J/K increments (G91.1); R words with `useRadiusArcs` | refused by the script | G81 |
| Carvera | I/J/K increments | refused by the script | plain moves |
| TinyG | XY arcs in half turns, XZ and YZ arcs as 5 um chords | refused by the script | plain moves |

No script computes a direction, a side or a centre from anything but the
record's own mirrored fields (`CircularDirection`, `IJKIncrement`, `Start*`,
the end point, the compensation side) and the mirrored `Origin*`. That is why
the same engine pass is correct for each of them. The scripts read no other
parameter with an X or Y meaning.

**Why a new reader and not `inverted_support::controller_events`.** That
reader compares event by event. It needs one posted block for each stream
record. TinyG splits arcs, Heidenhain and TinyG write chords, Carvera and
TinyG expand the hole, and each dialect has its own first-traverse blocks. A
comparison of dense polylines is independent of these forms. The reader
panics on a G word that it does not model, so a `G91`, `G68` or mirror word
cannot pass as a trace.

**Each script also writes the two notes** that Fanuc and Haas write: the
section note `WCS X FLIPPED - PROGRAM MIRRORED` (#1649) and the header note
`MACHINE X+ MOVES LEFT - INVERTED AXIS` (#375), in the dialect's comment form.
Doosan DNM and Mach4 had the header note already.

**Kept refused:**
- Heidenhain native XY helices, with or without a WCS flip. The trace and
  golden use the planar fixture for this dialect. A separate test pins the
  named refusal. The reader rejects a Cartesian G02/G03 that changes Z.
- Probing in a flipped WCS (the maintainer's decision of 2026-11-18, engine
  level).
- Probing on an inverted machine axis through the ten scripts. They do not
  declare `probeAxisInversion`; that needs the macro interpreter proof of
  #392 for each dialect. A test pins the refusal.
- A flipped Z, a flipped and rotated WCS, tilted moves, turning, EDM and
  plasma sections (engine level, not changed).

**Refusal texts.** The hint "(Fanuc, Haas)" in the three refusals for a
script without the flag is now "(each built-in mill post does)". A lathe
script and a user's script without the line are still refused; the two
refusal tests use those.

**Dropped:**
- A golden for each of X, Y and XY for each dialect. One X golden for each
  dialect pins the full text; the trace test covers the three cases.
- A trace of tapping, peck and bore cycles on the mirrored stream. These
  cycles take the same mirrored X Y as G81; their other words (Z, R, Q, P, F)
  have no X or Y meaning. Not exercised.
- The line in `.claude/skills/camcraft-mcp/SKILL.md` that says "Fanuc and
  Haas are". The worker session could not write to `.claude/skills`. The MCP
  tool description in `wcs_flip_tools.rs` has the new text.

**Review correction:** the HEIDENHAIN TNC 640 ISO Programming User's Manual,
ID 892909-25, pages 271-272, specifies a helix in polar coordinates. The
Cartesian `G02 X Y Z` form is not a verified helix form for this control.
The post now refuses a native XY helix by name before it writes the arc.
This applies to plain sections and to flipped sections. Planar XY arcs and
the existing XZ/YZ chord paths keep their trace proof.

**Chosen:** refuse the unsupported helix. **Why:** the mirror flag must not
permit a path that the control manual does not document. **Dropped:** add
native polar helix output in this review fix. That needs its own output
implementation and control-reader proof. The present fix limits the post
to its verified forms.

**Cost to change later:** to withdraw one script, delete its
`axisInversion = true` line and its row in `POSTS`. The engine then refuses
that script by name again.
