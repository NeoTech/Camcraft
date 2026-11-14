## #1509 - 2D Adaptive: trochoidal method inside the selected region

**What was decided**

1. **The Method lives on the 2D Adaptive operation, default Spiral.**
   `OpParams::Adaptive2D` gains `method: AdaptiveMethod` (`Spiral` |
   `Trochoidal`, defined in `camcraft-motion`, `machining2d::adaptive_method`).
   Spiral is the HSM plan the kind always ran: a Spiral operation generates
   the same bytes, keeps the same stamp (the method byte is written into
   the digest only for Trochoidal) and no existing golden moved. This is
   the maintainer's decision of 2026-11-13 ("Adaptive op only"); 2D Pocket
   is not touched here (its Adaptive strategy leaves with #1517).
2. **The trochoidal method is one lane along the region's medial line.**
   The cutter-centre area is the region eroded by the tool radius and the
   stock to leave (the pocket's own law). The lane is that area's medial
   line (`adaptive::medial`, the grassfire skeleton), straightened; the
   loop band is the least clearance along it, so every cutter-centre
   point is inside the area by construction. The loops, the measured
   engagement ladder and the lifted returns are the stand-alone slot's
   planner (`trochoid_slot`, #220), not a second trochoid.
3. **Construction is the plan, a check is the law.** The finished path is
   walked move by move against the cutter-centre area (`confined`); a
   point outside refuses the whole operation by name with its place and
   a ghost. The app's own gouge laws (verify, the post's stock sim) judge
   the result as they judge every operation; nothing is exempted.
4. **A band wider than the tool gets an opened bore.** A helix of the
   band's radius `b` leaves a post of radius `b - r` at the lane's start
   when `b > r`. The method bores at half the tool radius and opens the
   bore to the band with a two-centre semicircle spiral (`trochoid_bore`),
   read by the same engagement measurement. The stand-alone slot keeps
   its plain bore byte for byte (`open: false`); its own post is filed
   as #1589.
5. **What the lane cannot do is refused by name, never approximated.**
   Region narrower than the tool; too narrow to loop in (with the
   widths); a ring round an island; a region with no single medial line;
   a lane that would leave more than 25 % of the region out of reach; a
   loop outside the region; a drop past the helix turn cap; the Adaptive
   doors (zero or oversized optimal load, non-positive stepdown, feeds).
   Under 25 % the unreached area (corners, wider parts) is said in the
   notes in mm2 and per cent.
6. **No new numbers on the page in this slice.** The optimal load is the
   most the loops advance a turn; the engagement limit is the slot's
   default (90 degrees of contact, measured); returns are a 0.3 mm
   microlift, or retract hops when the clearance plane leaves no room;
   the return feed is the cutting feed. The Entry setting is not read
   (the lane bores its own helix) and the notes say so when it is not
   Helix.
7. **The stand-alone Trochoidal slot operation stays as it is.** It is
   kept as the confined slot #1503 made it (two points and a width,
   refused when it does not fit its selection). It is not folded into
   Adaptive and no project is migrated: it cuts a slot the user places,
   which a region-driven method cannot express (a slot across open
   stock, a slot narrower than its face). Whether it should later become
   a preset of Adaptive is left to the maintainer (#1590).
8. **The project word is always written.** `adaptive2_d.method` is
   `spiral` or `trochoidal`; absent (every older file) reads as `spiral`.
   It is written even at the default so the MCP parameter patch
   `{"method": "spiral"}` round-trips (a skipped default would be reported
   as an unknown field by the patch's own check). No pinned project JSON
   for the kind existed; the format version does not move (additive).

**Why**

The maintainer asked for trochoidal motion as a way of clearing inside
the selection, on the Adaptive operation. The in-pocket trochoid already
in the HSM planner (`adaptive::troch_plan`, #5) is still held off by the
one-descent law (#47, open), so it could not be switched on. The slot
planner (#220) already holds the laws that matter here - measured
engagement, no crossing, lifted returns on air - and takes a polyline
centre line, so the smallest honest slice is to give it the region's own
medial line and band and to prove the result stays in the region.

**Considered and dropped**

- *Switching on `adaptive::troch_plan`*: blocked on #47 (every return
  lands on a loop start in cut); not this issue's to solve.
- *Parallel lanes over wide regions* (rows of loops, each lane a stadium
  inside the eroded area): covers any region, but the lane direction,
  the lane-to-lane engagement and the corner clean-up are a design of
  their own - filed as #1587.
- *Deriving the engagement limit from the optimal load*: a front arc
  into a full-width band wraps more of the cutter than a straight wall
  at the same radial bite, so the straight-wall angle of the default
  load (45 degrees) is never met; the limit stays the slot's measured
  90 degrees until it is a page parameter (#1588).
- *Leaving the rest silently*: refused; the unreached area is said, and
  past 25 % the method is refused.
- *Skipping the default when saving*: breaks the MCP patch for
  `"spiral"` (see 8).

**What it would cost to change later**

The method is one enum field with a defaulting file word; adding a
method is one variant in motion, one DTO word, one combo entry. The
lane law is confined to `adaptive_troch.rs`; replacing it by a
multi-lane plan changes no parameter. Changing the stand-alone slot's
future touches only its own kind. 3D Adaptive's Method is #1591.
