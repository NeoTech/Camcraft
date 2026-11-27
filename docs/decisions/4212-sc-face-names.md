## #4212 - D1: SolidCraft face names are read from model cache format 10 only

**Decided.** The `.sc` reader reads the face names of the final body only when the cache manifest's `format` is `solidcraft-model-cache/10` (`camcraft_kernel::solidcraft::NAMES_FORMAT`). The names are `model.names[n]`, where `n` is the final model body's `names` field: `faces[i]` is the name of record face `i`, and `info[i]` is `[aliases, bounds, verified, synonyms]` with an optional fifth item, the former name of a Split Face piece. Only `faces`, `verified` and `former` are kept. A cache of another format (the format-8 sample) gives no names and no note, and imports exactly as before.

**Why.** The reader takes no layout from sample bytes (#1456). The layout we cite is SolidCraft's published source at v0.50.0: `crates/doc/src/model_cache.rs` writes and reads the names list, and `docs/design-container.md`, "Model cache", describes the cache. The face order comes from the same source: `Body::record` holds "faces, edges and vertices in the body's own order" (`crates/kernel/src/body.rs`), `encode_native_body` writes that record as it is (`crates/kernel/src/native.rs`, used by the cache writer in `crates/io/src/design/native_cache.rs`), and the names list has one name for each face of the body (`crates/doc/src/history_names.rs`). solidcore-record keeps a face's index as its face index everywhere (`brep.rs`, "Canonical order"). So `faces[i]` names the decoded record's face `i`, and the machining mesh's `tri_face` indexes the same record. The format-8 sample holds names in the same shape, but that shape would come from the sample's bytes. No note for format 8: a note on every format-8 import would change the status line and the MCP result of each one, and nothing uses the names in this PR.

**Considered and dropped.**
- Read format 8 too: the shape would be taken from sample bytes.
- A note for each cache that is not format 10: noise on every older design, a visible change for no use yet.

**Cost to change later.** One constant (a list of formats) and one pin in `solidcraft_import/names.rs`.

## #4212 - D2: The import never fails because of names

**Decided.** Names that do not fit give no names and one note, never a refusal. A misfit is: no names list, no or a bad index, a record without `faces` or `info`, a count of names or info items that is not the record's face count, a name that is not a text of 1 to `MAX_NAME_BYTES` (16 KiB) bytes, or an info item that is not four or five items of the right kinds. The note starts "Solidcraft: the face names were not read:" and says which. It goes with the import notes (the status line, the MCP result), only for a format-10 cache. In geometry, a record whose ranges do not tile the facets is dropped with the note "Solidcraft: the face names do not fit the mesh's facets; they were not kept". In the app, `source_faces::set` drops a record that does not tile the model's facets, with a logged warning.

**Why.** The body is the model; the names only help a later step to follow face picks. Without names, a `.sc` model behaves as today: picks are kept by geometry. A refusal would block files that import today. A partial or shifted list is worse than none: it tiles the facets and names the wrong faces, the one failure that would point a pick to the wrong face. So the list is taken whole or not at all. `MAX_NAME_BYTES` is our own bound: SolidCraft's names are short, and the manifest's own 32 MiB bound holds the total.

**Considered and dropped.**
- Refuse the import: blocks models that import today, for no safety gain.
- Keep the names that read and skip the rest: a gap shifts nothing only if the gap is marked, and a misread entry may mean the whole list is from another layout.

**Cost to change later.** Low: `names::read` returns the note, and the callers only carry it.

## #4212 - D3: The face names are held in memory only in this PR

**Decided.** A model imported from a `.sc` design carries `schema::SourceFaces { rev, len, record }`, written and read only through `source_faces::{set, of}`. It is not saved: the project file does not change, and a reopened project has no names. `of` gives the record only at the revision and facet count it was landed with, and only when its ranges tile the facets (the `SolidRanges` guard, #917). `ImportModel` lands it. `UpdateModel::with_source_faces` states it, and the update trades it on undo; an update that does not state it drops it, so `RepairModel` (through `keeping_fidelity`, which does not keep it), an update from a file that is not `.sc`, and the vise, accessory and plate rebuilds drop the names. `DeleteEntity` puts it back on undo. The window's re-import, the script and MCP `import_model`, and the fixture re-import state it.

**Why.** Saving is a project-file question with a maintainer decision (C13 Q1: a version step or not), and it belongs to the next PR. Held in memory, the work changes no saved byte and no behaviour. The revision guard makes the record safe against every landing of facets, also ones that do not go through these commands: `model::set_facets` moves the revision, so names never speak for facets they were not made for.

**Considered and dropped.**
- A plain component without the guard: a landing of facets outside the import commands (a command added later, or one not found in this search) would leave names on other facets.
- Saving now: it fixes the project-file answer before the maintainer gives it.

**Cost to change later.** The next PR adds `project/source_faces_dto.rs` and one call to `source_faces::set` in the loader, after the facets and the revision.
