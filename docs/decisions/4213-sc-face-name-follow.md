## #4213 - D4: the source face record is saved under `source_faces`, with no version step

**Decided.** A model imported from a `.sc` file saves its source face record (SolidCraft's name for each record face, and the facets each record face became) under the optional entity key `source_faces`. `PROJECT_VERSION` does not change. The key is in `project/source_faces_dto.rs`. The hooks are one line each in `entity.rs`, `wire.rs` and `apply.rs`.

```text
"source_faces": {
  "names":  [ { "name": "F2:side:a", "verified": true, "former": "<optional>" }, ... ],
  "ranges": [ [start, end, face], ... ]
}
```

- At save, only a record that tiles the facets it rides is written.
- At load, a record that does not read, does not tile the facets, names a face past `u32`, or holds a name longer than 16 KiB is dropped with a logged note. The file opens as it would without the key. The key never refuses a project.
- `verified` absent is false: a lost flag can only stop a follow.

**This is the maintainer's call (Q1).** The other form adds a version step, as `source` (#2463, v72) did.

**Why.**
- The `solids` precedent (#917): "an old reader that drops it reads the soup". Here an old reader drops the key and reads the model exactly as today. It loses only the follow by name at the next re-import, which is today's behaviour.
- A version step lands only at a cut. It makes every older build refuse every file that this build saves. `PROJECT_VERSION` also moves often (PR #4173 is a `feat!` on the project file, and its v83 step must now take a number past 84), so a step must be numbered again at each rebase.
- The record must be saved, not only held in memory: a headless run and every reopened project load the old mesh from the file, and a new revision often arrives while the project is closed.

**The versions on 2026-11-27.** The last release, v0.27.0, writes v82. `main` writes v84 (#3645). Neither v83 nor v84 is in a release yet. The decision holds in the two cases:
- This change merges before the cut that ships v84. Then each released build that opens the file knows `source_faces`, and v0.27.0 refuses the file for its version. A step of its own adds nothing for a released build.
- It merges after that cut. Then a released v84 build drops the key and reads the model as today: the `solids` case above.

**Considered and dropped.**
- A version step (the `source` precedent). See above.
- `deny_unknown_fields` on the record. It would make a later reader's extra field refuse the whole project in this build.

**Cost to change later.** A version step is one line in `project/file.rs` and the log entry. The DTO does not change.

## #4213 - D5: picks follow automatically at re-import, with a review line that holds the post

**Decided.** When a `.sc` model imports again, a face or patch pick that the geometry hash no longer finds follows the face with the same SolidCraft names, in the update's own undo step (`commands/update_model_follow.rs`). Each operation with a followed pick gets one `needs_review` line, for example: `face 'F2:side:b' of 'part' followed by its SolidCraft name after the model was imported again`.

- The follow runs in `UpdateModel::with_source_faces`, the builder step that brings the new record, not in `UpdateModel::new`. `new` only snapshots the old record and the operations that hold a face or patch pick on the model, because `with_source_faces` has no world.
- It runs only when the old model and the new mesh both hold a record that tiles their facets.
- The operations are snapshotted whole at construction and written back by `invert` (the `SetOperationGeometry` pattern). Undo puts back the ids and the review lines; redo follows again.
- The operation goes Stale, as before, because its picks changed.
- A follow does not run the pick-time gates of `SetOperationGeometry` (area, heights, sketch, "Do not touch" overlap). The followed pick names the same SolidCraft face. Generation stays the backstop for the shape of the pick (the area gate's own comment: "Generate stays the backstop"), and the review line makes a person look before the post. Not checked: that generation refuses every case that each pick-time gate refuses.

**What a review line does (checked).** `posting::review_problems` refuses the post while an unsuppressed operation has a `needs_review` line: "needs review before posting (...) - inspect the operation and mark it reviewed". `cam.verify` reports such an operation as holding the post (`script/api.rs::holds_post`). `MarkOperationReviewed` (MCP `mark_operation_reviewed`, the operation page's button) clears it, as one undo step. So a person checks each follow before metal is cut.

**This is the maintainer's call (Q2).** The other form offers the follows as a step that the user or the agent accepts.

**Why.** `revision.rs` says re-picking is "made visible to the user, never assumed". A SolidCraft name is SolidCraft's identity for the face, not a guess. The follow is one undo step and visible, and the review line keeps the human check before the post.

**Considered and dropped.**
- A one-click "accept the follows" step. It costs a new command and its MCP tool, and the user sees the same facts.
- No review line. Then a wrong follow could reach the post without a person.

**Cost to change later.** The review line is one function (`review_line`). An accept step would reuse `Follow::plan` and `Follow::write` behind a new command.

## #4213 - D6: a pick that cannot follow is named at re-import, and the pick format does not change

**Decided.** Each face or patch pick that could not follow stays as it was, so generation refuses it exactly as before. It is named in the re-import report, with its operation, its SolidCraft names and the reason:

- the window: `UpdateModel::summary()` (the window's status after a re-import, and `ImportFixture::update_summary()` for a fixture's re-import) gives "N picks followed by SolidCraft name; M picks could not follow: face 'top' in 'Face1' (no face has these names now); ...". It names the first 8 lost picks and counts the rest, and the phrase is capped at 900 bytes;
- `cam.import` and MCP `import_model`: the optional key `picks`, `{ "followed": [...], "lost": [...] }`, one row each `{ "operation", "pick", "names", "why"? }`. It names every pick. A first import, and an update where nothing followed or was lost, have no `picks` key, so the answer is as before.

**Not named:** a pick that was already missing before this import (it has no old face to name), and chain and vertex picks (sub-issue 4).

**Why.** Once the record is replaced, nothing holds the old name, so the refusal at generate cannot give it. To keep the name on the pick is a pick format change, which Marc's rule avoids and which the maintainer decides.

**Considered and dropped.**
- Keep the old name on a pick that failed to follow (a pick format change).
- Name every lost pick in the window. A large model can lose hundreds; MCP has them all.

**Cost to change later.** The report is `FollowReport` (`describe`, `to_json`). the `changes` key of #4214 sits beside `picks` in the same answer.

## #4213 - D7: which pick stores follow

**Decided.** The follow covers every pick stored inside an operation. The list comes from the project files that save picks (`commands/update_model_follow/stores.rs`):

| Store | Saved by |
|---|---|
| `Operation::src`, `SrcRef::Selection` | `project/file.rs` |
| `AreaControls::extra`, `BoundaryShape::Picks` | `project/area.rs` |
| `AreaControls::open`, `OpenBoundary::picks` | `project/area.rs` |
| `AreaControls::do_not_touch` | `project/area.rs` |
| `OpParams::Deburr::exclude_edges` | `project/params.rs` |
| `OpParams::MultiAxisDeburr::exclude_edges` | `project/multiaxis_deburr_dto.rs` |

- Only `Face` and `Patch` picks on the re-imported model follow. The deburr stores hold chains only, so the walk visits them but nothing in them follows yet.
- The sinker, probe and ruled-wire stores hold coordinates, not feature ids. Heights read the selection and hold no picks.
- The walk takes `Operation` and `AreaControls` apart field by field, with no `..`. A new field there does not build until someone says whether it holds picks.
- A test checks that the walk reaches each store.

**Not followed:** setup WCS and alignment picks, fixture-locate picks and electrodes. They behave as today.

**Not followed either: the picks of a Probe and of a Slot.** Their pick writes numbers into the parameters when it is made (`probe_from_picks`, `slot_from_picks`), and generation keeps those numbers: a Probe never reads its pick again, and a Slot only checks that its typed slot still fits the picked room. A followed pick would keep the old face's numbers against the moved face. So their picks stay as they were and are reported lost with the reason "its pick set the operation's numbers - pick it again"; a new pick seeds the numbers again (`stores::seeds_from_picks`).

**Why.** These are the picks that an operation's generation reads. A "Do not touch" pick that misses its follow is still refused at generate (`operation/avoid.rs`), so a miss fails safe.

**Cost to change later.** Each new store is one arm in `stores.rs`. Setup and fixture picks need their own snapshot in the same command.

## #4213 - D8: a follow happens only when the names leave no doubt

**Decided.** The match is in `camcraft_geometry::source_follow` (pure: features and records in, an id or a reason out). A pick follows only when all of these are true:

- its id was in the old revision and is not in the new one (a carried pick is untouched);
- exactly one old feature has that id, and no other old feature of the same kind has the same names;
- every name of the old feature is `verified` (it follows exact history in SolidCraft);
- exactly one new feature of the same kind (face or patch) has the same names. A new record face's `former` name counts as its name, so the pieces of a Split Face still answer to the face they came from. A face keeps its plane kind;
- that new feature was not in the old revision, and its names are verified too;
- that new feature answers to the names of no other old feature of the same kind (through a `former` name).

Anything else gives a reason, and the pick stays as it was.

The two revisions' features are built from their facets in `update_model_follow.rs` (`feature::features_of_facets`), never read from the features caches. Those caches key a mesh by an order-free content digest (`feature::digest_of`), so a cache hit can carry the facet order of another mesh with the same triangles. The record names facets by index, so those features would give the wrong names to the faces. The ids are the same either way, because they hash positions, not indexes. Pinned by `model_link::facet_order`. The cost is two feature builds at a re-import that has a pick to follow.

**Why.** A wrong follow is the worst failure here, and a missed follow is only today's behaviour. The rules beyond the plan close the cases where a follow could merge two picks into one face (two old faces with one name set, or a new face whose name is one old face's and whose `former` name is another's), or move a pick to a face that was already in the model under other names.

**Considered and dropped.**
- Name equality on the new side only (the plan's first form). Two old strips of one curved face would then both follow to the same new face.
- Read `former` as the only name of a new face. A project made after a Split Face would then lose every follow at the next re-import.

**Cost to change later.** One function, `source_follow::follow`, with its table tests beside it.

## #4213 - D9: where the code lives

**Decided.**
- The match is in `crates/geometry/src/source_follow.rs`: it reads no world, project file, command, MCP or CLI, so by the crate law it is not app code.
- The command half (snapshot, plan, write, report) is in the new `crates/app/src/commands/update_model_follow.rs` and its `stores.rs`.
- `update_model.rs` gets only hooks: one field, one line in `new`, one line in `with_source_faces`, one line each in `apply` and `invert`, and the two getters `follow_report` and `summary`.
- Nothing is added to `operation/mod.rs`: the path-kernel side branch changes it.
- The callers change by one line each: the window's status (`state.rs`) reads `summary()`, and `v_import` adds `picks`.

**Why.** Small hooks in the hub files keep the merges routine while the path kernel and the project-file PRs are open.

**Cost to change later.** None beyond moving the hook lines.

## #4213 - D10: a review line is cleaned and cut so the project file always loads

**Decided.** A review line has no control characters (each is a space), each SolidCraft name in it is cut to 60 bytes, and the whole line is cut to 240 bytes on a character boundary.

**Why.** `project/validate.rs` refuses the whole project as corrupt for a `needs_review` line over 240 bytes or with a control character. The names come from a file. An uncut line would save, then make the project refuse to open: data loss.

**Cost to change later.** The two constants in `update_model_follow.rs`.
