## #1808 - Sinker electrode from a cavity with a welded (curved) wall face

**What was decided.** A welded smooth face (`FeatureSel::Patch`) picked as
a cavity wall is read as its planar strips. The app expands the patch into
its member faces (`feature::patches`). Each strip is read like a planar
pick. The motion crate's new `sinker_cavity::welded_wall` holds every strip
to the vertical-wall rule. Then `cavity_of` stands each strip on the floor
ring like any wall. A strip that leans, or that is level, refuses the whole
welded face by its name and the strip's name.

**Why.** The strips are already the exact planar walls that #1152's body
electrode reads. A click on a rounded corner of a tangent wall band gives
one patch for the whole band (the four flat walls and the sixteen corner
strips of the test pocket weld into one face). So the floor plus one welded
pick spells the same cavity as the floor plus twenty strip picks. No new
geometry is needed, and each existing refusal still applies to each strip.

**Considered and dropped.**
- A new `welded` field on `PickedFace`, with the check inside `cavity_of`.
  Dropped: it changes every constructor (fuzz, unit tests, app) for no
  gain. The check is one function beside `cavity_of` in the same module.
- De-duplication of strips by face id when a patch is picked beside its own
  strips. Dropped: a duplicated wall only covers the same floor edge twice.
  The electrode is the same (the pin checks it), and nothing reads
  `Cavity::walls` outside the unit tests.
- Without `welded_wall`, `cavity_of` alone refuses a leaning strip, but it
  reads a level strip (a blend into the floor) as a second floor. That
  message points at "different cavities", which is wrong for a fillet. The
  explicit rule gives the right reason.

**Cost to change later.** Low. A drafted or blended curved wall (a strip
that leans) needs a lofted electrode; that is a new stage of #1152, and
`welded_wall` is the one place that refuses it now.

Synthetic conditions only. Not exercised on a real EDM control.
