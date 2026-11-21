# #2022 — Wire punch from a placed DXF racetrack

Use the four source primitives of one complete DXF racetrack as the nominal
through-punch. Prove its tangent joins, positive radius, retained material,
placement and stock limits before generating the ordinary Wire Contour. Use
the intersections of adjacent circle tangents for the verification mesh.
The radial excess is bounded by 0.000025 mm. This keeps the retained circular
material inside the material checked by the verifier.

The Agie file uses the source arc centers and nominal endpoints. Posting
requires cardinal straight faces and grid-representable coordinates so the
formatted endpoints, radii and I/J centers stay consistent. Use the existing
perpendicular straight entry and capture sequence. Keep mixed drawings,
selected subchains, die openings from sketches and laser
operations for separate qualification.

Accept one DXF CIRCLE of radius 6 to 15 mm with a cardinal radial threading
hole. All diameter endpoints and incremental centers must fit the native grid.
The control plotted the same 1 mm nominal separation (angle 1/R radians) at
R6, R10 and R15. At R10, each of the four cardinal threading holes led to the
same swept circle. The main cut covers the remaining angle and all four passes
share the original center. A fixture-driven radius bound and a cardinal entry
give a direct clearance check at all intermediate radii. Retain the stock
clearance and full hole checks for each actual placement. An oblique hole
needs an additional grid and lead proof before it can be supported.
