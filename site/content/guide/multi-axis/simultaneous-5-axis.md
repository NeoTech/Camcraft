+++
title = "5-axis simultaneous"
description = "Swarf, flow, contour and roughing with the tool axis moving: lead, lag and the tilt limit."
weight = 4
+++

In simultaneous 5-axis work the part stands still and the **tool axis moves**
on every move, while the linear axes follow. CamCraft has seven operations
for it, all in the **Multi-axis** group. This page covers what they have in
common; each operation's own fields are in the
[per-operation reference](https://neotech.github.io/Camcraft/guide/operations-multiaxis/).

| Operation | Use |
|---|---|
| 5-Axis Contour | Waterline rings with the axis leaning, for shank clearance on steep walls. |
| 5-Axis Parallel | Z-following raster with the same axis law. |
| Swarf | Side-of-tool cutting on ruled walls: the tool lies along the surface's straight direction. A doubly curved or Z-only-ruled model is refused by name, pointing at another operation. |
| Advanced Swarf | The swarf path run coarse-first in up to 4 passes (**Passes**, 1..4), finishing at the nominal tolerance. |
| 5-Axis Flow | The raster turned to follow the surface's least-curving direction, so lines stay with a vane or channel. Plain ground falls back to the parallel raster. |
| Multi-Axis Rough | The 3D pocket ladder with the axis posed on steep flanks. Floors and air keep the spindle vertical. |
| Multi-Axis Contour | Machines picked edge chains where they stand: the chain is the path, so there is no stepover. |

## Tool axis control

The axis starts at the surface normal at the contact point, then:

- **Tool lead angle** leans it forward along the feed.
- **Tool lag angle** leans it sideways across the feed.
- **Max tilt** is the cone half-angle from the normal. Where the lead and lag
  would pass it, the axis folds back onto the cone's edge, never past it.
  It is 90 degrees by default. Set it to what your machine's rotaries can
  give: **the operation states the limit, not the machine**.
- **Shank avoid** (Contour, Parallel, Flow, Multi-Axis Rough and Contour):
  where the commanded pose would put the shank into the part, tilt to the
  nearest clear angle inside the cone. The cut is the same path, since the
  positions are ball-centre paths, so this is for safety, not shape.

Lead and lag are limited to -90..90 degrees and Max tilt to above 0 and up
to 180 when the operation is created.

## What it needs

- A machine that tilts in two directions: `tilthead.mch` is the only shipped
  file with two tilting rotaries. The Doosan and microARC 4 files have a
  single A table, which can reach a pose only in the plane across A; the
  machine check refuses a pose it cannot reach.
- A post that writes TCP: `fanuc.cps` or `doosan-dnm.cps`. Every other
  shipped post refuses a tilted move, so a 5-axis program does not post
  with them. There is no 5-axis post for Heidenhain, Siemens, Haas or Mazak
  yet.

## Checks

See [Checks](@/guide/multi-axis/checks.md). In short: the machine check judges
the posed moves against rotary travel, the leaning-tool reach laws judge the
shank and holder against the part and fixtures, and the stock simulation
does not judge posed programs.

## Recipe: finishing a ruled wall with Swarf

1. Mount `tilthead.mch` with `fanuc.cps` or `doosan-dnm.cps`.
2. Import a part with a ruled wall (a bladed or tapered side wall).
3. Add **Swarf** with a flat or bull-nose tool. Leave lead and lag at
   zero, set Max tilt to 90, and leave **Stepover** empty so the tolerance
   sets it.
4. Run it. If the model is doubly curved, the refusal names it; use
   **5-Axis Parallel** instead.
5. Post, then read the machine check and any warnings before cutting.

## Not implemented

- Singularity detection or avoidance: CamCraft does not model what a
  control does when a rotary passes through a pole.
- Collision of the machine's own spindle and head body against the work.
  Machine files can state a spindle body size, but no check uses it; the
  checks cover the tool, shank, holder and fixtures.
- Judging a posed program in the stock simulation.
