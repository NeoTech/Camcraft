+++
title = "Wire EDM"
description = "The Wire Contour operation: the wire EDM machine and conditions it needs, side, start hole, workpiece planes, and the refusals you are most likely to meet."
weight = 5
+++

This page is the reference for **Wire Contour**, the one kind in the
**Wire EDM** group of the [operations](@/guide/operations.md) ribbon. For
the tabs themselves and what a click picks, see that page; for what a
refusal means in general, see [Reading a refusal](@/guide/refusals.md).

Lengths are in mm (inches in an inch document). Sinker EDM has no
operation yet, so it has no section here.

## The machine, the wire and the post

**A wire EDM machine.** Wire Contour needs a setup whose machine file
declares the kind `wire_edm`. No wire EDM machine file ships, so you
write one for your machine (the keys are in
[Machine types](@/guide/machines.md)). The **Wire EDM** ribbon group
shows only when such a machine is mounted. On any other machine the
operation is refused ("a Wire Contour operation is wire EDM work; the
setup's machine is a ... - mount a wire EDM first"), and a wire EDM in
turn takes no other kind ("... - a wire EDM cuts only wire EDM
operations (a Wire Contour)").

**No library tool.** The wire is not a tool from the tool library: its
diameter and the spark gap come from the machine's EDM **condition**, the
one source for both. An operation that names a tool is refused ("a Wire
Contour operation mounts no library tool - the wire is the machine's EDM
condition; leave the tool empty (it names '...')").

**Conditions.** The machine file lists its conditions under
`edm.conditions`, each with an `id` and, for a wire cut, a `gap` (per
side) and a `wire_diameter`, taken from your machine's manual. None ship.
CamCraft reads only those two numbers; power and flushing stay with the
control, which picks them by the condition's id.

**No post yet.** No post serves a wire EDM, so a Wire Contour can be
created and generated but not posted. Posting stops with "a Wire Contour
operation posts through a wire EDM post, which is not built yet - nothing
is posted". The machine check judges only X/Y travel against the strokes
the file states; it does not judge the wire against the part, the slug or
the start hole, nor rpm, feed or tool numbers (a wire EDM has no spindle
or tool changer). Simulate does not cut the stock for a wire cut ("a wire
cut is verified by the wire laws, not the stock sim").

## Wire Contour

**For:** cutting one closed profile through the workpiece with a vertical
wire. A **die** keeps the opening (the wire runs inside the profile and
the slug falls out); a **punch** keeps the part (the wire runs outside
it). The cut starts at a start hole, leads straight to the path, goes once
round the profile counter-clockwise and leads back out the way it came.

**Picks:** the one closed profile to cut, on the Geometry tab. Without a
pick it cuts the model's through-silhouette, which must then be a single
outline. Open runs are not cut.

| Tab | Setting | Default |
|---|---|---|
| Geometry | Side | Die - the wire runs inside the profile; or Punch - the wire runs outside the profile |
| Geometry | Start hole stated | off |
| Geometry | Start hole X / Start hole Y | 0 / 0 when first ticked |
| Heights | Top of workpiece | model top |
| Heights | Bottom of workpiece | model bottom |
| Passes | Condition | (none chosen) |

There is no Lead in / Lead out tab and no Feeds tab: the lead is the
straight line from the start hole, and the control's generator sets the
cutting rate from the condition.

**Kerf.** The path is the wire centre: it stands wire radius + spark gap
off the profile, on the side you state. The side is never guessed from the
geometry. The generated path's notes give the side, the offset and its
two terms, the kerf width (twice the offset) and the start hole.

**Start hole.** Type its X and Y; it is not picked in the viewport. For a
die it is a hole in the slug, inside the profile; for a punch it is a
hole or the stock's edge outside it. Either way it must stand more than
the kerf offset off the profile.

**Heights.** One pass through the workpiece. The profile is read on the
top, and the path is programmed on the bottom plane; the bottom must
stand below the top.

**Condition.** The Condition list offers the conditions the mounted
machine states. A condition the machine no longer states stays shown as
"... (not stated by the machine)" until you pick another.

**Inside corners.** No inside corner can be cut tighter than the kerf
offset. A corner, slot or neck the wire cannot reach is refused by name,
never rounded for you; round it in the model or cut with a thinner wire
(another condition).

## Common refusals

At **Apply** or creation, about the condition:

- "a Wire Contour needs a wire EDM machine mounted on its setup - the
  machine's EDM conditions state the wire diameter and the spark gap";
- "a Wire Contour names no EDM condition - choose one of machine ...'s
  conditions (it states ...)" - pick one on the Passes tab; "it states
  none - add edm.conditions to the machine file" means the machine file
  lists none;
- "machine ... states no EDM condition \"...\" (it states ...)" - the id
  is not in the machine file;
- "machine ...: EDM condition \"...\" states no gap" (or "no
  wire_diameter") - add the missing number to that condition.

At **Generate**, about the geometry and the start hole (each begins "wire
contour:"):

- "no profile to cut";
- "the profile is open (... open run(s) picked) - this operation cuts one
  CLOSED profile; an open profile has no punch or die side";
- "... closed profiles are given - one operation cuts one profile from
  its own start hole; pick one, and give each profile its own operation";
- "the profile crosses itself - a wire path cannot cross its own kerf";
- "the profile has no area (a zero-length or collinear ring) - there is
  nothing to cut around";
- "the profile is too small for the wire - no point inside it stands the
  kerf offset (... mm) from its edge";
- "the profile narrows to a neck under the kerf width (... mm) - the wire
  cannot pass it without burning the die";
- "the profile has a slot or notch narrower than the kerf width (... mm)
  - the wire cannot enter it; widen it or use a thinner wire";
- "an inside corner or slot of the profile near X... Y... is tighter than
  the wire can cut on the ... side - the minimum inside radius is ... mm
  (wire radius ... + spark gap ...), and the kerf would leave ... mm
  standing there; round it to at least that radius, or cut with a thinner
  wire";
- "no start hole is stated - a closed profile is cut from a threading
  point (...); the wire cannot be threaded through solid material" - tick
  **Start hole stated** and give X and Y;
- "the start hole at X... Y... stands on the wire path itself - move it
  off the profile by more than the kerf offset (... mm)";
- "the start hole at X... Y... stands closer to the profile than the kerf
  offset (... mm) - the wire would burn the ... there; move it away from
  the profile";
- "the start hole at X... Y... lies outside the profile - a die is cut
  from a start hole inside it, in the slug";
- "the start hole at X... Y... lies inside the profile - a punch is cut
  from outside it; the wire would burn through the punch";
- "the workpiece's bottom (...) does not stand below its top (...)".

A message that names a place can be clicked to frame it. For other
messages, see [Troubleshooting](@/guide/troubleshooting.md).
