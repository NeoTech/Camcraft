## #3023 - Threading refuses a helical modelled thread by name

**Decided.** Threading now refuses a model with a HELICAL thread, with a
message that names the helical thread, its pitch and its hand, and that
tells the user to model the thread as revolved V grooves. The refusal
comes before the slice of the job region and before the axis fit. These
two steps refused the issue's part with messages that named other causes
("the mesh is open ... dangling chord ends", "every axis fits this model
equally").

The detector (`camcraft_motion::multiaxis::axis_helix::helix`) uses the
normal lines of the facets. About a candidate axis, each inclined facet
gives the screw parameter `h = (n x u) . (o - p) / (n . u)`. A revolve
gives zero. A helical thread gives one common nonzero `h` on its two
flanks. The gates: the agreeing area is at least half of the inclined
area and 2% of the part, and it goes all the way round the axis (the
coarseness skip of `axis.rs`). The pitch is the area-weighted mean of
the agreeing band, because the two triangles of one twisted quad read
`h` low and high by the ratio of two radii (1.356 and 1.665 for a 1.5
pitch).

**Why.** The issue's "Done means" accepts a refusal that names the
helical thread. The kernel's own design already reads a modelled thread
as a revolved V section and cannot find a phase on a helix (its
silhouette is a flat crest). So the honest output today is a clear
refusal, not a toolpath.

**Dropped.**
- Read a helical thread (find its phase from the helix). This is a new
  algorithm (effort:max) and changes posted G-code. It needs its own
  issue.
- Repair the STEP tessellation of helical faces so the slice closes.
  That is an importer defect, separate from Threading; the refusal now
  comes first, so Threading does not depend on it.
- Make the axis fit accept helical facets. That changes every rotary
  and turning kind, not only Threading.
- Use the median `h` as the pitch. It lands on one side of the truth
  (about 10% low on the test shaft).

**Cost to change.** The refusal is one call in the kernel and one in the
app arm. A later helical-thread reader replaces the refusal with a
phase read and keeps the detector.

**Not tested.** The QA files of the issue (`q03-turn-shaft*.camproj`,
the STEP and `.sc` models) were not on this machine. The pins use a
procedural shaft of the same shape (M16x1.5 helical V, Ø24 body, Ø30
head, 70 long), closed and with gaps.
