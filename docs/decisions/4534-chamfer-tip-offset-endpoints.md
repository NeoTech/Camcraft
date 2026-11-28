## #4534 - Preserve open-chain Chamfer endpoint caps with Tip offset

**The open-chain path uses three distinct endpoint/offset rides.** The entry
cap and exit cap use `corner_z` and a vertical Plunge. The interior ride uses
`tip_z` and one continuous Feed. Rapid moves join the three rides.

**Why:** The old lateral Feed changed XY and Z together. It swept the
translated cone into the part before the selected endpoint cap. The new path
changes cone planes only in the air.

**Dropped:** The old air-lead path at `tip_z` did not preserve the selected
endpoints. Later standalone short Feed islands also added unnecessary cut
transitions. Neither choice gave the endpoint and interior one clear ride.

**Compatibility:** The 0 mm Tip offset path remains unchanged. Its captured
open and closed toolpaths stay unchanged.

**Evidence:** The endpoint route and fine-grid pins pass with no gouge.
The captured zero-offset open and closed paths also pass. The fine grid
reports zero gouge cells for the 0.1 mm Tip offset reproduction.

**Tests:** The route pin passed 1/0/0 in 11.9 m. The fine-grid pin passed
1/0/0 in 7.5 s. The closed and open zero-offset captures passed 1/0/0 in
7.3 s and 7.0 s. The quick gate passed 94/94 steps and 2535/0/102 tests in
27.0 m. The three catalog suites passed 52/0/4 in 2.3 m. The light gate
passed 4/4 steps and 11/0/0 tests in 41.2 m. No golden file changed.

**Cost to change later:** A return to one combined ride must prove that every
lateral Z change stays in air for every cone angle and endpoint shape. It
must also restore the endpoint, route, and stock-simulation pins.
