## #4406 - Path kernel law suite: each law assertion can break, and a refusal is never held

Review of PR #4383 (2026-11-27, class A). Taken unattended; the rule of
the class is the reviewer's.

**What was decided**

1. **A refusal is a verdict of its own.** `kit::LawVerdict` carries an
   `Outcome`: `Held`, `Broken`, or `Refused(reason)`. A `Measure` can be a
   refused or unread leg (`Measure::refusal`, `Measure::unread`). `judge`
   gives `Broken` when a read measure is above its limit, else `Refused`
   when a leg was refused or unread or when there is no measure, else
   `Held`. `refused_held` (a refusal as 0 against 0) is removed; its callers
   use `kit::refused`. Before, an empty measure list was held ("nothing to
   measure"); now it is refused.
2. **The law runner fails a refusal that is not registered.**
   `registry::KindRecipe` has `refusals: Vec<ExpectedRefusal>` (a case name
   and the exact reason). `law_failures` lists each broken verdict and each
   refused verdict whose reason the kind did not register for that case.
   `each_moved_kind_obeys_every_law` asserts that the list is empty. An
   expected refusal never hides a broken law.
3. **Each check, leg by leg.**
   - L1: the first call gives the base. A refused base is refused; a later
     call that refuses against curves is broken. Two refusals are never
     compared by their reason text (the review note: the stand-ins share
     one reason), so the limit of the old match is gone, not documented.
   - L2: a region that refuses one material side is broken; both sides
     refused is refused. A side flip that the kit cannot build is broken.
     The order leg is never skipped; the order refused for both regions is
     refused.
   - L3: each side refused is refused; one side refused where another
     plans is broken (as before); a run with no curve is unread.
   - L4: a guide with no curve, or an entry verdict with no entry, is
     unread.
   - L5: an `IntoMaterial` descent onto material is read. It must move in
     the plane by one grid cell or more (a helix or a ramp), or be a plunge
     that the user asked for. Else it is a plunge named as a helix or ramp:
     broken, measured by its fill. The stand-in helix (`standins::helix`)
     was a one-point descent (a plunge with the name of a helix); it is now
     one turn of radius half the tool radius. A plan with no entry is
     unread.
   - L6: the report row "the highest read ae/D" (always held) is dropped;
     the highest read is in the place text of each cut row. A plan with no
     cut is unread.
   - L7: the record comes from a file, not from the run under test (see 4).
     An empty record against an empty plan is unread; an empty record and a
     refused run is refused with the kernel's reason; an empty record
     against cuts is broken (count differs).
   - Link rule: a region that is not planar makes the stay-link leg a
     refusal, not a skipped leg.
4. **L7 in this crate reads a recorded file.** The old holds test
   compared the kernel with its own output of the same run. The real L7
   record is the old planner's output; its goldens are in `camcraft-motion`
   (`tests/pass_start_levels/kernel_baseline/`, `tests/golden/kernel_*`),
   and the old-against-new comparison is at the motion level
   (`tests/kernel_laws/`). The kernel crate cannot depend on motion. So the
   stand-in kind's record is a file, recorded one time from the stand-in
   planners and kept in `crates/pathkernel/tests/laws/records/`
   (`square_pocket.txt`, `pocket_with_an_island.txt`). The square record
   can be checked by hand: the 16 mm centre square from (2, 2), ten rings
   0.8 mm apart, the innermost first. The test reads the
   file and does not write it. A change of the stand-in output, or of the
   kernel types and the geometry under it, now breaks `l7_holds_against_the_recorded_file`
   and the registry runner. There is no `WRITE_GOLDEN` switch: a new record
   is a hand edit that a reviewer reads.
5. **The proof.** Each law has a stand-in that breaks it (in its law file,
   as before, plus `HiddenPlunge` for L5 and `no_outside` for L2) and a
   stand-in that always refuses, reported as refused (`refusals.rs`). The
   test `no_law_is_held_on_an_always_refusing_planner_set` runs every check
   on a planner set where each stage refuses, and on a case whose region
   refuses: no law is held, each law except L7 is refused, and L7 is
   broken (the record has cuts, the kernel refused).
6. **No change in `crates/pathkernel/src`.** Each check reads the public
   API. The geometry helpers of `kit.rs` moved to `kit_geo.rs` (the
   400-line law); `kit` re-exports them.

7. **Old passes that were hollow (findings).**
   - `guide_laws.rs`: the medial planner, the circle rings and the helix
     planner "held" L2 only because their fixture regions had no stock.
     The real region stage refuses the Outside side by name ("an Outside
     material side needs the stock: the area outside the selection has no
     limit"), and L2 read that refusal as 0 against 0. With the new rule L2
     broke (measured inf against 0). The fixture now gives a stock box (10
     mm plus four tool radii at each side), and L2 is measured and holds.
     No kernel source changed.
   - The stand-in helix of `standins.rs` was a one-point descent (a plunge
     with the name of a helix). The L5 check now catches such a descent
     (`l5_breaks_for_a_plunge_into_material_named_a_helix`).
   - The L6 breaker pin read 0.9 or more on the full-width first ring. The
     helix now clears the centre of that ring, and the worst unnamed read
     is a ring corner at ae/D 0.711 against the cap 0.625. The pin now
     asks for more than 0.7; the check did not change.
   - The link rule skipped a stay link with no sample (a hop of zero
     length). It now reads the points of such a hop.
   - The registry has no moved kind yet, so `each_moved_kind_obeys_every_law`
     ran nothing before and runs nothing now. The runner itself is pinned
     by `the_law_runner_fails_a_kind_whose_stage_refuses_a_case`.

**What was dropped**

- Dropping the L7 holds test: the check would then have no test that it
  can hold on a real plan. A recorded file is the form that the record
  names ("golden `.cl` recorded on the head before the step").
- Moving L7 to `crates/motion/tests/kernel_laws/` in this PR: the
  old-planner comparison already lives at the motion level, and the
  kernel crate keeps a check that can break.
- Counting a refusal as "held with a note": a refused law is a legal
  result of the kernel (Prime Directive), but it is not evidence that the
  law holds.
- A `NotMeasured` outcome apart from `Refused`: the runner treats the two
  the same (only a registered reason passes), so one outcome with the
  reason is enough.

**Cost to change later**: a check is one function in one file; an outcome
is one enum. A new record file is one text file.
