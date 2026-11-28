## #4407 - Path kernel: no silent recovery from a rejected result, and the facts of its two crate edges

Part of #4023. Review of PR #4383 (the path kernel to `main`), class B
and the minor items, review of 2026-11-27 on head `7cd322966`. A sub-PR
of `side/path-kernel`.

### Plain summary

- The new path planner is still off: no program changes.
- Where the new planner cannot use what it tried (an entry, a lead-out, a
  geometry result), it now always says why. It either refuses with a named
  reason, or it uses another way and writes a note that names the check
  that failed. Before this change, some of these cases were silent.
- The converter that writes the moves of the new planner now refuses an
  exit whose retract is under the clearance height. Before, it moved the
  retract up to the clearance height and said nothing.

### What was decided

**The rule.** Never recover in silence from a refused or rejected
result. Each place refuses by a name of its own stage, or it records the
fallback with a note (or a named `Fallback`) that names the failed check.

1. **An unoriented cutter-centre area is refused, not made normal
   again** (`region_build.rs`). The builder called `Region::new` on a
   centre whose loops disagreed with their nesting. Now the centre goes to
   `PlanarRegion::new` as the booleans gave it, and that constructor
   refuses it with `NOT_ORIENTED`. The canonical form is the contract of
   `camcraft-geometry` (each region is normal), so a fault there must be
   seen, not hidden. A public value still cannot reach the refusal (the
   geometry gives only normal regions; K1 decision 10 of
   `4025-kernel-region.md`), so the guard is pinned on the loops
   themselves: `region_check::tests::a_loop_against_its_nesting_is_found`.
   Dropped: keeping the re-normalisation and removing the refusal (the
   constructor would then accept a centre that no check reads).
2. **A cut above the material top says why** (`entry/stage.rs`). The
   tool goes straight down in air. `ABOVE` (now public) is the reason of
   the named fallback where `needs_fallback` asks for one (an asked hole
   or column), and the text of a note where the asked kind was not an air
   entry (a helix, a ramp, a plunge, an arc lead). An asked air entry gets
   neither: it is the entry flown. Dropped: a `Fallback` for an asked
   entry into material, because law 4 (`needs_fallback`) says that an
   entry outside material in its place is not a fallback.
3. **One checked path for the retract of an exit** (`entry/exit.rs`).
   `Exit::new(lead_out, retract, top)` and `Exit::proved` call the same
   `check_retract`. The check refuses a retract or a top that is not
   finite, a retract under 0 (`BAD_EXIT`), and a retract under the
   material top (`RAPID_IN_MATERIAL`). `Exit::new` gets the material top
   as a new argument (the work of #4415, done here after #4406 merged);
   its callers in the law suite and in the converter's tests pass the top
   of their fixtures. The converter refuses an exit under its clearance
   height as well (item 4).
4. **The converter refuses a retract under the clearance height**
   (`kernel_emit/pen.rs`, `EXIT_UNDER_CLEARANCE`). It moved the retract
   up to the clearance height in silence. Now `retract` returns a
   refusal for a height under the clearance height (minus `EMIT_TOL`) and
   for a height that is not finite. A lift retracts to the clearance
   height exactly, so lifts do not change. The converter's own test plans
   used an exit of 5 above the level (Z 3, under the clearance of Z 10);
   they now use the clearance height (12 above the level).
5. **An exit with no lead-out names its fallback** (`entry/exit.rs`).
   `Exit` has a `fallback()`: asked `arc lead-out`, used `straight
   retract`, with the reason `NO_LEAD_OUT_ROOM` (no arc fits where the
   tool cuts nothing), `NO_END_DIRECTION` (the last cut has no direction
   at its end), or the reason of a lead refusal other than
   `LEAD_OFF_ROOM`. An empty or zero-area room is "no room"; a room with
   an area that `Area::new` refuses (a point that is not finite) is now a
   refusal of the exit. The converter writes the fallback to the notes:
   `exit: straight retract instead of the arc lead-out - <reason>`.
6. **A refused try keeps its reason** (`entry/{air,hole,column,material}.rs`,
   `entry/room.rs`). Each `.ok()` and `let Ok(..) else` on a checked
   constructor (`Entry::new`, `Entry::led`, `Lead::new`, `Area::new`, a
   straight lead) goes through `Ctx::checked`, which keeps the kind and
   the named reason. The stage still tries the next place or kind. When an
   entry fits, each refused try is a note (`entry: a try of the <kind>
   was refused by its check - <reason>`). When none fits, the reasons go
   into the refusal: `Refusal::after` and `Refusal::tried`, and the text
   form ends with `; tries refused: ...`. An empty air zone or an empty
   column is "does not fit" (no note), not a refused check. A geometry
   error of the room tests (`Ctx::holds`, the clip of an air lead) is now
   a refusal with the kind of the error, not "does not fit"; a helix point
   that is not finite is refused by name, not "does not fit".
7. **The trochoidal pitch** (`load/trochoid.rs`). The review named line
   188 as an `.ok()`; the line is the clamp of the asked pitch to the
   engagement limit (the limit times the tool diameter). The clamp stays
   (the limit is the user's law), and the load verdict now says it: `load:
   the trochoidal pitch is limited to <p> mm by the engagement limit,
   under the asked <q> mm`, whenever the asked pitch is above the limit
   (also when no stretch gets loops: the note says the pitch that the
   stage plans with). The same search found
   `load/rings.rs`: a piece of a ring level that `Curve::new` refused was
   dropped. A piece too short to be a curve (`BAD_CURVE`, `ZERO_LENGTH`)
   is still no ring there; each other refusal is now a refusal of the
   stage.
8. **An empty arc output has its own reason** (`kernel_emit/arcs.rs`,
   `ARC_EMPTY`): a circle cut whose chords are all under the tolerance
   gives no arc. It was reported as `ARC_NOT_MONOTONE`.
9. **A geometry error keeps its kind.** The four `geo()` helpers
   (region, guide, load rings, entry) take a `GeoError` and give
   `<STAGE>_GEOMETRY` for `OutOfRange` and the new
   `<STAGE>_GEOMETRY_INVALID` for `Invalid`, through one function
   (`geo_refusal` in `lib.rs`). The reason is a `&'static str`, so the
   text of an `Invalid` error is not carried; its kind is.
10. **The circumcentre of the medial axis** (`geometry/geo2d/medial.rs`)
    checks its denominator (finite, not zero) before the division. The
    result does not change: a flat or non-finite triangle gave `None`
    before too.
11. **The planner switch** is read one time for each process
    (`OnceLock`); its doc comment now says that a later change of the
    variable has no effect until the process starts again.

**The rest of the class (the search of the crate and of
`kernel_emit/`).** These stay, with the reason:
- The link stage (`link/approach.rs`, `link/proof.rs`, `link/route.rs`):
  a chord, an approach or a room that cannot be proved (a geometry error
  included) gives a lift with a named `LiftReason`. A lift is the named
  fallback of the record ("Stage 5: Link") and it goes to the notes
  (`LinkTally`). No change.
- `load/law.rs`: a stock reader that does not know a place
  (`material_at` gives `None`) counts as material. This is the documented
  contract of `StockRead` (the stage must not claim a proof); it can only
  add load, never hide it.
- `unwrap_or` on an index into a list of the same length
  (`order/stage.rs`, `load/rings.rs`, `load/trochoid.rs`,
  `region_check.rs`), `unwrap_or_default` on the first point of a curve
  that has points by construction (`guide/cover.rs`, `guide/trace.rs`),
  and the first lead-out point in `pen.rs` (a lead has two or more points
  by construction): no result is refused there.
- `entry/room.rs` `air_zone`: the approach distance is limited to half
  the tool radius, and the limit is a note (`capped`). No change.

**Readings.** No measured count of the kernel readings changed
(`kernel_laws` k3, k4 and k6a): see the PR body for the results.

### `WRITE_GOLDEN=1`

The baseline goldens of `kernel_baseline` (and the other golden suites)
are written again when a run sets `WRITE_GOLDEN`. What guards a silent
re-record:
- The gate never sets the variable (`xtask/`, `scripts/` and
  `.github/` do not name it), so a gate run always compares.
- The goldens are committed files under `crates/motion/tests/golden/`.
  A re-record is a diff of those files in the PR, and the review reads
  it. The module doc of `kernel_baseline` says that a golden must not
  move unless the output change is intended.
No new mechanism: a check that fails a run under `WRITE_GOLDEN` would
change the procedure of each golden suite of the repository, which is
outside this issue.

### The fig8 2D Pocket readings test

The test is in PR #4328 (`issue/3752-kernel-pocket-recipe`), which is
not merged into `side/path-kernel`. This PR does not change it.

### Dependency facts of the two edges

AGENTS.md **No new dependency between our crates without a decision**.
The maintainer approved the crate and its two edges on 2026-11-25 (record
of #4024 in `RECOMMENDED.md`, heading `## #4024`, "Dependency decision").
This section gives the facts that the record left for K1, from the `use`
lines of the code on `side/path-kernel`.

**`camcraft-pathkernel` -> `camcraft-geometry`**

Imported items (`crates/pathkernel/src/`):
- `geo2d::{Loop, Point2, Region}` (re-exported from `lib.rs`), with the
  methods of `Region` that the stages call: `new`, `empty`, `offset`,
  `union`, `intersect`, `difference`, `sweep`, `contains`, `holds_path`,
  `clip_polyline`, `area`, `loops`;
- `geo2d::GeoError` (`lib.rs`, the four `geo()` helpers);
- `geo2d::arc_points` (`entry/air.rs`, `entry/material.rs`);
- `geo2d::{point_segment_distance, segment_gap, segments_cross}`
  (`link/proof.rs`);
- `geo2d::{RingClip, offset_side}` (`guide/rings.rs`, `guide/trace.rs`);
- `geo2d::MEDIAL_MAX_SAMPLES` and the medial axis of `Region`
  (`guide/medial.rs`).

Why the caller cannot pass them in: they are the data of each stage (the
region, its loops and points) and the operations on that data (offset,
booleans, sweeps, clips). A stage computes with them on each call; a
value or a closure from the caller would have to carry the whole polygon
engine.

What rebuilds when `camcraft-geometry` changes: `camcraft-pathkernel`
and its tests, then each crate that depended on geometry before the edge
(`motion`, `sim`, `app`, `ui` and the others). The edge adds no new
rebuild: geometry was already under motion.

**`camcraft-motion` -> `camcraft-pathkernel`**

Imported items (`crates/motion/src/`, outside the tests):
- `machining2d/kernel_region.rs`: `Bounds`, `Chain`, `KernelRegion`,
  `MaterialSide`, `Refusal`, `Selection`, `Stage`, `ToolShape`,
  `WorkPlane`, `region`, `region::OPEN_NEEDS_SIDE`,
  `region::REGION_GEOMETRY`;
- `kernel_emit/`: `entry::{Entry, Exit}`, `link::Hop`, `plan::{Cut,
  Plan, Step}`, `guide::Circle`, `PlanePoint`, `WorkPlane`;
- `lib.rs`: `pub use camcraft_pathkernel as pathkernel` (the app reaches
  the crate, its switch values among them, through this re-export, with
  no direct edge).

Why the caller cannot pass them in: they are the planner itself (the
stage functions and the checked types of their results). Motion calls
them; no value of motion can stand in for them.

What rebuilds when `camcraft-pathkernel` changes: `camcraft-pathkernel`,
`camcraft-motion`, `camcraft-app`, `camcraft-ui` and their tests. A
change does not rebuild `geometry`, `cl`, `gcode`, `tools`, `post`,
`sim`, `kernel` or `gpu`.
