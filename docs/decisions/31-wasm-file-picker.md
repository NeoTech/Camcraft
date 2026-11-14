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

### Safari: the chooser opens inside the user's click (2026-11-13)

**Chosen.** The user's hand check found File > Open model doing nothing
in Safari (Chrome and Brave passed). The web shell drained the queued
Open action at the start of the *next* frame, which eframe runs from
`requestAnimationFrame`; Safari opens a file chooser only while the
user's input event is being handled, and ignores `input.click()`
elsewhere without an error. eframe 0.36 already runs the app's frame
inside its `pointerup` listener (its own comment: Safari allows certain
things only "on user action, such as a click"), so the fix runs the queue
at the end of that frame whenever it holds an action that may open a
chooser (Open model, Open project, the unsaved-changes answer that lets a
held Open project go on): `State::run_click_actions`. The hidden input is
now attached to the page with `display:none` (MDN's "Using files from web
applications" pattern) and removed when replaced. A chooser asked for
outside a click (the Cmd/Ctrl+O shortcut: eframe's keydown listener
queues the key for a later frame) still tries, which works in Chrome and
Firefox, and the status line says to click File > Model / File > Open if
no chooser appeared, never a silent no-op. Merging #1431 from main, the
browser refusals of Open model / Open project (which waited for this
picker) are lifted.

**Considered and dropped.**
- An extra "Choose file…" button or overlay for every browser: a second
  click in Chrome, where one already works.
- Sniffing the user agent for Safari: brittle, and the click path now
  works the same in every browser.
- `navigator.userActivation.isActive` or `window.event` to detect a lost
  activation: the former reports a timed activation that WebKit's file
  input may still refuse, the latter is deprecated (MDN).
- A keydown listener of our own for Cmd/Ctrl+O: it would open the
  chooser twice unless deduplicated against egui's own shortcut; the
  status line covers the rare shortcut case.

**Sources.** MDN, "User activation" (transient activation, activation-
triggering events); MDN, "Using files from web applications" (a hidden
input's `click()` from a click handler); MDN, `Window.event`
(deprecated); the eframe 0.36.2 source, `web/events.rs`
(`install_pointerup` runs `runner.logic()` in the listener).

**Cost to change later.** Low: the click-time drain is one call in the
web shell and one method on `State`.
