+++
title = "Laser cutting"
description = "The Laser Profile operation: kerf, cut direction, leads, tabs, power, the laser cutter it needs, and the refusals you are most likely to meet."
weight = 8
+++

This page is the reference for **Laser Profile**, the one kind in the
**Laser** group of the [operations](@/guide/operations/_index.md) ribbon.
For the tabs themselves and what a click picks, see that page; for what a
refusal means in general, see [Reading a refusal](@/guide/reference/refusals.md).

Lengths are in mm (inches in an inch document), feeds in mm/min (in/min),
times in seconds, power in percent of the laser source's maximum.

Blue shows the operation travel; gray shows the workpiece. The schematic is not a generated NC path and has no scale.

## The machine, the tool and the post

**A laser cutter.** Laser Profile needs a setup whose machine is a laser
cutter (machine kind `laser`, see [Machines](@/guide/check-and-post/machines.md)).

**A laser head.** The tool is a **Laser head**, and its diameter is the
kerf width. The starter catalog has `LZ-KERF-020` (0.2 mm kerf). Another
tool kind is refused ("a laser profile mounts a laser head, not a ... -
its diameter is the kerf width").

**No laser post yet.** CamCraft generates and checks a Laser Profile, but
it does not post one in this release. The verification report names each
laser operation: "laser posting is unavailable: beam on/off records, power
records and pierce dwell records are not implemented". The stock
simulation does not judge laser removal either.

## Laser Profile

{{ <icon name="op-laser-profile" /> }} ![Top view of the beam cutting the hole first from a pierce inside it, then the plate outline from a pierce in the scrap.](../../../operation-paths.svg#LaserProfile)

**For:** cutting parts out of flat
sheet with a laser beam: one through-cut per contour, with the kerf offset
to the waste side, leads that start in the scrap, and holes cut before the
outline that frees the part. The cut follows the same rules as the
[Waterjet Profile](@/guide/operations/operations-waterjet.md).

**Picks:** the closed contours and open runs to cut, on the Geometry tab.
Without picks it cuts the through-silhouette of the model.

| Tab | Setting | Default |
|---|---|---|
| Geometry | Kerf width | Laser head: the laser head's diameter |
| Geometry | Compensation | In computer (offset path); or In control (G41/G42) |
| Geometry | Direction | Outer CW, holes CCW; or Reversed: outer CCW, holes CW |
| Geometry | Open run side | On the centre line |
| Geometry | Send arcs as lines | off (line tolerance 0.01 when on) |
| Heights | Clearance (beam off) / Top of sheet / Bottom of sheet | from the setup |
| Passes | Hold with tabs | off |
| Lead in / Lead out | Lead shape | Arc |
| Lead in / Lead out | Lead-in length / Lead-out length | 2 / 1 |
| Feeds | Cut feed | 600 |
| Feeds | Power | 50 % |
| Feeds | Pierce time | 0.5 |

**Kerf.** The path is the beam centre: each cutting move stands half the
kerf off the part on the waste side. One through-cut is made at the sheet
top; there are no depth steps.

**Power.** The power is a percent of the source's maximum: more than 0
and at most 100. The page names a power out of that range, and the
command refuses it.

**Feeds.** State the power and the feed for the material and the sheet
thickness from the source's cut data. The defaults are only a start.

**Common refusals:** "laser profile: no contour to cut"; "laser profile:
the power must be more than 0 and at most 100 % of the source's maximum
(it is ...)"; "laser profile: the kerf width must be a positive number
(it is ...)"; "laser profile: the sheet's bottom (...) stands above its
top (...)".
