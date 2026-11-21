## #3210 - Wire taper: judge a meshed cone at its chord tolerance, and refuse an undercut

**Decided.**

- A constant-taper Wire Contour (`taper_deg` is not 0, no 4-axis block)
  is judged under the model's chord tolerance. The tolerance is the
  importer's chord promise (`step::CHORD_CAP`, 0.005 mm). A model that
  records a coarser chord (`model_fidelity::chord_of`) uses that chord.
  The cut does not state the tolerance. The report line and a gouge say
  "the model's chord tolerance", not "the stated verify tolerance".
- A new undercut law judges the last pass of such a cut. A move that
  cuts a drafted wall within the tolerance at one judged height, and
  stands off that wall by more than the tolerance (plus `KERF_TOL`) at
  another height, is an undercut. The undercut blocks the post by name,
  as each wire finding does.
- A move whose wall is vertical (the upright wire's clearance does not
  change with height) is not judged: a taper on a vertical wall is a
  stated relief, as in the existing #536 and #1164 fixtures.

**Why.** The generator lays the tapered wire from one height's section.
The facets at the other heights stand off the true cone by up to the
chord they were meshed at. A tolerance of 0 then refuses the true angle.
The chord is what the importer promises, so a reading inside it is not
a real gouge. The undercut law gives the "too far" reading that verify
did not give before: a taper that does not follow the wall leaves the
part oversize or the opening undersize.

**Dropped.**

- A stated verify tolerance on the taper cut, as #1170 gives the 4-axis
  cut. It needs a new parameter, a command edit, the project file and MCP.
  The chord is known from the import, so the user has nothing to type.
- An undercut law for every cut. A vertical cut on a model with a
  chamfer or a draft would then be refused. Those cuts post today.
- An undercut law for the 4-axis cut. Its tolerance is stated, and its
  default is 0. The law would refuse 4-axis cuts that post today. A
  later issue can add it with the stated tolerance.
- A non-blocking undercut note. The wire laws have no warning tier, and
  a wrong size is wrong output.
- Read the taper angle from a picked wall (the optional part of the
  issue). It is a new UI and MCP input; it is not done here.

**Cost to change.** The tolerance is one function in
`operation/wire_verify.rs` (`taper_chord`). The law is one file,
`sim/src/verify/wire/undercut.rs`. A stated taper tolerance can replace
the chord later without a project-file change for older files.
