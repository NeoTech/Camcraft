## #2463 - Linked model: import a model again when its source file changes

**What was decided.**

- **A model records its source** (`schema::ModelSource`: path and SHA-256,
  `crates/app/src/model_source.rs`). `ImportModel::within` records it after
  the parse; a mesh handed in and a browser pick record none. The record
  rides `UpdateModel` (traded with the facets, restored by undo; a repair
  that keeps the fidelity keeps it), `DeleteEntity` (delete and undo) and
  `ImportFixture`'s update. Generated meshes (vise, plate, accessory,
  profile fixture edits) update without a source, so they unlink: their
  facets are no longer the file's.
- **Project v72** (`project/source_dto.rs`): an optional `source` on a
  model row. Absent is not linked, so a v71 file steps up unchanged. A
  source on a row that is not a model, an empty path, a digest that is not
  64 lowercase hex digits, or an unknown key in it is a corrupt file. An
  older CamCraft refuses a v72 file by name (`unsupported project version
  72`), as for every version step: it would otherwise save a linked model
  back unlinked without a word.
- **The check** (`crates/ui/src/model_link_watch.rs`) runs in the app
  window about once a second (`INTERVAL`), and the frame timer comes back
  for it while a model is linked. A file is read again only when its size
  or modification time moved. When its SHA-256 is not the model's, the
  file is imported again through `CommandKind::ImportFile`; `finish_import`
  lands a file that a model is linked to on that model, whatever the file
  is called. The landing is the existing re-import: one `UpdateModel`, one
  undo step, the operations go Stale. Nothing regenerates or posts.
- **Undo stays.** The check remembers the digest it last imported for each
  model and does not import the same bytes again. An undo of the import
  thus stays until the file changes once more.
- **A missing or unreadable file** leaves the model as it is. The status
  line says it one time, by name; the check says nothing more until the
  file is readable again.
- **Unlink**: the Browser's model menu shows "Unlink from <file>" on a
  linked model (`CommandKind::UnlinkModel`, `SetModelSource`, one undo
  step). The MCP tool `set_model_link` links (to a path, recording its
  digest now, no import) or unlinks (`null`); `get_job` shows
  `models[].source`.
- **SHA-256** is the kernel's FIPS 180-4 implementation, re-exported as
  `camcraft_geometry::solidcraft::sha256_hex` (native only). No new crate
  dependency: geometry already depends on the kernel on native builds. A
  browser build records no link: it has no file to check again.

**Why.** Solidcraft proposes "Send to CamCraft" (tangency-labs/solidcraft
#18936, Solidcraft RFC 0001 phases 1 and 3). Each save in Solidcraft writes
the `.sc` file again; with this, CamCraft updates the part after each save.

**Considered and dropped.**

- An OS file-watch crate (`notify`): a new dependency, and the once-a-second
  check of size and time costs nothing while the files do not change.
- Hashing on the import worker: the STL importer streams the file, so the
  digest is a second read after the parse. A file that changes between the
  two reads records the newer digest; the next change imports it again.
- Regenerating Stale operations after an update: refused by the issue. The
  user regenerates and posts.
- Import on open: a project opened while a linked file changed is updated
  by the first check, the same way, and says so on the status line.

**Cost to change later.** Moderate: the project key is a format step (v72).
The check's rules live in one module and its tests pin them.
