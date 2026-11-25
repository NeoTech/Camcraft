## #2957 - Post a selected through die opening

Use the edge chains of the opening in the retained plate. Compare the complete picked region with the actual full-height hole on the geometry lattice. Reading only the selected loop could post a cut through a blind floor or on an unrelated face. The current scope admits one plate with one opening.

Qualify the 20 mm QN2 die cycle with nominal CCW geometry and a separate, internal threading hole. The native die work requests inward compensation. The controller owns the lead, release and trims. Use the measured maximum rough offset for the corner reach and face spacing checks. Test the inside lead against every other wall of the convex opening. An arbitrary non-convex opening needs a separate proof of the wire envelope and the lead path.

Use an original nominal polygon with fine facets for the fixture. Its straight walls provide a measurable seam. Arc fitting would change the selected geometry.

### Port onto the zero-step setup (#2984), the measured heights (#2964) and the reentrant checks (#2968)

**Decided.** Keep the setup writer of main (`agie_setup.rs`, zero erosion steps). Add `PunchSetup20::encode_die`. It writes the same zero-step setup as `encode` and changes one field: `ID_PUNCH,c_die` in place of `ID_PUNCH,c_punch`. It refuses every thickness other than 20 mm (`SetupError::UnqualifiedDieHeight`). The contour check refuses a die on another thickness before that, with a message that names the 20 mm rule. The die golden `crates/post/tests/golden/agie-setup/DIE20.SBL` is the control-checked `CCSEP20.SBL` with only the stem, the stock, the start hole and the side changed. The app test compares the CLI output of `examples/agie/die-plate.camscript` with that file.

For a die, `prepare` refuses a non-convex opening, then reverses the counter-clockwise ring to clockwise. The retained plate is then on the left and the waste on the right, as for a punch. So the existing envelope check (renamed `check_cut_envelope`) offsets toward the waste inside the opening, and its `cross < 0` reentrant test marks each corner of the opening as an inside corner of the plate. The reach check and `check_feature_spacing` use the rough offset of the selected thickness, as for a reentrant punch. The stock margin check stays the same: each nominal vertex must have the 0.4 mm swept margin to the stock edge. The ISO object is reversed again after the seam, so the control reads a counter-clockwise opening that starts at the entry point, as in the control-checked die run.

The entry code moves to `agie/contour/entry.rs`. For a die, the complete 2 mm hole must lie more than 1 mm inside each wall. The chosen wall needs 3 mm of lead and 3 mm of straight wall on each side of the foot. The foot must lie more than the swept margin (0.4 mm at 20 mm) inside every other wall. A punch keeps the supporting exterior face rule. The die margin comes from the measured rough offset, not a fixed 0.4 mm constant.

**Why.** The old branch changed `ID_PUNCH` in the retired writer (`jescript.rs`), which made erosion steps on import. The control-checked die run (PR #2969 comment, 2026-11-20) used that writer: c_die, nominal CCW ISO, hole X-5.5 Y0, inward offsets 0.245, 0.155, 0.140 and 0.134 mm, a 1 mm separation, zero job errors. Those offsets are the 20 mm row of `QN2_OFFSETS_UM`. The zero-step setup has its own control check for the punch only. A die setup that differs from it in the side field only keeps both checks as close as possible.

**Dropped.** A separate die setup writer. A die on 15, 40 or 80 mm: no control data. Non-convex openings: the straight lead and the half-plane proof need convexity. Removing the stock margin check for a die: it is conservative and keeps a thin wall between the opening and the stock edge out.

**Not yet checked.** No offline control import of the zero-step die job (`DIE20`) exists yet. The import must confirm zero steps, the c_die side and the inward offsets before machine use.

**Cost to change.** Another die thickness needs a control run on that thickness, then a change of `DIE_HEIGHT_UM` to a list. A non-convex opening needs a lead path check against each wall and a proof for the reach of each inside corner.
