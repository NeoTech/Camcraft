## #2218 - Simulate and verify an external NC program

**What was decided.** `camcraft_app::nc_sim` simulates a program that
`camcraft-gcode` parsed. It cuts the setup's stock with the column engine,
measures the deviation from the target model, runs the solid law and the
two fixture laws, and runs the machine limits on each move. Each finding
has its 1-based NC line. `nc_sim::play_seed` builds a `PlaySeed` for the
playback. The runner reads the document and changes nothing, so it is no
undoable command. The MCP tool is a follow-up issue.

**The crate edge `camcraft-app` -> `camcraft-gcode`.**
- Imported items: `parse_with`, `ParseOptions`, `ParsedProgram`,
  `ParsedSection` (with its `WorkOffset`, `move_lines` and `tool_number`),
  `GcodeError`.
- The caller cannot pass the value in: the runner must read the parsed
  sections, their tool numbers, work offsets and line maps. A closure or a
  recorded file would copy the parser's types into `app`.
- What rebuilds: a change in `crates/gcode` rebuilds `camcraft-app` and the
  crates above it (`camcraft-ui`, the binaries). `camcraft-gcode` depends
  only on `camcraft-cl` and `camcraft-geometry`, which `app` already uses.
- The workspace entry comes back. #3955 removed it until a crate used it.

**Choices taken unattended.**
- Tool map order: the explicit `T -> tool` map (a library row by name, else
  a document tool by name), then the document's tool numbers, then a
  `.camtools` row that states the number. Each `T` number with no cutter
  geometry is listed with its reason. The simulation is refused when the
  list is not empty. Only milling kinds are cutters here. A probe, a lathe
  insert, a torch, a wire, an electrode or an undercut head is refused by
  name.
- Frame: `world = origin + sign * program_mm`. The sign is the setup's WCS
  flip times the machine's axis direction. This is the same law that
  `post::machine::check_path` uses to read program travel. An arc center
  maps with its plane's two axes. The arc sense reverses when exactly one
  of them is negated (`post::invert`).
- Work offsets: the caller can give an origin for each of `G54`..`G59`.
  Without one, only the setup's own offset is read, from its WCS. Any other
  offset is refused by name, never read as the setup's origin. A turned
  setup WCS is refused: the map holds a translation and a sign only.
- The first move: the parser starts from `Dialect::initial_position`, not
  from a place the machine stood. The program's first straight move is
  collapsed to its end point. A note says so. Without this, the approach
  from (0, 0, 0) gave false gouges and false fixture strikes.
- Gouge lines: the deviation of the dexel grid has no move index. The
  line-numbered gouges come from `verify::solid_hits` against the same
  target, at the simulation tolerance (default `DEV_TOL`).
- Machine findings: `MachineCheck::check_path` runs once for each move with
  `SourceLocation::NcLine`, so each finding starts with `line N:`. The tool
  size and the spindle speed are judged on the section's first move line.
  The changer capacity is a program-wide finding.
- Fixtures: the setup's fixture boxes (`fixture::boxes_for`), or the
  caller's boxes. The holder is the tool's holder cylinder.

**Dropped.**
- Using the field engine (`Held`) for the verdict: the column engine is the
  program verdict's engine, and a 3-axis program has no posed move.
- Reusing `posting::program_ops` or `stock_sim::plan_ops`: they read
  operations from the world. The runner builds the same `Planned` values
  from the parsed sections.
- Transforming the canned cycle records: the parser expands cycles to
  moves, and the runner keeps only the moves.

**Cost to change later.** The runner is one module with one entry point.
An MCP tool or a UI dialog calls `simulate_nc_text` or `simulate_nc`.

**Review-fix decisions (PR #4470).** These supersede the first-move
choice above.
- Require `NcSimOptions::initial_position_known` before simulation or
  playback. The caller confirms that the parser's initial position is
  the actual tool position. Preserve the complete first move for straight
  paths and arcs alike. Collapsing it can hide a collision between clear
  endpoints; a note cannot make that verdict safe. Inferring knowledge
  from a nonzero parser start was dropped because zero can be a real
  start, and any configured value can still be an assumption.
- Reject `taper::is_unread_tip` after tool resolution, for explicit,
  document and library mappings. Flat and ball taper tips must not be
  judged as sharp cones. Implementing their sweep and collision geometry
  was dropped from this review fix: those shapes need their own verified
  laws before this door can accept them.
- Use one fidelity preparation guard for simulation and playback. Check
  the selected stock, the document target and the selected fixtures;
  explicit target facets or fixture boxes replace their respective
  document inputs. Unknown fidelity refuses both entry points. Known
  coarse fixture boxes retain their existing chord growth. For reduced
  stock or target fidelity, subtract the maximum excess from the lesser
  of the requested tolerance and `DEV_TOL`, refusing when no positive
  budget remains. This uses the strict law because the checks include
  shanks and holders. Cutting-only tolerance widening and allowing an
  explicit tolerance to restore an exhausted fidelity budget were
  dropped: neither supplies an honest body-collision verdict. Playback
  receives the same adjusted tolerance rather than an unguarded value.

**Cost to change these choices later.** Callers must explicitly confirm
the parser start. Accepting other taper tips requires verified stock and
collision shapes. Relaxing the fidelity guard requires laws that preserve
the same surface and body bounds in both simulation and playback.
