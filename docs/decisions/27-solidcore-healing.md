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

**3. `HEAL_GAP_LIMIT = 1e-3` mm (one micron).** Bench reading: the port heals a
vertex-edge gap of ANY size (1e-6 up to 1.0 on a 10-box) by growing the
vertex tolerance to cover it, never by moving the vertex. A healed model
whose vertex sits 1 mm off its edges is not one a toolpath may trust, so the
seam measures the largest vertex-to-curve-end distance after healing and
refuses a gap past 1e-3 mm. The value is a CHOSEN bench limit, not a
sourced one: it passes the sub-micron gaps of the bench fixtures and sits
under any machining tolerance CamCraft uses. It is to be revisited against
real-world models (#27 remaining list, item 4). Cost to change: one
constant; making it a per-import parameter is a later step when a
production importer exists.

Edges with no 3D curve (a boolean's intersection edges can carry only UV
trims) have no curve end to measure. `vertex_gaps` COUNTS them as
`unjudged_edges` instead of passing over them. `import_smb` refuses a model
with any unjudged edge: a file's edges carry 3D curves (the port's writer
writes one per edge; all five round-trip fixtures, the L union included,
read back with zero unjudged edges), so an unjudged edge in a file is
itself a defect. The seam's `heal` on an in-memory solid still accepts
them: the port's own boolean results (the L union) carry such edges, they
were made by the port's intersector rather than read from a file, and the
integrity and closed-manifold checks have passed on them. That is the
remaining risk, recorded here: a gap on a curve-less edge of an in-memory
solid is not measured.

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

**5. Whole edits import, with a note; a future version is refused.** A
dropped final newline and a zero or negative zone tolerance leave the model
whole: the importer hands out exactly the closed-form box (volume AND
bounding box, asserted). The tolerance substitution is not silent (Prime
Directive, "log diagnostics"): `import_smb` returns `Imported { brep,
notes }`, and a declared zone tolerance the reader did not take as written
(replaced, or clamped) becomes `ImportNote::ZoneToleranceReplaced {
declared, used }`. The kernel crate carries no logging dependency, so the
note is returned for the caller to show, and the corpus pins its text
("zone tolerance 0e0 replaced by 1e-4"). A non-finite declared zone
tolerance is refused by name before the reader runs.

A file whose `Version` is newer than the port's writer version
(`DATABASE_VERSION`, 42) is now REFUSED, not noted. Reason: the reader
parses a newer layout with version-42 rules; the corpus file only read
right because its body is a version-42 body under a changed number. A real
newer file could read as a subtly different model - the silent wrong answer
the Prime Directive forbids. Dropped: accepting with a note (output right
today only by accident of the fixture).

**5a. Allocation from declared counts.** Checked in the port (solidcore
1.0.1): the reader bounds every declared record count against the bytes left
in the input before it allocates (`topology::brep_data` `checked_count`, its
module docs say so; NURBS curve and surface reads call `ensure_available`
with the declared value count before sizing the arrays). With the seam's 64
MiB cap on the input, an allocation is therefore bounded by the file size.
The corpus pins two absurd counts refused that way, quickly and without a
large allocation: face count 2^32-1 and a curve control-point count of
1e15.

**6. Writer identities.** The port's writer keeps only the surfaces whose
recorded owner equals the face's identity; a B-rep the port built names its
faces its own way, so the seam's writer uses each face's surface owner as
the face identity. Without it the writer emitted "0 Surfaces" and the read
back failed - found while building the round trip, not a port defect.

**Findings against the port (pinned, not papered over):** the raw reader
returns `Ok` with an EMPTY B-rep for a blank file, plain text, binary noise
and a header with no body (and for a NaN zone tolerance, which the seam now
refuses before the reader runs); and `Ok` with a closed box whose vertex
sits 1.0 off its edges. The importer refuses all of them. No panic and no
hang on any corpus file.
