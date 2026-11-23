## #3608 - Ask or refuse before a save replaces a changed project file

**What was decided.**

- A project remembers the disk stamp of its file: the path, the size and
  the modification time. `project::DiskStamp` takes it when the window,
  `cam.load` or the MCP tool `open_project` opens the file, and after each
  save. The stamp is taken before the open reads the file. A write during
  the load then gives a question at the next save, not a lost change.
- A save to the file of the stamp takes the stamp again first. When the
  size or the time is different, the file on disk changed after the open
  or the last save.
  - The window holds the save and asks: Save As, Cancel or Overwrite.
    Nothing is written before the answer.
  - `cam.save` and the MCP tool `save_project` refuse by name. The
    refusal names the file and the option `overwrite: true`.
    `save_project` has a new boolean argument `overwrite`. `cam.save`
    has a new options object, `cam.save(path?, { overwrite?: boolean })`.
- A save to another path is not checked. A file that is gone is not
  checked: the save puts it back and removes no other work. A new
  document has no stamp, so its first save is not checked.
- The stamp of our own save replaces the old stamp, so the next save
  does not ask. The stamp is taken after the save returns `Ok`, so it is
  the stamp of the renamed temp file (#3509). The write probe of
  `project::save` opens the old file for writing, writes nothing and is
  done before the rename, so it does not change the new stamp. The check
  runs before `project::save`, so a changed file is refused before the
  probe and the temp file.
- In the window, a new document and an open drop the old stamp and a held
  save. A held save can then never write a new document over the old
  file (for example after Ctrl+N while the prompt is open).
- An agent attached to the window uses the window's stamp. The live seam
  takes the stamp in (`LiveOptions::doc_stamp`) and gives the stamp after
  the call back (`LiveAnswer::doc_stamp`). The window keeps it when it is
  the stamp of the window's file. Without this, an approved agent save
  made the window ask about our own write at its next save.
- The stamp is in the `project` module, beside `project::save` and
  `project::load`. It describes the same file that they write and read,
  and each caller of the guard (the script verbs, the MCP tools, the live
  seam and the window) already uses `project` for that file.
- The checks are in their own functions: `script::save_guard::check` for
  scripts and MCP, and `State::may_save_over` for the window.
  `project::save` does not change.
- The behaviour matches the overwrite guard of SolidCraft's `file.save`.
  The texts are CamCraft's own.

**Why.** Before, a save over the project's own path wrote without a check.
A second window, a `git pull` or a branch switch, a colleague's save in a
shared folder, or a sync client could change the file while it was open.
The next save then removed that work and gave no message.

**Considered and dropped.**

- A content digest (SHA-256) in the stamp: each save must read the whole
  file again before it writes, and each open must hash the text. A project
  can be more than 100 MB. A write by git, a sync client, another window
  or a copy changes the time or the size (general knowledge). The
  linked-model watch uses the same pair.
- A check of the digest only when the size or the time changed, to stop a
  question after a `touch` or a copy that keeps the bytes: it needs the
  digest of the open and the save, so `project::load_noted` and
  `project::save` must give their text. A question that was not necessary
  is safe, and the user or the agent answers it.
- The check inside `project::save`: its signature changes for the tests
  and all callers, and the window must ask, not refuse.
- Keep the held New, Open or Quit of the unsaved-changes prompt and go on
  after Overwrite: two prompts must then share one held action. The held
  action stops, the document stays open, and the status line says so. The
  user does the action again after the save.
- A top-level module for the stamp: the stamp has no use apart from the
  project file, and its readers and writers are in `project`.
- Compare paths after `canonicalize`: it fails for a file that is gone
  (general Rust knowledge), and the "file last opened or saved" rule
  compares the path as given.
- Leave the attached agent out: its save would not be checked, and the
  window would ask about our own write at the next save.

**Known limits.**

- A write that keeps the size and the modification time is not seen. Some
  file systems keep the time in large steps (general knowledge: FAT32 2
  seconds, HFS+ 1 second), so a second write with the same size in the
  same step is not seen.
- A write between our rename and our stamp is not seen. The window is
  one `metadata` call.
- The path is compared as given. Another spelling of the same file (a
  relative path, other letter case on Windows, a symbolic link) is
  treated as another path.
- A program that changes only the modification time (`touch`, a copy that
  keeps the bytes) gives a question that was not necessary.
- A save over another existing file (not the document's file) is not
  checked by this change. The stamp is about the document's own file.
- The browser build saves by download and has no files, so it has no
  stamp.

**Cost to change later.** Low. The stamp is in `project/disk_stamp.rs`.
The checks are `script::save_guard::check` and `ui::save_guard`. The tests
in `crates/app/tests/save_overwrite_guard.rs` and
`crates/ui/src/save_guard_tests.rs` pin the refusal, the overwrite option
and the save after our own save.
