## #31 - an async file picker for the browser build (Open model, Open project)

**Decided.** In the browser (wasm32), File > Open model and Open project
open an `<input type=file>` (`crates/ui/src/file_pick.rs`) and return at
once. When the user has chosen, the file's name and bytes go into an
inbox. `State::drain_actions` takes them on the next frame and runs each
one as a `CommandKind::OpenPicked`. A model lands through
`ImportModel::from_bytes` and `finish_import`, so it is one undo step,
and a same-named model is an update, as before. A project lands through
`project::open_noted`. A failed read, a refused file or a non-text
project is said on the status line, using the same "import refused:" and
"open failed:" wording as a file opened from disk. Native builds keep
rfd's synchronous dialogs, and their inbox is never filled.

**Why.**
- Bytes are what a browser hands over, so the app takes bytes.
  `ImportModel::from_bytes` shares the path import's checks
  (`checked_budget`: IGES and unknown extensions, STEP budget bounds),
  plus the file-size cap and the finite-mesh gate. A picked file lands
  exactly as the same file from disk; `import_bytes` pins this.
- The inbox is plain Rust on every target, so everything after the bytes
  arrive is tested natively. Only the DOM input and the awaited read are
  wasm32.
- The picker uses `eframe::web_sys` and `wasm_bindgen` features that
  eframe already enables (`HtmlInputElement`, `File`, `FileList`,
  `Blob`). No dependency line was added.
- The read is capped at 256 MiB (`BROWSER_READ_LIMIT`) and refused by
  name before it starts. A wasm32 module has at most 4 GiB of memory, an
  import needs several times its file, and an allocation that fails in
  wasm aborts the tab.
- The `<input>` and its change handler are held in one slot until the
  next pick replaces them, rather than `Closure::forget`. A cancelled
  picker never fires, so a forgotten closure would leak once per cancel.
  The slot holds at most one picker.

**Considered and dropped.**
- rfd's `AsyncFileDialog` on wasm32: it is already a native dependency,
  but its web backend adds its own overlay and button, and it would take
  rfd onto the wasm32 dependency list.
- Writing the bytes into a virtual file system so that `import_path`
  could open them: more shim, and nothing gained over a bytes entry
  point.
- Running the bytes import on a worker thread: wasm32-unknown-unknown has
  no threads. The import runs inline, which is what the path import
  already falls back to when a spawn fails.

**Owed (still on #31).**
- Save and Save As, and Post, in the browser. These need a download, not
  a picker. A project opened from bytes has no path, so Save asks where,
  and in a browser that answer is "cancelled".
- The other pickers (fixture import, machine, post, theme, tool library)
  still answer "cancelled" on wasm32.
- Dropped files: eframe hands over dropped files' bytes on the web.
- A browser hand check of the picker, which was not run here.

**Cost to change later.** Low. Other pickers join by adding a `PickFor`
variant and a route in `open_picked`.
