## #3752 - Path kernel K6: the 2D Pocket recipe behind the planner switch

The first recipe on the path kernel (K6 of #4023). Sub-PR of
`side/path-kernel`. With the switch at its default (`old`) the output does
not change: the old Pocket arm runs byte for byte. Under
`CAMCRAFT_PLANNER=new` the 2D Pocket runs on the kernel.

**What was decided**

1. **Where the code is.** The recipe and the level loop are in
   `camcraft-motion` (`machining2d/kernel_pocket.rs`, `kernel_level.rs`):
   motion calls the kernel; the kernel still depends on geometry only.
   `plan_level` runs the stages in the record's order (guide, order, load,
   then link and entry for each cut, the exit) and is not specific to the
   pocket, so the next recipes use it. The app arm only chooses the route
   and refuses the options that the recipe does not plan
   (`operation/pocket_kernel.rs`).
2. **The switch is read in one place.** `planner_engine()` is still the
   one reader of the process setting. `run_job` passes its value to
   `run_job_on(job, engine)`, which gives it to `check_engine` and to the
   arms. `KERNEL_KINDS = [Pocket]` lets the 2D Pocket through under
   `new`; each other phase-1 kind is still refused by name. Tests pass the
   engine directly: `operation::generate_on`, `run_job_on` and
   `GenerateOperation::new_on` (the #4055 test hook).
3. **The strategies.**
   - Waterline: rings of the boundary (`RingRef::Boundary`, first offset
     0) at the stepover; load `None` (each read above the stated limit
     stepover / D is named on the status line).
   - Offset rings: rings at the load step (`tiling_step`, as the old
     planner spaces them); `ConstantEngagement` at the optimal load with
     `Reach::Rings`; the stated limit is the load step / D.
   - Zigzag: refused by name under `new` (`KERNEL_POCKET_ZIGZAG`). The
     record maps the Zigzag row to #3753. A trial of the kernel's rows
     (`RowsChoice(0, stepover, true)`, the rows with a closing ring) gave
     verifier `Crossing` findings that the old planner does not have: 30
     on a 40 mm square pocket and 48 with a round island (old: 0). Each
     row ends on the closing ring, and the ring cuts across the row ends.
     Dropped: posting those rows under `new` with the crossings named (a
     path that crosses its own cuts is the class of defect that #3752
     removes from the rings). The rows go with #3753.
   - Auto: the old rule (`PocketStrategy::resolve` on the outermost loop):
     Waterline for a round outline, else Zigzag. So under `new` an Auto
     pocket with an outline that is not round is refused by name until
     #3753 (the text says that Auto chose Zigzag).
   - Adaptive: refused by name (`KERNEL_POCKET_ADAPTIVE`). The old arm
     refuses it too (`POCKET_ADAPTIVE_RETIRED`, the 2D Pocket retired it),
     and the load stage refuses `Reach::Stock` on purpose (the HSM
     planner). Dropped: the old planner under `new` (law 4: no silent
     fallback). Under `new` the refusal is the status line text of the op.
4. **Out of scope, refused by name under `new`.** Rest machining (#3753),
   "do not touch" areas (#3754) and the corner relief. Each is refused
   before the kernel runs (`pocket_kernel.rs`), with "unset
   CAMCRAFT_PLANNER" in the text. 3D levels are the 3D Pocket kind, which
   `check_engine` still refuses under `new`.
5. **The entry.** The op's entry maps to the entry stage's asked kind:
   Helix and Ramp are the entries into material that the user selected
   (the helix angle: the op's angle, else the angle of its pitch, else the
   old law of one turn per helix diameter of drop, 17.7 degrees; the ramp
   angle: the op's, else the ladder's). Arc asks for an arc lead from the
   air. Plunge asks for an entry outside material: an arc lead in the air,
   a pre-drilled hole or a cleared column. A closed pocket with none of
   them is refused with `NO_SAFE_ENTRY` and its fix text ("pre-drill a
   hole at the start of the cut, or select a helix or ramp entry").
   Dropped: a straight plunge into solid, the old default. The holes that
   the app proved open are the entry choice's holes, each with the tool
   radius, only at the levels down to its floor. The level transition
   (`links.transition`) is the entry of the levels after the first.
6. **The link policy.** The pass link `Clearance` is the user's retract
   between cuts (`LinkPolicy::Retract`, each lift named `USER_RETRACT`).
   `Retract` (the default of `Links::PLANAR`) and `KeepDown` are
   `StayWhereProved`: the rings link at depth where the link stage proves
   it, and lift to the clearance height with a named reason elsewhere.
   This is the point of #3752. Dropped: `Retract` as a lift on each ring
   (it would keep the lifts that the issue removes). A keep-down length
   limit (`max_stay_down`) is not planned by the link stage yet: the path
   says so by name ("instead of the limit").
7. **Each level is planned with its own heights.** The entry heights of a
   level are along the normal above that level: the material top (the
   highest of Top Z, the stock top and the carried material top) less the
   level Z. The descent of each level starts above the stock top, because
   the converter accepts no approach under the stock top (#4269 decision
   3). So the helix of a deeper level descends through the levels already
   cut, at the helix pitch. That costs time in air, not safety. Dropped: a
   per-level material top (it needs the converter to accept an approach
   under the stock top where an upper level cleared the column: a
   converter change, follow-up). The stock reader of each level is the
   pocket area (a 2D pocket has vertical walls), and no column is cleared
   down to a lower level, so the cleared area is empty.
8. **Incomplete and refused verdicts.** A refusal of a stage is the op's
   error, with its stage, its reason, its fix and its place (the place
   goes to the UI as the refusal spot). An incomplete verdict is refused
   too (`KERNEL_POCKET_INCOMPLETE`, with the count, the largest gap and
   its place): only a done verdict can reach the post. A converter
   refusal is the op's error. Each fallback and each note of a stage is a
   path note, once.
9. **The status line names the kernel** (`KERNEL_POCKET_NOTE`), so that a
   user who set the switch sees which planner cut the op.
10. **The registry.** `crates/pathkernel/tests/laws/kind_pocket.rs`
   registers the 2D Pocket as `Moved` with the real stage planners and the
   pocket's Waterline choices, on a closed square pocket and on a pocket
   with a round island made by the kernel's region stage. Its L7 record
   is the kernel's own first run: #3752 changes the old output on purpose,
   so the old cut sequence is not the record (the maintainer reads the
   changed programs at the switch-over, proof item 3). The whole-program
   proof is in motion (`kernel_laws/k6_pocket.rs`) and app
   (`pocket_pass_start/kernel_route.rs`).
11. **Changelog: none.** The default stays `old`, so no user sees a
   change. The switch-over PR writes the row.

**Cost to change later**: low. The recipe is one function per strategy
(`guide_and_load`) and the level loop is generic. A per-level material
top, blended corners and a keep-down limit are local changes.
