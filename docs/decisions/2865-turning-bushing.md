## #2865 - Separate turning examples by compatible surfaces

**Decided:** Add three cases. `bushing` shows Boring and Form Turning.
`contour-finishes` shows Chamfering and Multi-Pass Turning. `taper-finish`
shows Taper Turning. Each case includes an original generator, closed
48-segment meshes, embedded project models, a build script, NC and a README.

Use a pre-bored blank to give the boring bar entry clearance. Use an already
turned cone for the taper finish ride and safe radial retraction. The
multi-pass case leaves 0.25 mm during roughing, then makes three finishing
rides to zero offset. Keep the 0.05 mm tolerance and all verification holds.

**Why:** These cases cover all five requested kinds and pass the catalog
checks. Embedded models keep the projects portable. Separate surfaces keep
the examples usable while the reported finish-envelope refusals remain open.

**Dropped:** One combined model and a two-case design with the taper on the
stepped part caused false rest refusals. The reports are #3005, #3006,
#3007 and #3008, linked on #2392. Do not change the verification algorithms
in this example PR or weaken their holds.

**Cost to change:** After those defects are fixed, combining the examples
requires changes to their generators, build scripts, generated projects,
NC files, READMEs and named catalog tests. Run the catalog checks again.
