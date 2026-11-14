## #1431 - browser build: Save, Save As and Post download; the rest refused by name

- **Save, Save As and Post download the same bytes the desktop writes.**
  Chosen: the app layer gains `project::save_text` and
  `posting::post_setup_unwritten` (the same collect / gate / engine run,
  the program held in memory); the desktop's `save` and `post_setup` now
  call them and write the result, so the two cannot drift. The UI hands
  the bytes to the browser through a `Blob`, `URL.createObjectURL` and a
  hidden `a[download]` click (`crates/ui/src/browser_files.rs`), with
  `eframe::web_sys` and our own two `#[wasm_bindgen] extern` bindings for
  `URL` - no new dependency (web-sys has no `Url` feature in eframe's set,
  and an `inline_js` snippet is not copied by `web/build-app-wasm.sh`).
  Considered and dropped: the File System Access API
  (`showSaveFilePicker`) - Chromium only, and async, which the
  synchronous command arm cannot await.
- **Save As asks for the name with `window.prompt`.** A download has no
  folder to choose; the name is the only thing Save As can still change.
  The answer is cleaned (`browser_files::file_name`: last segment, no
  reserved characters, the extension added). Dropped: an egui text
  dialog - more UI for the same one field.
- **A downloaded document is marked clean.** The download is the save;
  the browser cannot tell us whether the user then kept the file, the
  same as a desktop save to a folder the user later deletes.
- **Everything else is refused by name and darkened, not implemented.**
  Opening (model, project, fixture, machine, post, theme, tool library,
  tool body) needs the browser's async file picker, which PR #1314 (a
  contributor's fork branch) is building; this PR does not stack on a
  fork branch and does not touch that code. Recent files, the fixture
  library folder, the per-operation folder post and the other exports
  (setup sheet, tool list, stock STL, tool library) are refused with the
  reason as the status line and, where the control is a button or a
  ribbon cell, as its greyed tooltip. Context-menu and theme-dropdown
  rows refuse on the status line only (their row builders have no
  disabled state; adding one was larger than this fix). When #1314 lands,
  its open commands come off `browser_files::refusal`.
- **Not stacked on #1314.** Its branch lives in a fork; stacking would tie
  this fix's merge to a contributor's draft. The two PRs touch different
  arms (open vs. save/post), so they merge in either order.
