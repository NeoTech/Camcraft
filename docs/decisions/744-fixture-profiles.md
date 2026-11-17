## #744 - Editable spun/extruded fixture profiles with DXF and STEP source support

**Chosen:** Add shared ASCII DXF profile import/export and mesh generation in
`camcraft-geometry` (`dxf`, `dxf_profile`, `profile_mesh`, `profile_derive`),
command/persistence/MCP support in `camcraft-app` (`step_source`,
`fixture_profile`, `commands::profile_fixture`, `mcp::fixture_profile_tools`,
`fixture_library`, `project`), and UI dialog/row editing in `camcraft-ui`
(`state_fixture_profile`, `fixture_profile_dialog`, `fixture_profile_rows`).
Fixtures support revolved `Spun` half-profiles around `+Z` (compatible with `#743`
tool `SpunProfile` via `from_tool_spun`/`to_tool_spun`) and closed 2D `Extruded`
profiles along `X`/`Y`/`Z` with positive depth from arguments or DXF
`TEXT`/`MTEXT`/`DEPTH*` annotations. Models and fixtures imported from STEP keep
their raw STEP text in ECS `StepSource`, `.camproj`, and `.camfixtures`;
`SetFixtureProfile` refuses overwriting STEP solids, while
`DeriveFixtureProfile` records an optional reference profile without altering
STEP facets or `StepSource`. Left `egui_app/fixture_menu.rs` untouched to avoid
conflicting with open PR #1687.

**Why:** Rebuilding `Facets` on every profile edit feeds `fixture::meshes_for`,
`fixture::boxes_for`, rapid-strike detection (`fixture_struck`), and post
verification through the existing collision path with no duplicate intersection
code. Keeping raw STEP intact avoids degrading vendor STEP solids into
approximated 2D outlines while still supporting optional reference DXF export.

**Dropped:** Converting imported STEP solids into editable profiles (destroys
exact 3D vendor geometry on re-save), storing profile metadata without updating
`Facets`, or adding external DXF parsing crates (kept dependency-free per repo
policy).

**Cost to change later:** Isolated to the `dxf_profile`/`fixture_profile`
modules, five undoable commands/MCP handlers, and lenient `.camproj`/library
deserializers; multi-region or island profiles can extend `FixtureProfile` in
place while older files fall back cleanly.

### Review corrections for PR #2101

**Chosen:** Calculate a chord-error bound for revolved sections and curved
extrusions. Expand fixture collision boxes by that bound. Do not use those
approximate meshes to clear a box collision. Keep mesh refinement for exact
straight extrusions. Show the box-only collision status for approximate profiles
in the fixture row, so the UI describes the actual collision check. Refuse DXF input when conversion leaves no segments.
Preserve the closing endpoint and edge bulge for closed spun sections.
Pass each new STEP source through both UI and script re-import commands.

**Why:** An inscribed mesh can miss steel between its vertices. A conservative
box keeps the collision warning. Empty chains must return a named error.
Closed annular sections need their closing edge. Re-import must retain the
new source for library saves and preserve undo and redo behavior.

**Dropped:** Increasing the mesh resolution without a proven error bound,
clearing collision warnings with approximate facets, rejecting valid annular
sections, or retaining old STEP text after the model changes.

**Evidence:** Before and after images show the actual fixture page, profile
editor, and STEP derivation controls. The before images use the two presentation
files from this PR's parent commit with the same fixture state. The capture
harness and images stay outside the PR history. Images are stored only on the
fork's `pr-screenshots` branch and linked in the PR body.

### Reference profile collision correction for PR #2101

**Chosen:** Apply the profile chord bound only when the profile generates the
fixture facets. Use the same STEP-source distinction as `InsertFixture` for
collision boxes, mesh eligibility, and the fixture row. Keep the existing
reduced-fidelity and unknown-fidelity checks for imported STEP geometry.

**Why:** A derived reference profile leaves STEP facets unchanged. Its chord
bound does not describe those facets. Collision results must remain unchanged
through derivation, undo, redo, library save, and re-insertion.

**Dropped:** Applying a reference profile's chord bound to original STEP facets.
That choice expands unchanged steel and causes false collision warnings.

### Reference derivation only from STEP fixtures (PR #2101 review P1)

**Chosen:** `DeriveFixtureProfile` (and the MCP tool `derive_fixture_profile`)
refuses, by name, every fixture without a STEP source. A DXF/profile fixture
gets "is generated from its own profile; edit that profile". An imported mesh
fixture (STL, OBJ) gets "has no STEP source".

**Why:** Without a STEP source, the stored profile is the steel. Collision
reads its chord bound, and library re-insertion re-meshes from it. A derived
profile is a bulge-free approximation. On a generated spun cylinder it set the
chord bound to zero, and mesh refinement cleared a real hit. On an imported
mesh, library re-insertion would re-mesh the fixture from the approximation,
which can be smaller than the real steel. The GUI already offers derivation
only on STEP fixtures, so no GUI workflow is lost.

**Dropped:** A separate `reference_profile` field beside the generating
profile. It keeps derivation open for every fixture, but it changes the
project and library formats and every reader of `FixtureDef::profile`. That
is more surface than the review finding needs.

### Fixture profile domain ownership (PR #2101 review)

**Chosen:** Move the pure `FixtureProfile` module intact into `camcraft-sim`.
This crate already owns fixture collision data and depends on geometry, tools,
and serialization. App re-exports the module for document and command callers.
The public API and stored formats stay the same. No dependency is added.

**Why:** Profile meshing, chord bounds, DXF interchange, and tool conversion
operate on domain values. They do not access the document or ECS world.

**Dropped:** Placing the complete module in geometry. Its tool-profile
conversion would add a dependency from geometry to tools. The existing sim
dependencies support the complete module without that new dependency.


### Arithmetic and project-format review corrections (PR #2101)

**Chosen:** Refuse non-finite or out-of-range profile arithmetic before a
fixture command exists. Check the arc intermediates, generated mesh coordinates,
and the conservative collision envelope against the existing geometry coordinate
limit. Preserve main's v63 through-hole break-through format and assign v64 to
the cutting-data additions. Existing additive defaults migrate older files.

**Why:** Finite input values can overflow while flattening a bulged edge. A
successful command must not put NaN facets or infinite collision bounds into a
project that then cannot reopen. Main already owns v63, so retaining that number
would let its reader silently discard this PR's cutting-data additions.

**Dropped:** Accepting finite inputs without checking computed geometry,
sanitizing invalid output after modifying the document, or reusing main's
project version. Refusal preserves steel and undo/redo; distinct format versions
preserve the old-reader refusal rule.
