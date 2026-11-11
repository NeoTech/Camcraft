## #801 - plasma laws: hole slugs, the reverse crossing, inch and G41/G42 at the gate

Follow-ups from the three reviews of PR #791 (#672 step 2). Sources as
there: Tormach *1300PL Operator's Manual* UM10720 §1.4.2 (a dropped or
tipped slug), §4.12.8; the LinuxCNC Plasma Primer §12-13. No vendor CAM
output or implementation read.

**1. A hole's slug is a freed area for tip-up, every size.** Step 1
(#675) judged a rapid against earlier OUTER rings only. A hole's cut
frees its slug just as an outline frees its part, and a slug that does
not fall clear rests on the slats where it can tip into the torch. Now a
rapid is judged against every earlier closed ring; a hit on a hole's
ring is `TipUp { slug: true }`, read "crosses the slug of hole C<n>".
*No slug-size floor.* Whether a slug drops depends on the table's slat
pitch and how the slug lands, which the program does not know; the
finding is a warning, so its only cost is a line on the report.
*Considered and dropped:* exempting the `small` holes - the small-hole
rule's threshold (32 mm by default, or 1.5x the plate) is about pierce
and feed quality, not slat pitch, and a 30 mm disc rests on any common
slat spacing; a kerf-width floor - a hole under the kerf is already
refused at generation (`ring.rs`), so it would exempt nothing.
*The departure.* A hole's lead-out ends inside its own slug (the scrap
side, as every hole is cut), so the rapid out of the hole just cut
always starts over that slug. When the rapid starts inside or on the
ring of the hole cut immediately before it, its first stretch (up to its
first contact with the ring) is the departure and is not judged against
that hole; any later stretch back over the same slug is (an L-shaped
hole, pinned). A part's outline gets no such exemption - its lead-out
ends on the scrap side, outside the part, so a rapid starting inside a
freed part stays a tip-up (#791, pinned).

**2. The reverse inside-part crossing is refused.** The judge checked a
later ring against earlier ones, so L cut first and then E around it
(L's x = 25 edge already through E's material) was only the kerf
warning. Now, when the later cut is an OUTER contour, an earlier ring
with a stretch strictly inside it that it does not strictly enclose is
a `CutCrossing` too (the same `rings_cross`, the other way round). The
INNER case is left alone: a part nested in a hole and touching the
hole's wall, cut before the hole, is common-line cutting on a scrap
slug and stays legal. The third review's nesting probes are the
regression set (part, hole, outline; with the part's own hole first;
touching, cut first; part after its hole; shared edge and corner, both
orders).

**3. Inch and G41/G42 at the gate.** Pinned in `plasma_gate`: an inch
document reads clean and a cut crossing blocks the post with its place
in inches (and in mm in a mm document); a control-compensated program is
judged on its torch-centre moves - the same rings as the computer-offset
program, not the programmed part contour its comp spans carry - and the
same cuts laid outline-first are refused.

**4. The diamond-hole pin was vacuous.** A hole touching its outline at
a vertex is not strictly enclosed, so the record's PARENT is refused as
malformed before any law runs; the assert held whatever the law did.
Replaced by the nested part touching its hole's wall, cut first, which
reaches the laws and must be the kerf warning only.

**5. Decision 4a** (`672-plasma-laws-2.md`) now says plainly that the
2 r cap refuses a normal-sized inside V sharper than about 30°; pinned
by a 20 mm, 25° V-notch refused.

**6. A zero-width slit is refused at the contour door, not dropped.**
`Region::new` keeps the AREA of a loop, and an out-and-back spike has
none, so a picked contour with a zero-width slit came out as the plain
outline and generated without it. That is right for a region (milling's
pockets and offsets ask for area, and the spike encloses none), so
`Region::new` is unchanged and no geo2d issue is filed. It is wrong for
a plasma contour, which is a cut path: `plasma::nest::contours` now
refuses, by name and place, a picked vertex the normalised ring no
longer reaches within 1 µm-lattice slack (1e-3 mm) - "pick the slit as
an open run". A collinear vertex the region drops lies on an edge and
stays legal (pinned).
