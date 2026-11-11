## #804 - the swept radius against the safe Z before an A index

Sources (behaviour only): Tormach 770MX technical specifications (spindle
nose to table, max 13.75 in); Tormach *4th Axis Tables* comparison sheet
(02/24: microARC 4 table to centerline 3.15 in); the #785 decisions (WCS
Y0 Z0 on the A centerline, `G53 G0 Z0` before every `G0 A..`, a `G30`
moves A).

**The check lives in the program gate, as one more clause.**
- **Decided:** `posting::index_swing` adds its problems to
  `verification_report`, after the machine-limit clause. A problem blocks
  the post like every other gate problem.
- **Why:** the gate is the one place that knows the stock, the fixtures,
  the tool and the machine together. The post knows only the stream.

**Where the tool tip is: a new machine-file statement, `nose_to_axis`.**
- **Decided:** a rotary part that turns the work may state `nose_to_axis`,
  the height of the spindle nose above its centerline with Z at
  `safe_retract`. With WCS Y0 Z0 on the centerline (the #785 setup law),
  the tip at the index sits at program Z `nose_to_axis - tool length`.
  The tool length is cutter + shank + holder, the same protrusion the
  magazine check measures (a modelled tool body counts).
- **Why:** the `.mch` travel is the full stroke each way (`-L..L`, #457),
  because the file can't know the work offset. So the gate can't work out
  where the top of Z is in program terms from travel alone. With a 4th axis,
  though, the work offset is pinned to the rotary's centerline. The
  distance from the nose to that line is then a fact about the machine and
  the rotary, not about the setup.
- **The microARC 4 file states 10.6 in:** 13.75 in nose-to-table less
  3.15 in table-to-centerline, both from Tormach's spec sheets.
- **Dropped:** a setup field (the setup's "WCS Z0 below the nose"). That
  would need a schema change, a command and an MCP tool for a number that
  is fixed for every setup on this rotary.

**Which rotaries are judged.**
- **Decided:** a rotary turns the work when no `head` part hangs below it
  in the kinematics tree (`AxisLimits::turns_part`, read at parse time).
  Every posted op on a machine with a work-turning rotary about X is
  judged, whatever its own A is. A section start or a `G30` can turn A
  before any op.
- **Unaffected:** a head tilt (`tilthead.mch`'s A carries the head) and
  machines with no such rotary.
- **Known limit:** a machine embedded in a project saved before this
  change has no `turns_part`, so it reads as not judged. The only shipped
  work-turning A is the microARC file, which is new in #785 (unmerged
  until the same cut). The kit `.camproj` has been re-saved with the
  field. Reloading a machine restates it.

**Unstated `nose_to_axis`: refuse, not warn.**
- **Decided:** a work-turning A about X with no `nose_to_axis` refuses
  each op by name ("... states no nose_to_axis - the tool's clearance ...
  cannot be judged").
- **Why:** the gate has no warning channel that reaches the operator
  except the sim's single not-judged line. A clearance nobody can show is
  the silent pass the Prime Directive forbids. The fix is one number in
  the machine file.

**What sweeps, and over which X.**
- **Decided:** the swept radius is the farthest vertex, from the line
  through the WCS origin along X, of the setup's stock, the parts, and the
  tagged fixtures that turn with the work. A fixed fixture doesn't turn.
  This is judged over the whole X range.
- **Why:** the tool's X at the index is the `G30` position, which the
  program doesn't know. A vertical tool's nearest point to a line along X
  is its tip, because every point at height h is at least h from the line.
  So the tip is the binding point, and the holder's bottom, being higher,
  can never bind first.
- **Not modelled:** an unmodelled chuck and jaws are not judged. The kit
  README keeps the operator's check at the top of Z. A stated chuck radius
  in the machine file was dropped: the jaws' reach depends on the grip.

**The margin: 5 mm.**
- **Decided:** the tip must clear the swept radius by `MARGIN_MM` = 5 mm
  (about 0.2 in).
- **Why:** it covers the spec sheets' nominal figures behind
  `nose_to_axis`, tool-length measurement error and the chuck's centring.
  The kit bar (0.53 in swept at its stock box's corner) with a 3 in tool
  clears by about 180 mm.
