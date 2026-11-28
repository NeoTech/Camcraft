## #2574 - A stand-alone camcraft-sim (research decisions, carried by #2586)

The research of #2574 used no branch. Its answers are in the issue
comments of 2026-11-18. This file records the decisions, as the research
asked, in the first code slice (#2586).

**The layout.** A new crate `crates/simview`, package `camcraft-simview`,
holds the binary `camcraft-sim`. It does not depend on `camcraft-app` or
`camcraft-ui`: a stand-alone program that builds the document engine
(flecs) is not stand-alone, and it shares the crash domain of the app. The
generated machine model is in `camcraft-sim` (#2584); the part tree type is
in `camcraft-post::machine` (#2583); the per-move limit judgment moved to
`camcraft-post` (#2585). `camcraft-sim` gets no `post` edge: `simview` maps
the tree into the model's own input types.
- Dropped: the viewer inside `camcraft-ui` (it pulls flecs and the
  document); the viewer inside `camcraft-sim` (that crate is data only).

**The old binary.** `crates/sim/src/bin/camcraft-sim.rs` moved into the new
crate as the subcommand `camcraft-sim cl <file.cl> ...`, with the same
arguments, output and exit codes. Two binaries with one name in one
workspace collide in `target/`. It was not in the release packages, so no
user loses it.

**The process launch (#2589, later slice).** CamCraft starts `camcraft-sim`
as a separate process (`std::process::Command`), found beside
`std::env::current_exe()`. A missing binary disables the button with a
named reason. Dropped: a shared window in the app process (it ties the
viewer to the flecs build and the app's crash domain).

**The file contract for Solidcraft (#2593, research).** A file, not a live
MCP call: the machine assembly is exported one time as URDF, and the `.mch`
names it and maps each URDF joint to an axis letter. `camcraft-sim` reads
only the file. Dropped: a live MCP call (a runtime dependency on another
process); a reader of the `.sc` document's joints (it couples CamCraft to
Solidcraft's document version).

**The `body` key layers (#2591, later slice).** One optional key on a
`.mch` part, in three layers: no `body` gives the generated model (#2584);
`body.primitives` gives boxes, cylinders and cones in the part frame;
`body.step` names a STEP file read by the solidcore reader. Each finding and
each pass on a generated body says "judged on a generated model". Dropped: a
separate definition language for the primitives.

**The exit codes.** 0: each check ran and passed. 1: bad input. 2: a check
failed (each finding names its NC line). 3: one or more checks were not
judged (each is named). A check that did not run never reads as a pass, and
2 wins over 3.

**The crate edges of `camcraft-simview`** (AGENTS.md "No new dependency
between our crates without a decision"). `camcraft-sim` is a binary: it is
the top caller, and it must read the files itself. No caller can pass the
values in.
- `camcraft-gcode`: `parse_with`, `ParseOptions`, `ReferencePositions`,
  `Dialect` and its seven profiles (`rs274`, `fanuc`, `haas`, `tormach`,
  `grblhal`, `tinyg`, `carvera`), `ParsedSection` (`tool_number`,
  `work_offset`, `toolpath`, `move_lines`), `WorkOffset::number`, and the
  text of `GcodeError`.
- `camcraft-post`: `machine::{parse_mch, parse_mch_tree, kind, Kind,
  Machine, MachineTree, TreePart, PartKind, MovingSide, MachineCheck,
  SourceLocation}` and `post::{engine::extension_of,
  dialect::declared_serves, js::Script, work_offset::work_offsets,
  PostError}`.
- `camcraft-sim`: `stock_sim::{Stock, Backend, SimOp, auto_cell,
  FIELD_BAND}`, `verify::{bands_vs_fixtures, rapids_vs_fixtures, Warning,
  FixturePart}`, `fixture::FixtureBox`, `tools::ToolGeom`,
  `machine_model::{generate, Body, BodyKind, BodySource, JointKind,
  MachineSpec, PartSpec, SpindleSpec, ToolStationSpec}`, and its re-exports
  `cl::{Move, MoveKind, Toolpath, text}`, `geometry::{Point3, Triangle}`,
  `geo2d::Point2`.
- No direct edge to `camcraft-geometry`, `camcraft-cl` or `camcraft-tools`
  (the research listed them): each type that `simview` uses comes through
  the re-exports of `camcraft-sim`, so a direct edge adds nothing.
- What rebuilds: `camcraft-simview` rebuilds when `gcode`, `post` or `sim`
  (and below them `cl`, `geometry`, `tools`, `kernel`) change. No crate
  depends on `camcraft-simview`: a change in it rebuilds only itself.
- `camcraft-sim` drops its `tracing-subscriber` dependency: only the moved
  binary used it.

**Cost to change later.** Low for the layout (one binary crate, no
dependants). The exit codes are a contract for scripts and for the
"Simulate" button: change them only with a breaking note.
