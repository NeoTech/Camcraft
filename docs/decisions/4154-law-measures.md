## #4154 - Path kernel K7a: the law measures on a finished toolpath, and the old planners' readings

**What was decided.**

1. The measures are one test-only module, `crates/motion/tests/common/law_measure/`
   (a directory module: `mod.rs` and six files, each under 400 lines). It
   reads only the toolpath, the stock outline and top, the part, the holes
   and the reach of the tool centre. It runs the same on an old planner and
   on the new kernel.
2. The app suite gets the module by `#[path]`, not by a new crate edge:
   `crates/app/tests/offset_pass_start.rs` declares an inline module
   `motion_common` with `#[path = "../../motion/tests/common"]` that holds
   `law_measure` and `mask_grid`. The module names only `camcraft_motion`,
   which is already a normal dependency of `camcraft-app`. No `Cargo.toml`
   changes. A copy of the file was dropped: two copies drift.
3. The engagement reader reuses the rim meter of `common/mask_grid.rs`
   (`Grid::contact_width`, `Grid::sweep_capsule`) on a column-height grid
   (cell 0.25 mm). The crossing reader calls the verifier's own law
   (`verify::crossed_pairs`). The fallback reader calls the planner's own
   classifiers (`is_entry_stand_in`, `rings_fallbacks`) and then a short
   word list for notes that have no classifier. The lift count is the
   one-level reader of `pass_start_levels/ring_links.rs`, done for each
   level. The `ring_links.rs` helpers stay as they are: the issue changes no
   existing test, and they are private to their suite and fixed to one Z.
4. The law marks of a reading (the third column of each table):
   - `link`: a lift with no named reason, a crossing of two planar feeds,
     or a planar feed whose centre leaves the reach;
   - `L4`: an entry into solid of another kind than the asked one, with no
     fallback note;
   - `L5`: a straight plunge or a rapid into solid. A helix or a ramp into
     solid is not marked: the record gives it the `IntoMaterial` proof;
   - `L6`: a cut above the planned engagement (one rim sample, 2.5
     degrees, of tolerance), or a full-width cut outside a named slot.
     Slot, Trace, Engrave, V-Carve, Chamfer, Thread and Slot Undercut are
     named slots (they cut at the tool width by plan).
5. The planned engagement of each case is `acos(1 - step / r)` of the
   operation's own step (pocket kinds, Face, Circular, the contour wall
   stock). Kinds with no planned step (Bore, Slot) use 180 degrees; the
   trochoidal slot uses its `max_engagement`.
6. The motion-level readings are one new directory suite,
   `crates/motion/tests/kernel_laws.rs` with `kernel_laws/`. No existing
   suite covers the 16 kinds, and a separate suite keeps the merge with the
   golden baselines of #4153 and the skeleton of PR #4123 free of conflicts.
   The fig8 readings join the existing app suite `offset_pass_start`, where
   the fig8 world builder is.

7. The fig8 cases run past the 60 s debug budget (2D Offset about 86 s,
   3D Pocket about 211 s, 2D Adaptive about 218 s, 2D Contour about 460 s
   in debug on this host), so they are in the app heavy lane. The 2D
   Pocket case (about 30 s) stays in the default lane. Each heavy case
   with two start sides has a light twin that reads the Inside case only.
   The 2D Contour case has one case and so no reduced set: it has no light
   twin. The motion suite reads the same contour planner on the disc and
   the dog bone in the default lane.

**Why.** The switch-over proof of #4024 needs each law measured old against
new on the same parts. The measures must not come from the planner (an
independent stock grid), so the same code reads the kernel later.

**Dropped.**
- A new `camcraft-motion` dev-dependency edge for a shared test crate: the
  module is test code, and `#[path]` needs no edge.
- Reading the simulator stock of `camcraft-sim`: its API is larger than the
  need, and a 2D column grid is enough for one-level 2D cuts.
- A cone or ball tool model: V-Carve, Chamfer, Thread and Slot Undercut are
  read with a flat disc, so their engagement column is a reading only.

**Cost to change later.** The tables pin strings. A change of a measure
changes the tables in one PR; the `pin` helper prints the new table.
