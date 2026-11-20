## #2739 - A basis-0 tilt saved after an angle edit by a #2599 build is a documented limit

**What was decided.** The load door does not find, mark or change a legacy
tilt (`basis_version: 0`) that a build with defect #2599 saved after an angle
edit. Such a file opens, generates and posts as it did in the build that saved
it. The limit and its manual repair go into the release notes. Nothing in the
code changes; `crates/app/tests/tilt_basis0_edited_file.rs` pins the facts.

**The facts** (A-only Tormach 770MX + microARC 4, a slot drawn from (10,0) to
(30,0), A0 B180):
- Basis 0 generates the slot along world Y at X0; basis 1 generates it along
  world X at Y0. The cut plane is turned 90 degrees about the tool axis.
- The post agrees with the preview. The basis-0 operation posts
  `G1 X0 Y-10` / `X0 Y-30`, the same lines and the same world toolpath as a
  basis-1 operation drawn at (0,-10)..(0,-30). Basis 1 posts `X10 Y0` /
  `X30 Y0`. The program cuts what the viewport shows; it does not cut what
  the plane coordinates mean in a new operation.
- No field tells this file from a legal one. A pre-#2133 project that held
  A0 B180 at basis 0 from the start (the shape of
  `crates/app/tests/fixtures/legacy-tilt-basis.camproj`, operation
  "Flat A180", behind `examples/first-4axis-part.nc`), opened and saved by a
  current build, is the same file byte for byte as one that a #2599 build
  saved after the edit A90 -> A0 B180. The file states no saving build and
  no edit history. The project version does not help: every build from v65
  (#2133) to v72 had the defect, v0.24.0 included, and the fixed build also
  writes v72.

**Why.** A load warning needs a test that is true for the edited file and
false for every legal old file. None exists: any test that fires on the
edited file also fires on the legal "Flat A180" project, whose output
`tilt_basis_compat` pins byte for byte. A warning on a legal file asks the
user to re-enter an angle, and that re-entry steps the basis and moves a
correct cut. The output of the edited file is not a hidden hazard: the
preview and the post read the same frame, so the user sees
the turned slot before posting.

**Manual repair** (release notes): open the operation, change Tool axis A
or B to a different value, then set it back to the wanted value. The edit
moves the tilt to the current basis (#2599), and the operation is drawn and
posted in the current plane coordinates. Check the preview before posting.

**Dropped.**
- Warn at load on each basis-0 tilt whose basis-0 frame differs from the
  basis-1 frame (A0 B180 and the other dust angles): it fires on the legal
  "Flat A180" file too, and a re-entry there moves a correct cut.
- Warn only on files of version 65..72: the legal file re-saved by any
  build in that range is the same file.
- Migrate every basis-0 tilt at load: the issue and #2599's decision forbid
  it; it changes the output of legal old projects.
- Add a "saved by build" or "basis set at" field from now on: it helps no
  file that already exists (a fixed build already steps the basis on each
  angle edit), and a new file field needs a project version step.

**Cost to change later.** If a later file field records the edit history,
a load note can use it. No code depends on this decision.
