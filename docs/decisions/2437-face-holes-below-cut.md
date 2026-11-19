## #2437 - a Face does not lift over a hole that is below the cut plane

**What was decided.** Under the default `FaceHoles::Lift` law, the Face arm
of `generate` now gives the facer the outline of the material that stands
at or above the cut plane (`material_source::standing_at`). The facer
(`face_holes::beside`) bridges a hole of the region only when that material
stands in the hole or within 0.05 mm of it. A hole that no such material
touches opens below the cut plane. The row cuts straight across its column.

**Why.** A model source gives the facer the silhouette of the whole part.
A through-hole in a low flange is a hole of that silhouette at each height.
The facer lifted over it also when the cut plane was 5 mm above the hole's
rim. The lifts made rapids and plunges for no reason. Their row ends stood
within a tool radius of the open outer rim, and verify law (b) reported
each one as `cuts past the finish surface`. The 16 findings of the issue
were false: no part material stands above the cut plane there.

**What stays the same.**
- A boss in a hole is lifted over (the boss is material above the plane).
- A bore whose rim the cut plane reaches is lifted over, as M13.11 set it.
- `CutAir` and `Through` do not change.
- With no material outline (the slice refuses, or a direct `face` /
  `face_over(.., None)` call) each hole is lifted over, as before.

**Considered and dropped.**
- *Cut across each air hole under `Lift`.* That makes `Lift` equal to
  `CutAir` and changes each existing Face on a plate with holes at the cut
  plane. Dropped: larger than the issue.
- *Change verify law (b) so that an open region rim is not a wall for a
  law point.* The probe showed that a lifted row end within a tool radius
  of the open outer rim is still reported when the hole is at the cut
  plane (a plain plate, hole 5 mm from the edge, 10 mm tool). This is a
  second defect with its own cause in `sim::verify::section::tangency`. A
  change there relaxes a gouge law and needs its own issue, pins and
  review. Dropped from this PR and reported on the issue.
- *A new field on `FaceParams`.* The facer already takes `solid` beside the
  parameters. Reading it under `Lift` needs no new parameter and no
  project-file change.

**Cost to change later.** Small. The law is one match arm in
`face_holes::forbidden` and one arm in the Face arm of `generate`. Remove
the two arms to get the old behaviour.
