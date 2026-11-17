## #2201 - the shown operation status is cached on a world write revision

**Decided.** The tree, the status bar, the operation page and the debug
panel read `operation::status_shown` and `operation::stale_reason_shown`.
These keep the answer of `status_of` / `stale_reason` for each operation
until the world's write revision (`world_rev::of`) moves. The revision is
a new singleton component, `schema::WorldRevision`. One flecs observer on
the wildcard id (and one on the wildcard pair) moves it on each `OnAdd`,
`OnSet` and `OnRemove` of each component, except the two read-time memos
`GeoSig` and `GeomPrintMemo` and the revision itself. Each move takes a
fresh number from one process counter, so two worlds never share a value.

Generation, verify, the collision checks, the post, the post dialog,
`RegenerateAll`, MCP and the script API keep calling `status_of`, the
uncached truth.

**Why this key is complete.** `status_of` reads only world components:
the `Operation` (tool name, source, params, area controls, setup and the
stored stamp), its `Toolpath`, the tool entity, the source and picked
entities' geometry and `Location`, the setup's `SetupDef` and stock
source, the stock entity, the part alignment, the work offset, the
`MachineDef` (wire and sinker conditions), the unit setting, and the
toolpaths and order of the operations ahead (`rest::prior_digest`). The
content-keyed memos on the way (`chain_memo`, `carried_cache`, the feature
cache, `setup_offsets`) are pure functions of those values. No file is
read on the stamp path.

Each writer of those components and how the key covers it:

| Writer | How it writes | Covered by |
|---|---|---|
| Every command `apply` (create, edit params / links / geometry, rename, reorder, tool add / change / delete, stock, setup, machine, post options, model import / update / move / repair, fixtures) | `set`, `add`, `remove`, `destruct` | OnSet / OnAdd / OnRemove |
| Undo and redo | the command's `invert` / `apply`, same calls | same |
| Generation result and `ClearToolpath` | `set` / `remove` of `Toolpath` and `Operation` | same |
| Verify results | carried inside `Toolpath.warnings`, written with the toolpath | same |
| Project load (`project::open` into a live world, or a new world) | `set` into the world; a new world starts with a fresh revision | same, and the per-process counter |
| Entity delete | `destruct` | OnRemove of each component |
| `SetSetupPostScriptOptions` | was the one in-place write (`get::<&mut SetupDef>`), which sends no event | changed to clone + `set` in this PR |

A source scan test (`status_cache::no_component_is_written_in_place`)
fails when a new `get::<&mut ..>`, `each::<&mut ..>`, query or system
with a `&mut` term, `ensure_mut` or `get_mut::<` appears in `app` or
`ui`. No schema component holds interior mutability (`Cell`, `Mutex`,
atomics) except `WorldRevision` itself. Selection changes move the
revision too: that is a harmless extra rebuild, never a stale answer.

**Dropped.**
- `doc_version` as the key: the UI bumps it by hand, and
  `crates/ui/AGENTS.md` forbids it as the only key.
- A digest of the inputs as the key: to compute it is the cost we remove.
- A revision per input (tool, stock, setup, prior chain): eleven writers
  in many commands, and a missed bump is a stale "OK". The ECS observer
  covers all writers by construction.

**Cost to change later.** The observer runs on each component write
(one atomic store). A heavy writer loop can cost a little. If the
revision must become narrower, filter the observed ids; the tests in
`crates/app/tests/status_cache.rs` pin each writer.
