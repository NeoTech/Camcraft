## #917 - The app reads STEP models through solidcore (part 1 of 3)

**What was decided.**
- The model import of a STEP file (`Format::Step`) goes through
  `camcraft_geometry::step::parse`. That module calls the kernel seam
  (`solidcore_step_import::import_step_model`, then `machining_mesh` for
  each solid). `camcraft-app` does not depend on `camcraft-kernel`. No new
  edge between our crates: geometry already depends on the kernel for
  Solidcraft (#1456). The time bound and the mesh-to-facets glue that the
  two kernel imports share moved to `camcraft_geometry::kernel_import`.
- The truck mesh path of the app is removed (the `step_*` helper modules of
  `crates/app/src/geometry/import/` that only the truck reader used), with
  the truck dependencies of `crates/app/Cargo.toml`.
- The public STEP entry for text in memory stays
  `camcraft_app::geometry::import::parse_model(data, Format::Step)` and
  `parse_model_graded`, with `import::step::parse_noted(data, budget)` under
  them.
- The browser build (wasm32) does not link solidcore. It refuses a STEP file
  by name: "STEP models open in the desktop app only, not in the browser
  build".
- A model over the facet budget is meshed again at a coarser chord: at most
  6 more tries, the chord times the facet ratio, clamped to x2..x16. The kept
  chord goes to `Graded::coarse_mm` as before (#864, #1054).
- **Sheets.** A sheet (a shell of a `SHELL_BASED_SURFACE_MODEL`, open by the
  file's own word) is meshed with its open boundary
  (`machining_mesh_open::sheet_mesh`) and labelled "sheet #N"; its notes
  read "STEP sheet #N: ...". A solid whose faces the mesher leaves out gives
  a partial mesh (`partial_mesh`). It lands only as an incomplete model
  with each left-out face named (the #1610 hole line, `step::hole`). It
  never lands as a whole model, and the graded loaders refuse it.
- **No STEP text reaches the partial path at the pinned solidcore commit.**
  Probes on `box.step` and the bent `fillet_corner`:

  | Probe | Result |
  |---|---|
  | one face removed | refused at read: "the outer shell is not closed" |
  | plane moved 0.5 mm or turned 90 degrees | "pcurves failed ... an edge lies 0.5 mm off the face's surface" |
  | plane tilted | reads; the mesh is refused: "face 2 stands 2.873e-1 mm off its surface" |
  | the same face twice | "face #53 is used twice" |
  | bent `fillet_corner` weights | refused at read, by face and value (face #1228, 1E300) |

  So the partial mesh is pinned at the kernel by a record edit
  (`crates/kernel/tests/machining_mesh_open.rs`), and the app pins of the
  incomplete model use a synthetic hole.
- **Closed solids ride the facets (`SolidRanges`).** The STEP import lands
  one facet range for each closed solid (`Mesh::from_solids`,
  `app::mesh_solids`). The slicer and the winding check read the solids
  from the ranges, not from a weld of the soup. The ranges are valid only at
  the model revision and facet count that they landed with. Import, update
  and undo land them. Any other landing of facets (`model::set_facets`,
  Align) steps the revision, and the soup read applies again. `MoveModel`
  sets only the transform, so the ranges stay. A file with a sheet or a
  partial mesh gets no ranges: such a mesh is not a set of solids.
- **Project file: `solids`, no version step.** The model entity has an
  optional `solids: [[start, end], ...]` (serde default, not written when
  empty). An older reader drops the key and reads the facets as one soup,
  which is what it did before. So the format version does not step. Ranges
  that do not tile the facets (out of bounds, overlap, gap, short, out of
  order, empty, one range) are dropped at load with a logged note, never
  trusted.
- **Shared-edge rule (`shared_edge::solids_touch`).** Two outward solids
  that touch or overlap along a wall use one mesh edge in 2k facets. Such an
  edge is accepted only when all of these are true:
  - the use count is even and balanced, with at least 4 uses;
  - each body (linked over plain edges only) uses the edge once each way;
  - no facet apex lies on the edge line;
  - each body has a facet at the same angle (1e-6 rad) as a facet of
    another body (a wall of one lies on a wall of the other).

  The slicer fallback (`slice_bodies::closed_solids`) and the winding check
  (`pick::winding`) use the same function. A soup that is truly
  non-manifold (one shell that meets itself, a fin, blocks that meet only
  along an edge line) is still refused by name. A model without such an
  edge gives the same result as before.
- **Face winding notes.** solidcore orients each face with its shell and
  leaves no trace. `step_face_winding` finds the faces in the file whose
  loops run against their neighbours and names them, as the earlier
  importer did: "shell #7: 1 face(s) wound against their neighbours were
  flipped: face #32 (3 of 6)".
- **Inward solids: one note.** The closed solids that are wound inward are
  turned outward. The import gives one note for all of them (#1617
  wording): "{k} closed solid(s) wound inward (negative volume) were turned
  outward: {n} facet(s) reversed".
- **Syntax locator.** A syntax refusal keeps the #912 location. Since #917
  the locator (`step_diag`) runs on every file before the kernel reads it,
  so it now skips a resource reference `<...>` (ISO 10303-21 ed. 3): a `/*`
  or a `'` in it opens nothing.
- `Graded::open` and the project field `mesh_open` stay.

**Why.** The maintainer asked to remove stepio and finish the switch to
solidcore. One reader for each format means one set of refusals and one
mesh law (watertight, outward, deviation measured). The solid ranges keep
what the reader knows (which facets close which solid) instead of a guess
from a weld of coincident facets.

**Changed pins** (old -> new, cause; the `#[test]` count of each file is
kept):
- `import_step_closed_shell.rs` (3 tests) and `import_step_open_shell.rs`
  test 1: "closed shell #1288 (10 faces) meshes open" / an incomplete model
  with its hole named -> a refusal at read, "first #1289 at read: face
  #1228: weight: 1... out of range". Cause: solidcore refuses the bent
  weight before any mesh (probe table above). The #1610 tests 2-4 do not
  change.
- `import_step_guard.rs`, `import_step_guard_default_cap.rs`: truck's
  evaluation cap ("within 100000 evaluations", "did not converge",
  "stopped after 3 faces or edges") -> the kernel's reasons: "off the
  face's surface" for the jumping face, "used twice" for the repeated face.
  The #1568 pin now uses a facet budget, and the volume bound is
  `1.2 * chord` at the kept chord (1.0194 at a 0.01 chord). Cause: the
  evaluation cap is gone with truck.
  Per assert (old reading -> new reading; cause; why not weaker):
  - `import_step_guard_default_cap.rs`: "face #53 (B-spline surface, face
    6 of 6 in shell #17)" and "did not converge" -> "first #16 at read:
    pcurves failed on 1 of 6 faces; first: face #53", "edge 0 lies 0.2500
    mm off the face's surface" and "import an STL". Cause: the kernel names
    the solid (`MANIFOLD_SOLID_BREP` #16), not the shell (#17), and it does
    not print the surface kind. Not weaker: the face, its solid and the face
    count are still asserted (restored on the final head), and the reason
    is now a measured gap (0.25 mm over the 0.01 mm cap), not a time-out.
    The surface kind ("B-spline surface") has no text in the new refusal.
  - `import_step_guard.rs`, `a_dome_over_its_facet_budget_keeps_a_coarser_mesh_at_its_own_chord`:
    volume bound `2e-3` -> `1.2 * chord` for the kept, coarser mesh; the
    `2e-3` bound stays on the full mesh at the cap. Cause: the old fallback
    kept a mesh of chord ~0.002 mm; the new fallback keeps a mesh at the
    budget's chord (0.01 mm reads 1.0194 against 1.0222). Not weaker for
    safety: the kept mesh carries its chord in `coarse_mm`, so the program
    verdict tightens by the excess or refuses (#864, `model_fidelity.rs`);
    the bound follows that chord (curved area under 1.2 mm2, each facet at
    most `chord` off its surface).
  - `import_step_guard.rs`, `a_shell_of_many_runaway_faces_is_refused_after_a_few_trips`:
    "stopped after 3 faces or edges" -> "first #16 at read: face #53 is
    used twice" (restored to the full text on the final head) and a time
    bound under 30 s. Cause: the old pass-wide trip budget stopped a mesh
    pass after 3 failed faces; the kernel refuses the repeated face when it
    reads the shell, so no face is meshed at all. Not weaker: the law (a
    hostile shell ends at once, by name) holds at zero meshed faces, and the
    time bound pins "at once".
  - `import_step_closed_shell.rs` (3 tests): "closed shell #1288 (10 faces)
    meshes open", "boundary edges lie more than", "got no mesh from the
    kernel" (the dropped face named) and "in shell #1288 could not be
    tessellated" / "returned no mesh for it" -> the one refusal at read,
    "first #1289 at read: face #1228: weight: 1... out of range". Probed on
    the final head: none of the old texts is in the new refusal, so none
    can be restored. Cause: solidcore refuses the bent weight before any
    mesh. Not weaker: the old asserts named a face that was dropped from a
    mesh; now no mesh exists, and the refusal names the same face (#1228)
    and its value. The dropped-face path itself is pinned at the kernel
    (`machining_mesh_open.rs`).
  - `import_step_open_shell.rs` test 1: the STEP-level incomplete-hole
    wording ("closed shell #1288 (10 faces) meshes open", "got no mesh
    from the kernel", `coarse_mm == UNKNOWN_MM`, the "Incomplete:" line,
    MCP `incomplete: true`, "the model would import with walls missing")
    is no longer asserted from a STEP file anywhere. Cause: no STEP text
    reaches the partial path at the pinned commit (probe table). Not
    weaker: each of those laws is still asserted in the same file, on a
    synthetic hole that lands through the same command
    (`HOLE`/`SAID` constants, `UNKNOWN_MM`, `mesh_open::hole_of`, the post
    refusal "walls missing", `verdict_tol`), and the kernel pins the
    partial mesh. The ui seam (`model_incomplete.rs`) is the same case: no
    ui test reaches "Incomplete:" from a STEP file now.
- `import_step_curve_guard.rs`, `import_step_corner_blend.rs`: the
  `with_face_eval_cap` wrapper is removed; the asserts do not change.
- `model_fidelity.rs`: a metre file was flagged at the old reader's 0.01 mm
  floor -> it meets the promise at the cap, with the measured deviation
  under the cap. Cause: solidcore meshes in millimetres; it has no floor of
  its own. The dropped assert `excess_floor(DEV_TOL, excess(0.01)) ==
  0.005` did not read the metre file, only the 0.01 mm number; it still
  holds, so it is restored as a pure pin in
  `the_round_trip_of_a_unit_scaled_cap_is_not_a_reduced_fidelity_flag`.
  Not weaker: the new metre pin asserts the measured deviation
  (`0 < deviation_mm <= CHORD_CAP`), which is the promise itself; the old
  pin asserted only that a flagged model kept a tolerance.
- `crates/geometry/tests/step_twisted_flank.rs` (2 tests, new with this
  branch): they first accepted Ok or a named Err. Now they are hard:
  `twisted_flank.step` reads Ok at 7999.857 mm3 (5132 facets), within the
  kernel twin's 0.8 mm3; `twisted_flank_off.step` is refused with "#117 at
  read: pcurves failed on" and "off the surface's range" (face #93, a
  0.89 mm shift over a 0.51 mm reach). The app twin
  `crates/app/tests/import_step_twisted_flank.rs` pins the same two
  readings through `ImportModel` (#2780).
- `feature_edge_g1_join.rs`,
  `no_pickable_edge_is_a_tangent_piece_of_a_longer_one_in_any_step_fixture`:
  known-fail #2845. Old reading: no tangent pair in any STEP fixture.
  New reading: 216 tangent pairs on `sample_ap214.step`. Cause: the
  feature graph reads the shared edges of touching solids as
  non-manifold and splits tangent pieces there (#2845). Not a safety
  loss: it changes which edge pieces a user can pick, not a toolpath
  (pins named under #2845 below). The assert is kept on its marker.
- `import_step_thread.rs`, `import_step_doubly_curved.rs`: the fixture
  shell now stands in a `SHELL_BASED_SURFACE_MODEL` (ISO 10303-42). The
  thread pin is known-fail #2844 (see below).
- `machining_mesh_parity.rs` (kernel, from #2843):
  `an_inward_solid_is_turned_outward_and_said` read a kernel note "the
  solid's faces all pointed inward ... turned outward" -> it reads the flag
  `report.turned_outward`, and asserts that the kernel adds no note. Cause:
  the STEP import gathers the flags of all solids into the one #1617 note
  (`import_step_outward`, `winding_align`, ui `import_outward` pin it), so
  a file with two inward solids gets one line, not two.
- The plain-slice tests do not change: the slicer sections the sample
  through the solid ranges.
- `bore_facet_sagitta.rs`, `step_chord_units.rs`: the 0.05 mm oversize
  bore asserted a program-gate "gouge:" problem -> it asserts `ok` false
  and the bore in `held_by` (the toolpath check holds the post). A new test
  in each file asserts the "gouge:" problem at 0.1 mm. Cause: the finer
  wall puts the 0.05 mm cut under the stock sim's 0.625 mm lattice
  (#2846, see below). The warning that holds the post is pinned too
  (`held_by_a_toolpath_gouge`): a `verify` finding of OP with kind
  `gouge`, `blocks_post` true, "move N cuts past the finish surface" (the
  generator's toolpath gouge check, not the stock sim). Not weaker for
  safety: the post is still refused at 0.05 mm, now by a named gouge
  warning.
- ui `model_incomplete.rs`: the page test imported a box STEP with a face
  removed, which landed as an incomplete model -> such a file is now
  refused at read, so the test builds the open mesh in the test (a cube
  with its top two facets removed) and lands it through the test seam
  `State::land_import_for_test` (`crates/ui/src/state_test_import.rs`),
  which calls the same `finish_import` as the import worker. The hole text
  and the "Incomplete:" line now name a test reason; the other asserts do
  not change.
- `lead_approach.rs` (2 tests) and `shoulder_adaptive.rs` (1 test): known
  fail #2849, the asserts do not change (see below). Two new pins for the
  corner check fix: motion `unreached::law::a_micron_bow_tie_cusp_gives_its_lobes_and_no_error`
  and `lead_approach::the_users_pocket_names_its_seven_sharp_corners_and_the_corner_check_runs`.
- Review fixes on PR #2904 (no assert made weaker, no `#[test]` lost):
  - `import_step_guard.rs`: renamed
    `a_face_that_may_jump_is_refused_at_a_coarser_chord_too` ->
    `a_face_that_may_jump_is_refused_by_name_under_a_small_facet_budget`.
    Cause: the kernel refuses the face at read, before the importer tries
    a coarser chord; the old name said more than the test proves. The
    body does not change.
  - `model_fidelity.rs`, `an_inch_step_and_a_metre_step_are_full_fidelity_by_measure`:
    the call of `camcraft_geometry::step::parse` (the app does not
    re-export it) -> the app's `step::parse_with_chord`; the chord and the
    facet count asserts stay. The measured deviation assert (`> 0` and
    `<= 0.005 mm`) moved to geometry `step_metre_unit.rs`
    (`a_metre_step_meshes_at_the_cap_with_a_measured_deviation`), the crate
    of the reader (test placement law). App count unchanged; geometry +1.
  - New pins: kernel lib `step_face_winding::tests` (2: the hostile edge
    order costs linear steps and keeps each parity; a cancelled pass stops
    by name), kernel `machining_mesh_open.rs` (2: an `ORIENTED_FACE` that
    names itself, a chain of 64), kernel `machining_mesh_bounds.rs` (1:
    `a_face_with_no_surface_to_measure_is_refused_by_name`, #2903 item 1).

**Review fixes, decided unattended (PR #2904).**
- **The winding check is bounded.** `face_of` reads at most 4
  `ORIENTED_FACE` wrappers, as `edge_of` reads `ORIENTED_EDGE` (and as the
  solidcore reader's `MAX_NESTING`); a deeper or self-naming chain gives
  the shell no note, and the topology reader judges it. The parity
  union-find has union by size and full path compression. The pass reads
  the import's cancel token for each shell member and stops with the
  reader's time-out refusal (stage "checking face winding"). Dropped: a fixed cap on the member count
  (the cancel token is in reach, and a cap would refuse a legal large
  shell).
- **#2780: `Refs`, not `Fixes`.** The twisted-flank reading and its refusal
  are pinned in this PR (kernel, geometry and app `step_twisted_flank.rs`).
  The speed item of #2780 ("the import does not become slower by a large
  factor") stays open and is held by #2844. Dropped: timing runs of main
  against this branch in this PR (two more builds for a number that #2844
  will change).
- **`Unmeasured::NoSurface` pin at the measure step.** The planner gives no
  face without a surface for a STEP solid that can be written by hand, so
  the pin edits the plan of `box.step` after planning (no surface, and a
  surface index past the list) and asserts the named refusal of
  `mesh_deviation::measure` and its `MeshRefusal::Unmeasured` text. The
  map in `machining_mesh::planned` (`other => MeshRefusal::Unmeasured`) is
  read, not driven by a test.

**Known regressions, tracked.**
- #2844: the four-turn thread flank gives 391338 facets (main: under 5000,
  the pin passes on main). solidcore-tess plans the strip isotropically
  and replans at half the chord ("replanned at 2.5e-3 mm chord to meet the
  requested 5e-3 mm measured tolerance"). The pin takes 93 s in debug. The
  bound stays the acceptance. Facet counts of the other helical, twisted
  and thin ruled fixtures (debug, the app import, main / this branch):

  | Fixture | main | branch |
  |---|---|---|
  | flank, 1 turn | 229 | 24200 (5.9 s) |
  | flank, 2 turns | 424 | 97506 (22.5 s) |
  | `twisted_flank.step` | 4 | 5132 |
  | `twisted_flank_off.step` | refused (boundary point off) | refused (pcurves, #117) |
  | `spline_revolved.step` | 5012 | 11500 |
  | `fillet_corner.step` | 2072 | 1866 |
  | `bspline_dome.step` | 350 | 78 |
  | `fig8.step` | 7974 | 12590 |
  | `Milling-test.step` | 952 | 6228 |
  | `sample_ap214.step` | 3178 | 11894 |

  The growth is in the thin twisted strips. Each branch mesh has its
  deviation measured within the tolerance; main's 4-facet twisted flank
  was not measured.
- #2845: the shared-edge rule cannot judge a face whose whole boundary is
  shared edges (cubes stacked face to face): that face makes its own body,
  so the rule refuses. Align refuses such a model by name (safe). The
  feature graph still splits tangent pieces on `sample_ap214` (216 pairs).
  Neither gives wrong toolpath output:
  `shared_face_bodies::the_sample_laid_flat_generates_3d_finish_and_pocket_with_no_gouge`
  and `welded_non_manifold_solids_are_refused_as_a_model_fault` prove it.
- The bore sagitta pin (cell 0.625, tol 0.01): the gouge cells at a 0.05 mm
  bore drop from 2 to 0 (0.1 mm: 10 -> 8; 0.3 mm: 34 -> 32). The ring holds
  only 8 cell centres, at r 12.507..12.509, inside the #299 slack. At 0.05
  verify still holds the post (`ok false`, `held_by`); a new "gouge:" pin
  at 0.1 mm keeps the gate's refusal. The lattice gap is #2846.
- #2849: on the new meshes of `Milling-test.step` and `fig8.step` three
  toolpath pins move. They pass on #2843's head. Three causes:
  - The corner check (fixed here). The leftover of the fine bore beside a
    stock corner kept an 11-point bow-tie cusp (area 1.2e-4 mm2, 0.3 mm
    long). `connected_pieces` read each outer loop with `Region::new`,
    which refuses a loop that crosses itself, so the check said "did not
    run". The loop is boolean output, not sketch input: `outer_fills` now
    reads it with `Region::from_slices` (the even-odd reading). A simple
    loop gives the same single path both ways, so every loop that
    `Region::new` accepted gives the same pieces as before (the four
    `unreached::law` tests and `narrow_band.rs` do not change). The check
    names the same 7 corners as on #2843's head (four stock corners, three
    wall boxes; the wall box edges move by at most 0.095 mm). Pins:
    `a_micron_bow_tie_cusp_gives_its_lobes_and_no_error` (hand-made
    bow-tie) and
    `lead_approach::the_users_pocket_names_its_seven_sharp_corners_and_the_corner_check_runs`
    (red without the fix, at the "did not run" assert).
  - The horizontal arc entry (stays). The pass start moves from
    (-31.789, 8.842) to (-31.878, 8.513), and the arc entry note changes
    from "fits only under its radius - flown at 1.500 mm" to "does not
    fit". Test 1 keeps its known-fail marker: it fails on that note.
  - `axis_reseat` (stays, own item on #2849). The first edge of the ring is
    a 0.973 mm chord 0.93 degrees off X, inside the 1 degree tolerance, so
    the start is kept; then no standard plane fits and the contour posts 0
    G18/G19 arcs (6 on #2843's head). The gap is on main too, for any start
    edge between 0 and 1 degree.
  - The fig8 EM-0800 adaptive (stays). On both heads the HSM plan is
    refused and the Offset rings cut. The ring order changed by the chain
    law of `waterline_passes`, not by a tie in a sort: the loops are listed
    by offset and area (no equal areas at k 0..2), and a chain takes the
    first loop whose chord clears. On the new mesh the face ring's start
    vertex moved, the chord from the k 2 stock ring to the k 2 face ring
    clears, and the k 1 and k 0 stock rings become the last chain. Test 3
    removes no less material: 9098.7 mm of feed against 9128.7 mm, and the
    smallest distance to the part is 4.0000 on both.
  Each fallback is stated in a note; none is silent wrong output.

**Removed truck modules (100 `#[test]`).** The truck `step_*` modules of
`crates/app/src/geometry/import/` and their tests are removed with the
truck reader. For each: the law, and the pin on the solidcore path or the
reason the law has no object (the truck step is gone, and no branch code
does it; the solidcore internals are in the external `solidcore` git
dependency). Open safety items are filed as #2903 (P7).

| Module (tests) | Law | Pin on the new path, or no object |
|---|---|---|
| `step_continuity.rs` (8) | A face is proven continuous (no jump) only with sound knots and weights; this let a coarser mesh be kept. | The proof is gone with the evaluation cap. Breaks are refused at read: kernel `step_curve_guard.rs` (`a_spline_with_a_zero_weight_is_refused_by_name`, `an_edge_spline_that_breaks_at_a_repeated_knot_is_refused_by_name`, `a_swept_surface_on_a_breaking_spline_is_refused_by_name`, `a_repeated_knot_with_no_break_still_imports`); app `import_step_curve_guard.rs`. Non-finite input: kernel `step_import.rs` (`corrupt_step_files_never_panic_or_hang`). |
| `step_division_tests.rs` (13) | Truck's curve and face division: parity with truck, a division budget, a jump refused at any tolerance. | Parity and the budget have no object (no truck division). The mesh is within tolerance by certified bounds: kernel `machining_mesh_bounds.rs` (`spline_and_revolved_step_fixture_meshes_within_tolerance`, `between_sample_sag_past_tolerance_is_refused`, `between_sample_spline_sag_past_tolerance_is_refused`), `machining_mesh.rs` (`machining_mesh_is_watertight_and_within_tolerance`). Jump: app `import_step_guard.rs` (`the_jumping_face_is_refused_by_name_at_once`). |
| `step_fit_tests.rs` (8) | The reported deviation is never under the true stand-off of the facets; a face that cannot be measured is said, never silent. | Stand-off: kernel `machining_mesh.rs` (`the_deviation_measure_sees_a_moved_node_and_a_wrong_surface`, `analytic_distances_are_the_closed_forms`) and `mesh_bound.rs` (`analytic_bounds_dominate_dense_barycentric_samples`). The unmeasured refusal (`Unmeasured::NoSurface`) has no pin: #2903 item 1. |
| `step_guard_tests.rs` (18, 1 ignored) | The evaluation meter: a runaway face trips the cap, a jumping face is refused by name, no hang. | The meter and cap have no object (the ignored heavy test measured the old cap's headroom). Read-time refusals: kernel `step_twisted_flank.rs` (`an_edge_a_quarter_millimetre_off_its_face_is_refused_by_name`, `a_twisted_flank_with_a_vertex_off_its_curves_is_refused_by_name`); app `import_step_guard.rs` (`the_jumping_face_is_refused_by_name_at_once`, `a_face_that_may_jump_is_refused_at_a_coarser_chord_too`, `a_smooth_bspline_face_still_imports_closed_at_its_volume`, `degenerate_bspline_variants_end_cleanly_and_bounded`), `import_step_guard_default_cap.rs`. |
| `step_nearest_tests.rs` (8) | A boundary point lands on its own sheet, inside the knot domain, or is refused (the #2780 class). | Truck's search has no object. Kernel, geometry and app `step_twisted_flank.rs` pin the class on two fixtures (volume 8000; refusal "#117 ... off the surface's range"). The per-sheet, root-band and fold cases have no pin: #2903 item 2. |
| `step_orient_tests.rs` (9) | Face orientation by vote: one answer in every order, one decision per shell, a cavity keeps its faces, a lone curved face keeps its winding. | Truck's vote has no object; solidcore orients each face with its shell. Kernel `machining_mesh_parity.rs` (`only_one_shell_is_ever_turned_outward`, `a_solid_with_a_void_and_its_outer_shell_inward_is_refused`, `an_inward_solid_is_turned_outward_and_said`); app `import_step_outward.rs` (`a_lone_curved_face_is_left_alone`, `an_inward_closed_box_is_turned_outward_and_said`); kernel `machining_mesh_open.rs` (`a_face_wound_against_its_neighbours_is_named_once`). Order independence has no pin: #2903 item 3. |
| `step_pass_tests.rs` (6) | A pass with many runaway faces stops after a few trips. | The pass budget has no object. The hang law is a time bound: app `import_step_guard.rs` (`a_shell_of_many_runaway_faces_is_refused_after_a_few_trips`, under 30 s, refused at read); kernel `step_import.rs` (`importer_keeps_its_size_and_time_bounds`). |
| `step_pole.rs` (6) | Truck's rescue of a nearest-point answer outside the knot domain (a pole of a corner blend). | No object: truck's answer is not used. The outcome on the corner blend: kernel `step_closed_form.rs` (`fillet_corner.step` at its closed-form volume), `machining_mesh_parity.rs` (at the app chord). |
| `step_refine.rs` (7) | A dropped finer pass keeps the coarser mesh and says its chord; a jumping face is refused. | The multi-pass meter has no object. Its successor, the facet budget: app `import_step_guard.rs` (`a_dome_over_its_facet_budget_keeps_a_coarser_mesh_at_its_own_chord`, `a_face_that_may_jump_is_refused_at_a_coarser_chord_too`), `step_fidelity.rs` (`a_model_forced_over_budget_reports_the_coarser_chord_it_kept`). |
| `step_seams_tests.rs` (15) | Face-by-face meshes close at their seams within a tolerance; a real gap or a missing face is found; flipped faces are named. | Solidcore reads closed solids and the kernel welds by side count. A missing face: kernel `machining_mesh_open.rs` (`a_face_left_out_meshes_partially_and_is_named`). A real gap: kernel `step_twisted_flank.rs` (`an_edge_a_quarter_millimetre_off_its_face_is_refused_by_name`), `heal_import.rs` (`vertex_gap_is_closed_by_the_importer`). Flipped face: `a_face_wound_against_its_neighbours_is_named_once`. A bow within the tolerance against one past it has no pin: #2903 item 4. |
| `step_stretch.rs` (2) | Truck's mesher plane is stretched to a square face. | No object and no safety law: it changed the parameter plane only; the deviation is measured on the mesh (`mesh_deviation.rs`). |

**#2850 and the corner check.** Coordinator decision on #917, 2026-11-20.
- **What was decided.** PR B keeps its own fix bc5daa32d (`outer_fills`
  in `crates/motion/src/machining2d/unreached.rs`). Its pin
  `lead_approach::the_users_pocket_names_its_seven_sharp_corners_and_the_corner_check_runs`
  needs the fix, and PR B must be green alone. Draft PR #2850 makes the
  same behaviour change inline in the same function, so the two PRs have
  one conflicting hunk. No order is forced. The PR that merges second takes
  `git merge origin/main`, keeps the `outer_fills` form, and keeps three
  pins: `a_micron_bow_tie_cusp_gives_its_lobes_and_no_error`,
  `a_chord_cusp_that_crosses_itself_does_not_stop_the_corner_check` (it
  comes with #2850; it is not on this branch) and
  the lead_approach pin above. The three `known-fail #2849` markers stay:
  their causes 2 and 3 are open on #2849.
- **Why.** Each PR stays green alone, and the merge rule keeps one form
  and all the pins.
- **Considered and dropped.** Wait for #2850 before PR B: it would hold the
  #917 stack on a draft of a different session. Remove bc5daa32d from PR
  B: its pin would be red.

**Considered and dropped.**
- A new `camcraft-app -> camcraft-kernel` edge. Dropped: the seam goes
  through geometry, and geometry already has the edge.
- Keep truck as a fallback for files that solidcore refuses. Dropped: two
  readers give two sets of answers, and the maintainer asked to remove it.
- Refuse sheets. Dropped: the old reader meshed them open, and a sheet is a
  legal surface model; it lands without solid ranges.
- An own STEP fixture for the partial path. Dropped: no STEP text reaches
  it at the pinned commit (probe table). The kernel pins it with a record
  edit.
- A radial pairing in the shared-edge rule (sort the facets about the edge
  by angle and pair each with its neighbour). Dropped: the sample's bodies
  overlap with coplanar facets, so two facets stand at the same angle and
  the radial order is not defined. The rule asks only that a wall of each
  body lies on a wall of another.
- Solids from a weld of the soup only, with no ranges. Dropped: the reader
  knows which facets close which solid; the ranges keep that answer.
- A version step of the project file for `solids`. Dropped: an older
  reader drops the key and reads the soup, as before.

**Cost to change later.** Low to medium. The STEP path is one function in
`crates/geometry/src/step.rs`; the app facade did not change its API. The
`solids` key is optional: a later format can drop it, and the soup read
stays.
