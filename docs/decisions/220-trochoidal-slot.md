## #220 - Trochoidal slot, slice 1 (generator + standalone operation)

**Loop parameterisation: front arcs that meet the walls, not full circles.**
Decided: each loop is the FRONT arc of a circle of radius
`rho = B / cos 10deg` (B = loop band half-width, `(W - D)/2` by default),
centred `rho sin 10deg` behind its wall points, so the arc runs wall to wall
and meets each wall at 10 degrees. Why: the verifier's crossing law holds the
post on Feed moves crossing at one depth, and every textbook full-circle
trochoid (`x = B cos phi, y = B sin phi + S phi / 2pi`) crosses the loop
before it. A front arc is a graph over the band's width and the arcs of a
straight chain are translates, so they never cross for any step; the tool
moves from one arc's start to the next along the wall (cutting the cusp
between loops), so the walls come out clean. Considered and dropped: the
existing Adaptive trochoid's trim at `90deg - step/rho` (leaves a wall strip
`rho (1 - sin H)` that needs a wall lap), and loops of radius exactly B
(same strip). Cost to change later: the layout lives in one module
(`machining2d/trochoid_loops.rs`); epicycloidal / double-trochoid variants
slot in as other `front` builders.

**Ends.** The start is a helical bore of radius B about the chain's start
(the helix count is `links::helix_turns`' entry law, one turn per 2B of
drop) plus one flat circle; the far cap is a pencil of arcs through the
end's two wall points, tips stepped no further apart than the loops, so the
cap's crescent is cut at the loops' engagement rather than in one bite.

**Return.** Microlift (default 0.3 mm, the HSM planner's own lift): up at the
arc's end, the front arc retraced at the lift height at the return feed, down
at the arc's start - which is where the tool stood when the arc began, so the
landing is in air by construction and the retrace stands over the arc's own
swept air; same length as the textbook back-half return. Retract: up to the
retract plane and back down. Dropped for slice 1: the in-plane ("cutting")
return - every in-plane return either crosses the next front arc or overlaps
its neighbour's return collinearly, which the crossing law reads (or may read,
under float noise) as a crossing. Owed if the user wants it: a crossing-law
exemption for a retrace over a shared kerf, or a return routed outside the
loop band.

**How engagement is measured.** With `quality`'s dexel yardstick - the
contact-angle reading every Adaptive pin uses - run on the candidate level
before it is kept, not with a closed-form `acos(1 - S/r)` estimate. The step
ladder tries 100%, 85% ... 7% of the user's forward step; each step is first
measured on its first 12 loops (the steady state), then on the whole level.
The first step whose maximum reading is at or under the limit is kept and the
cut step is said in a note; none holding is refused by name with the least
reading, where and at which step. The adaptive planner's `Raster` was not
reused: it reads the part's walls as contact continuing the stock (right for
the adaptive's wall strips, wrong for loops riding a finished wall).

**Lifted returns on air.** `quality`'s rim reading sits 1.5 cells OUTSIDE
the cutter, so a retrace grazing the face its loop just cut reads ~15-28
degrees at zero depth. The law that a return lands on air reads INSIDE the
disc instead (`machining2d::lifted_cut`: the disc two cells in, on the dexel
cut move by move); the planner refuses by name if any lifted move would stand
over stock, and a test proves the law bites by deleting the cuts under the
returns.

**Defaults.** Forward step 1 mm in the op seed (10% of a 10 mm tool), limit
90 degrees (a straight slot reads 180), smoothing (wall-lead to arc fillet)
1 mm in the seed, clamped under 0.9 B; return feed 2x the cutting feed. The
full-width pin (D10, W16, step 1) reads 66.5 degrees against the straight
slot's 180.

**Operation.** A new appended kind (`OpType::TrochoidalSlot`, id/tag 40 (39 is the plasma profile, #670),
listed in the 2D mill section), authored like the Slot by two endpoints and a
width (the stadium region verifies it). Appended the probe's way, so no pin or
digest tag moves and - following the probe precedent (#108/#344) - no
`PROJECT_VERSION` rung: the new `trochoidal_slot` params arm is additive and a
file holding one cannot predate it. The generator itself accepts a polyline
chain and refuses by name where a turn would carry a loop out of the channel.

**Left for slice 2+.** Adaptive's "trochoidal in slots/corners" mode
(Done-means 2) and the corner-overload pin; picking the centre line from a
model chain (the op is typed endpoints today); polyline chains in the op;
the in-plane cutting return; feed reduction where theta rises; wall finishing
when the loop band is narrower than the room (said in a note today).
