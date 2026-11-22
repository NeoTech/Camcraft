## #3316 - "Pass starts at" is one shared parameter (2D Offset first)

**What was decided.**

- One domain type, `machining2d::PassStart { Auto, Inside, Outside }`,
  in `crates/motion/src/machining2d/pass_start.rs`. `app` re-exports it.
  Every kind with a pass order uses this type. No kind gets its own enum.
- One project key, `pass_start` (`project::PassStartDto`, snake case).
  `auto` is the default and is never written. Files without the key read
  as Auto. There is no version step, and every file without a choice
  stays byte-identical.
- One command: the field is a plain `OpParams` field, so
  `EditOperationParams` carries it (undo and redo included). One MCP
  tool: the generic `edit_operation` merge patch. The patch guard keeps an
  explicit `"auto"` (`pass_start_kept` in `mcp/edit.rs`), because a
  skipped default never comes back from the round-trip.
- One Passes-tab row, `Pass starts at`
  (`crates/ui/src/egui_app/pass_start_row.rs`). A kind shows the row when
  its pane passes `Some(&mut pass_start)`.
- The digest writes the value only off Auto, so every earlier operation
  keeps its cache stamp.
- The ring law (`plan_passes_from`):
  - Auto: `waterline_passes`, not changed (byte-identical, core first).
  - Inside and Outside: `sided_rings::sided_passes`. Each component of
    the cutter-center area (an outer loop with its own holes) is planned
    alone. Ring `j` is the boundary of the START SIDE alone, moved `j`
    stepovers into the area: Outside fills the outer loop and erodes it;
    Inside fills the holes and grows them. The parts of ring `j` that
    lie in the area are open pieces that end on the far wall. Ring `j`
    is cut before ring `j + 1`. The far wall loops (Outside: the holes;
    Inside: the outer loop) are cut last, whole.
  - Each piece then has cut material one stepover behind it, so it cuts
    one stepover. On an Outside band the first ring is the stock reach:
    the tool starts in air.
  - Direction: climb keeps the uncut material on the right (#1555). The
    start-side rings of Outside run clockwise for climb; those of Inside
    run counter-clockwise. The far wall rings use the Auto ring law.
  - Inside on a component with no hole has no inside to start from. That
    component keeps the core-first ring order.
  - Two consecutive pieces share a pass only over a chord of at most two
    stepovers that stays in the area (`chord_clears`) and crosses no
    piece and no earlier chord. Else the pass ends and the caller links
    to the next one with a retract and a new entry.
  - Lead-in: a new pass starts at the nearest point of a piece that this
    level already cut (at most 1.5 stepovers away, with the same chord
    proofs). Without it, each open piece starts on the far wall in uncut
    stock: a plunge or a ramp at the part wall for every pass. On fig8
    the shank-reach law then took about 12 s for each such entry move.
  - End trim: an open piece stops 50 um short of the far wall at each
    end. A split point stands on the wall loop; after the micron lattice
    it can stand a hair across the wall ring, and the verifier then
    reports feed crossings (141 on fig8 without the trim). The wall ring
    cuts the last 50 um.
  - A piece whose inner vertex is not in the area is refused by name
    ("a start-side ring leaves the cutter-center area"), and a hole with
    no enclosing outer loop is refused. The output never leaves the
    cutter-center area silently.
- A start on a wall gets a note (`start_wall_note`, added by
  `generate.rs`): Outside where the outer boundary of the area has a
  vertex in the stock outline (a closed pocket), and Inside where the
  area has a hole (a part or island wall). The first pass of that level
  has uncut stock on both sides. No order can avoid that.
- Only 2D Offset reads it in this PR. The motion entry points are
  `pocket_started` and `pocket_avoid_started`; the old
  `pocket_entered_with_relief` and `pocket_avoid_with_relief` call them
  with Auto. `plan_passes` calls `plan_passes_from` with Auto, so the 3-D
  pocket, the HSM fallback and rest machining are not changed.

**Room for the #1448 start point.** The start point (where on its ring a
pass starts) and the exit point go beside `pass_start` as their own
fields with their own key. A whole-loop piece is entered at its vertex
nearest the previous end today; the start point replaces that choice.

**Why.** The maintainer asked for one parameter with one shape for every
kind (#3316 comments, 2026-11-22). The field rides the existing command
and MCP door, so the GUI and agents use the same undoable path. The
second read of PR #3353 showed that an order of whole closed rings
cannot be correct after a band splits: one loop of a split piece has
arcs on the stock side and arcs on the part side. The level curves of
the start side alone remove that case.

**Considered and dropped.**

- A rank of whole closed rings by family (the first head of PR #3353).
  It cut full slots along the part-side arcs of split pieces (pinned by
  `pass_start_engagement::outside_on_a_split_band_never_slots`, red on
  that head at 8.03 mm wide for an 8 mm tool).
- A clip with Clipper2 open subjects. The direction of an open path out
  of Clipper2 is not part of its contract, and the climb law needs it.
  The clip here splits the ring at its crossings with the far loops and
  keeps the parts whose longest edge midpoint is in the area.
- A `pass_start` field on `PocketParams`. About 57 struct literals
  without `..` would change for one value that only one kind reads now.
- A change of Auto for an Outside band. The maintainer said Auto keeps
  the current behaviour.
- A fill of far-wall rings before the wall ring (offsets of the far
  wall, only where they stand off every start piece). It was written
  for a deep notch that the start rings might not enter. A figure-eight
  waist pin passed with and without it, and a notch that opens toward
  the start side is entered by the start rings. So it was removed: no
  pin proved it.
- Both-way pieces (alternate directions, so each piece chains to the
  next across the corridor). It would mix climb and conventional cuts.
  The climb setting stays true on every piece; the lead-in covers the
  entry instead.
- Long keep-down chords between pieces. A chord near the far wall can
  run through uncut stock at full width; the two-stepover limit keeps a
  chord to a step-over move.

**Cost to change later.** Low. The type and the key are shared already.
A new kind adds its field, its row call and its planner read.
