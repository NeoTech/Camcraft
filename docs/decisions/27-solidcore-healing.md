## #27 - M12.1 stage 3: healing seam op and the solidcore-side importer corpus

**1. `heal` joins the seam; truck answers `Unsupported`.** The `Kernel` trait
gains `heal(solid) -> solid`. solidcore runs the port's `api_heal_brep`
(SMLib's whole heal sequence); truck has no B-rep healing in truck-modeling /
truck-shapeops 0.4, so it returns `Unsupported` by name and the bench records
it as "not yet", never as agreement. Dropped: a truck "heal" built from
mesh welding (truck-meshalgo) - it would heal the tessellation, not the
B-rep, and would make the two columns compare different things.

**2. The seam judges the heal by its result.** The port's heal reports
success whatever its healer found (SMLib's `HealBrep` ignores every status
but "unported"). So the seam refuses an empty B-rep and a B-rep whose
tolerance is not finite and > 0 up front (the port's stitch doubles its
tolerance from a tenth of the B-rep's - upstream HANG-24 never ends from
zero), and afterwards requires the integrity check, a closed manifold
solid, a positive finite measure and a vertex-edge gap under the limit
(next point).

**3. `HEAL_GAP_LIMIT = 1e-3` model units.** Bench reading: the port heals a
vertex-edge gap of ANY size (1e-6 up to 1.0 on a 10-box) by growing the
vertex tolerance to cover it, never by moving the vertex. A healed model
whose vertex sits 1 mm off its edges is not one a toolpath may trust, so the
seam measures the largest vertex-to-curve-end distance after healing and
refuses a gap past 1e-3 (one micron in mm). Why 1e-3: it passes the export round-off of
CAD files (1e-6..1e-4 typical) and stays well under any machining tolerance
CamCraft uses. Cost to change: one constant; making it a per-import
parameter is a later step when a production importer exists. Edges with no
3D curve (a boolean's intersection edges can carry only UV trims) are not
gap-checked - the integrity check has passed on them.

**4. The importer corpus uses the port's own native format (`.smb`).**
solidcore 1.0.1 reads SMLib's ASCII B-rep files (`read_from_file`), the only
format it reads without truck; STEP stays on truck-stepio as the issue
says. The corpus is made from the port's own writer output (the 10-box),
edited: blank, foreign text, noise, header only, truncations, NaN/inf
vertex, zero/negative/NaN zone tolerance, nonsense counts and indices, a
dropped face, an unhealable gap. Every case runs under a 60 s time bound on
its own thread, so a hang fails the test instead of stalling the gate.
Verdicts (raw reader and importer) are pinned in a table, like stages 1-2.
Dropped: a corpus of third-party `.smb` files - none are in-tree and the
port's upstream test files are not ours to vendor.

**5. Whole edits may import.** A future version number, a dropped final
newline, and a zero or negative zone tolerance (replaced by the reader's
default) leave the model whole; the importer may hand those out, and then
must hand out exactly the closed-form box. Everything else must be `Err`.

**6. Writer identities.** The port's writer keeps only the surfaces whose
recorded owner equals the face's identity; a B-rep the port built names its
faces its own way, so the seam's writer uses each face's surface owner as
the face identity. Without it the writer emitted "0 Surfaces" and the read
back failed - found while building the round trip, not a port defect.

**Findings against the port (pinned, not papered over):** the raw reader
returns `Ok` with an EMPTY B-rep for a blank file, plain text, binary noise,
a header with no body and a NaN zone tolerance; and `Ok` with a closed box
whose vertex sits 1.0 off its edges. The importer refuses all of them. No
panic and no hang on any corpus file.
