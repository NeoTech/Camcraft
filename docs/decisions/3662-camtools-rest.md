## #3662 - a single-form thread mill without a pitch range, and the dovetail neck tolerance

**Decided (single-form thread mill).** A stored single-form thread mill
whose definition has no `pitch_min` key reads as a range of its one
stated pitch: `pitch_min = pitch` (`thread_mill::read_unstated_range`).
The project file and `.camtools` reader (`ToolDto::to_tool`, so also an
MCP `add_tool` definition) and the shipped catalog reader call it, only
when the key is absent. A stated `pitch_min: 0` and an in-memory record
with no range are still refused by the sanity law, so `edit_tool`
cannot silently turn a stated 0 into another number.

**Why.** A converter that reads only the tooth knows the pitch that the
tooth is ground for. It does not know the finest pitch that the tooth's
tip can cut: a tip flat wider than the root of a fine thread cuts into
its flanks. The narrowest range is the only safe reading. The thread
operation then cuts that pitch and refuses every other pitch by name
(the existing `resolve_pitch` door). The writer writes `pitch_min`
explicitly, so a file that this build writes is read by older builds
too. No `.camtools` or `.camproj` version step is needed, and nothing
is breaking: an older build refuses only a file without the key, by
the tool's name, as it did before.

**Dropped.**
- "No range" as "any pitch up to the coarsest": it widens what the
  operation accepts and can cut a wrong flank. The plan's risk list
  forbids it.
- A new optional field or a version step: the one-pitch reading needs
  neither, and it keeps the files that this build writes readable by
  older builds.
- `pitch` optional too: the core law and the operation read the pitch.
  A converter takes it from the tooth (the base width of the V).

**Decided (dovetail).** The dovetail neck check is `top < sr -
NECK_MEETS` with `NECK_MEETS = 1e-9` mm (float dust only, the same
slack as the lollipop check beside it). An inch dovetail whose flank
meets its neck exactly (0.24 in head, 0.0575 in high, 45 degrees,
1/8 in neck) lands one ulp under the neck in mm and is now accepted.
A neck 1e-6 mm wider is still refused (pinned).

**Cost to change.** One function and three call sites; the tolerance is
one constant.
