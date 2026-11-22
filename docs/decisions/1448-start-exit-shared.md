## #3324 - Start and exit points: the shared mechanism (design for #1448)

The maintainer set the scope of #1448 on 2026-11-22: a user start point and
exit point on every operation that cuts a path, through one shared
mechanism. This file records each answer of #3324. The family sub-issues of
#1448 build on it. "Pass starts at" of #3316 (`docs/decisions/3316-pass-start-shared.md`)
is a separate parameter; section 7 says how the two work together.

### 1. One shared type - chosen: B, `camcraft_motion::path_picks`

- **Decided.** `PathPicks { start: Vec<PathPick>, exit: Vec<PathPick> }`,
  with `PathPick { xy: Point2, z: Option<f64> }` in model mm and a
  `PickRole` (`Start` / `Exit`). One accessor, `OpParams::path_picks()`,
  answers it for every kind. Today it reads the contour family's
  `links.approach.start_point` / `exit_point` and the wire's
  `WireContourOp.start`.
- **Why.** Each family planner reads one shape. Option A gives `links` to
  kinds where links mean nothing (turning, EDM, plasma). Option C repeats
  the refusal law for each kind.
- **Shape of a pick.** A model-frame point with an optional Z. Each family
  projects it into its own frame (setup XY, the tilted frame, the lathe
  (Z, X) plane).
- **One pick or a list.** The type holds a list now, so the file format
  does not change again when a family takes one pick for each chain or
  region. `PICKS_PER_ROLE = 1` today; `PathPicks::refusal` refuses a
  second pick by name. A family that reads a list raises the number in its
  own slice.
- **Dropped.** A (links on every kind), C (a field for each kind).

### 2. The project key - chosen: B, staged

- **Decided.** The target key is `picks: { start: [[x, y, z?], ...],
  exit: [...] }` at the operation level. The loader keeps reading the old
  `links.approach.start_point` / `exit_point` keys and the old wire
  `start` key, so every saved file opens unchanged. The digest keeps
  `0xC7` / `0xC8` for the same values, so no contour golden moves.
- **Staged.** This PR does not move the storage. The move puts one field
  on every path variant of `OpParams` and its DTO. More than ten open PRs
  change `wire_edm.rs` and `wire_edm_dto.rs` today, so the move in the
  same PR would conflict with all of them. The accessor of section 1
  hides the storage, so the family slices do not wait for the move. The
  move, with the load tests of the two old keys, is its own sub-issue of
  #1448: #3339. It must merge before the first family slice that has no `links`
  field (Face, turning, plasma).
- **Dropped.** A (keep `links.approach.*` as the key for good): a kind
  without links cannot hold it.

### 3. The command - chosen: A, `EditOperationParams`

- **Why.** One undo step changes the pick and "Pass starts at" together.
  The maintainer asked for one command. A pick is a parameter, and
  `named_refusal` already runs at create, edit and load.
- **Dropped.** B, a dedicated `SetStartExit` command: a second door to the
  same parameters.

### 4. The MCP tool - chosen: A, the patch of `edit_operation`

- **Decided.** The schema of `edit_operation` names
  `links.approach.start_point` and `links.approach.exit_point` and the
  kinds that take them. Each kind of `list_operation_types` has a
  `start_exit` answer: `start`, `exit` (true / false), and `field` or
  `why_not`.
- **Dropped.** B, a `set_start_exit` tool: two doors to one command.

### 5. The viewport pick and the rows - chosen: a capability per kind

- **Decided.** `KindInfo::pick()` returns a `PickCapability`: `Ring`,
  `Region`, `HoleOrder`, `Ends`, `Raster`, `Threading` (the wire's start
  only) or `NotTaken(reason)`. The match is exhaustive on `OpType`, so a
  new kind does not compile until it states one. Today: 2D Contour is
  `Ring`, Wire Contour is `Threading`, Probe is `NotTaken` ("cuts no
  path"), and every other kind is `NotTaken(PICK_OWED)` until its family
  slice ships. A family slice changes only its own arm.
- **Snapping.** The viewport pick snaps by capability: a wall for `Ring`,
  a point in the region for `Region`, a hole for `HoleOrder`, an end for
  `Ends`, a corner for `Raster`. Only `Ring` exists in the pick code
  today; each family adds its snap.
- **Rows.** Every operation page now draws the Start point and Exit point
  groups. A kind that does not take a pick draws each group with one line,
  "Not taken - <reason>", on the Lead in / Lead out tab, or on Geometry
  when the kind has no lead tab. The contour keeps its editable rows. A
  family slice moves its kind onto the editable rows of
  `start_exit_rows.rs`.
- **Dropped.** A row drawn only on the kinds that take a pick: the
  maintainer asked that "the Start point row says so by name" where a kind
  has no free start.

### 6. How the path is generated again to the point

- **Decided.** The pick is an input of the planner. The command changes the
  parameters, and the normal generate runs after it, as for the contour
  today.
- The ring helpers of `contour_start.rs` (`nearest_on_ring`, `reach`,
  `lead_held`) move to a shared module in `crates/motion/src/` in the first
  family slice that needs them (2D Offset or Pocket), not before: a move
  with no second caller is churn. Each family keeps its own seam.

### 7. Beside "Pass starts at" (#3316)

- The two parameters stay two fields of the same parameter group. The pass
  side chooses which ring of a level cuts first; the start pick chooses
  the place on that ring and the descent place.
- Order in the planner: the pass side first, then the pick is projected
  onto the first ring that the pass side gives. A pick that the pass side
  makes impossible ("Outside" with a start pick inside the part) is
  refused by name.

### 8. A point that cannot be reached - chosen: A, refuse by name

- **Why.** The maintainer: "a pick is a wish. The planner refuses it by
  name when the lead law or the safety laws cannot hold it. It never moves
  the pick without a note."
- **At command creation** (`named_refusal`, `operation::pick_kinds`): the
  kind does not take the pick (the message names the kind, the pick and
  the reason), or the pick is not finite or is more than 1 km from the
  origin (`Approach::validate`, `PathPicks::refusal`). A pick outside the
  setup needs the world, so it is a generation refusal.
- **At generation:** no lead fits, the helix does not fit, the pick is too
  far from the path, the pick conflicts with the pass side. The operation
  is refused and the post is blocked.
- **Dropped.** B, move the pick to the nearest legal point with a warning.

### What changed in the messages

The refusal of a pick on a kind that does not take it was "a start point is
taken only by the 2D contour so far - clear it on this operation". It is now
"<kind>: a start point is not taken - <reason>; clear it on this
operation". The tilted contour's refusal is unchanged.

### Cost to change later

The capability enum and the accessor are additive. A family slice changes
one match arm and its planner. The project key of section 2 is the one
part that is hard to change after it ships, so it is fixed here before the
move.
