## #2601 - Preserve separate blind holes and their entry mouths

**What was decided.** Merge coaxial wall components only when their axial
spans touch or overlap. Test finite segments from inside the bore to its
wall ends for a blind floor. Collect triangles in each end plane and within
the fitted bore disc. Connect triangles across full shared edges with the
existing exact-bit vertex keys. Start from triangles with a bore rim edge.
Accept a segment hit on any triangle in the connected floor. Use the opposite end as
the mouth and mark the hole as blind. Keep downward Z holes in `Hole`. Store lower-mouth Z
holes in the existing directed `CrossHole` record. Report directed
findings on mills through the existing detection command.

**Why.** A shared axis does not make two blind holes one through hole.
The solid core between their floors must remain outside both hole depths.
A span check alone does not identify the entry side. Floor evidence gives
each record its own mouth and inward direction. The connected rim edge identifies
the floor as part of this bore, even when subdivision adds interior vertices.
Disconnected interior patches and outer annuli cannot supply this evidence.
An unrelated face that crosses the bore
must not turn a through hole into a blind hole. Finite segments exclude
distant model faces and the opposite hole's floor. Meshes that contain
only walls keep the existing through classification when no floor exists.

Keep the existing generation refusals for mixed lists, radial work on a
plain mill, and Z plunges in the radial generator. Detection can preserve
the findings without sending an unsupported direction to a drilling cycle.
The change adds no dependency and changes no project file format.

The pins use an original closed block and a closed D50 bar with six blind
D5 holes, each 8 mm deep. The bar runs along world Z and, after rotation,
along world Y. The rotated bar includes opposite world-Z entry mouths.
The motion fixture copies the project's original
`punched_bar` helper into the lowest crate that supplies its imports.

**Considered and dropped.** A span check without mouth classification
leaves incorrect lower-side records. Requiring the hit triangle itself to
contain a rim edge fails when a valid floor has interior vertices.
Accepting a hit without checking its connection to the rim admits unrelated
crossing faces. General arbitrary-axis recognition
and indexed drilling need a separate design. They exceed this detection
fix and would require new generation and machine checks.

**Cost to change later.** An indexed drilling implementation can consume
the directed records. It must define setup orientation, dispatch, travel,
and post behavior before it replaces the current generation refusals.
