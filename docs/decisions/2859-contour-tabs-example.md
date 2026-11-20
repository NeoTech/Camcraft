## #2859 - Example case 2d/contour-tabs: one plate, one 2D Contour

**What was decided.** The case `examples/catalog/2d/contour-tabs` is a
90 x 50 x 6 mm plate with four 10 mm corner chamfers, in a 120 x 80 x 6 mm
sheet. One 2D Contour cuts the outside with a 6 mm flat end mill. The lead-in
is a 3 mm arc. The lead-out is a 4 mm line. There are four tabs, 8 mm wide
and 2 mm high. The stepdown is 2 mm, so the levels are Z4, Z2 and Z0.

**Why.**

- One operation keeps the program short (104 lines). A reader sees each lead
  move and each tab lift without a search.
- The outline has only straight edges. Each `G03` block is then a lead-in,
  and each `G02` block is a corner roll. The pinning test counts them.
- An arc lead-in and a linear lead-out are two different shapes. The program
  shows that the two parameters are independent.
- The sheet is as thick as the plate. The contour goes to the sheet bottom,
  and the tabs are the only material that holds the plate. This is the usual
  cut-out job.
- The stepdown equals the tab height. The tab top (Z2) is a level of the
  stepdown ladder, and no extra level is added (#2579).
- The model is a prism that the generator script writes. No outside geometry
  is used (Clean-Room Protocol).

**Considered and dropped.**

- A plate with holes and a Drill operation. The seed case `2d/first-part`
  shows Drill. A second operation hides the lead moves and the tab lifts.
- A rectangle. The chamfers give corners that are not 90 degrees and make
  the geometry original for this case.
- A stock top above the plate top with a Face operation. It adds levels and
  an operation that this case does not show.
- A lead-in written as `lead: { arc: { radius: 3.0 } }`. `add_operation`
  refuses it: the patch merges with the seeded linear lead, and the result
  has two shapes. The script writes `linear: null` with the arc. The refusal
  message does not name the cause; #2876 is the bug for it.

**Cost to change later.** Small. Change `build.camscript`, run
`cargo xtask catalog --case 2d/contour-tabs`, and update the counts in the
test `contour_tabs_case_posts_the_lead_moves_and_the_tab_lifts`.

**Pinned by.** `crates/app/tests/example_catalog.rs`:
`contour_tabs_case_posts_the_lead_moves_and_the_tab_lifts`, and the two
catalog tests that read each case.

**Review fix (PR #2877).** Keep the imported mesh embedded, and remove its
source link in both the committed project and `build.camscript`. The UI
checks a relative source path from the process directory. The example must
open when that directory differs from the case directory. The seed project
also embeds its mesh without a source link. An absolute path would tie the
example to one checkout. A product path-resolution change is outside this
example fix. The two app catalog tests pin the embedded mesh and the absent
link in the committed and rebuilt projects. The existing UI catalog test
checks that the committed project opens.
