## #3609 - Open a project read-only

**What was decided.**

- `camcraft-ui --read-only job.camproj` opens the project read-only. The
  option name and its behaviour match SolidCraft's `--read-only`
  (SolidCraft #20830), so users of the two apps get the same result. No
  SolidCraft code or text is used.
  - It applies to the one project file on the command line.
  - A model file is imported as usual: an import makes a new project with
    no file. With no file, the option does nothing.
  - A headless start ignores the option, because it never saves.
- One rule refuses a save over the file that a project is open read-only
  from: `project::refuse_read_only_save`, in
  `crates/app/src/project/read_only.rs`. It is in the project module with
  `project::save` and its other refusals (a missing folder, a file that
  this process cannot write). All save routes call it:
  - the window (`State::save_project_path`);
  - the MCP tool `save_project` and the script call `cam.save`, through
    `script::api::v_save`;
  - the live MCP seam of a window, through the same `v_save`.

  Each caller gains one line for the check and one line that ends
  read-only after a good save to a different file.
- The read-only state is kept next to the document path that it protects.
  The window keeps it in `State::project_read_only`, next to
  `project_path`. A headless MCP session or a script run keeps it in
  `Env::doc_read_only`, next to `doc_path`. The live seam copies the
  window's state into its `Env` through `LiveOptions::read_only`.
- The rule compares files, not spellings. Two paths are the same file
  when they are equal, or when their canonical paths are equal. The Save
  As dialog gives an absolute path for a file that was opened by a
  relative path. A compare of spellings would let Save As write over the
  opened file.
- In the window, Save on a read-only project opens the Save As dialog,
  and the status line tells why. The Save of the unsaved-changes prompt
  does the same. A save to the opened file from the dialog is refused on
  the status line, and the file stays as it was. CamCraft's editor for
  shipped machines works the same way (`machine::draft::MachineDraft`:
  Save is refused, and Save As writes a copy).
- Changes are allowed. They are commands on the history, and the title
  shows the unsaved mark as usual. The shipped-machine editor also allows
  changes.
- The headless MCP tool `open_project` takes `read_only: true`. Its reply
  then has `"read_only": true`. A later `open_project` without the flag
  ends read-only. A value that is not true or false is refused by name
  before the open, so the document does not change. A script gets the
  same through `cam.call("open_project", {path, read_only: true})`.
- The title bar shows `CamCraft - job.camproj (read-only)` until a save
  to a different file.

**Why.** Reference copies on a shared folder, the shipped examples and
tester kits, and released jobs must open with no risk that Save writes
over them. No project read-only state existed. The read-only text in
`crates/ui/src` and `crates/app/src/project` is about other things, for
example the AI chat, the shipped machines, the machine and post lists of
a setup folder, and the work-offset row of the heights page.

**Considered and dropped.**

- Keep the read-only state in the world or in the project file: the
  document path is not there either, and read-only is a fact of one
  open, not of the project. A key in the file would also need a format
  step.
- Put the check inside `project::save`: that function does not know
  which file the document came from. The callers know it.
- A module of its own outside `project`: the rule is about where a
  project file may be saved, so it belongs with `project::save`.
- Block each change in a read-only project: each command would need a
  check. The file on disk is what needs protection.
- Refuse Save in the window and let the user find Save As: the Save of
  the unsaved-changes prompt could then only fail.
- A new option `cam.load(path, {readOnly})`: it changes the script
  header and bindings. `cam.call("open_project", ...)` already gives
  scripts the open.
- Let the live seam open a project read-only: the live seam refuses
  `open_project`, because an open from an agent would skip the window's
  unsaved-changes prompt. That stays.
- Set the read-only bit of the file: it changes the file for other
  programs and users. A project file that this process cannot write is
  already refused by `project::save`; that refusal is separate.
- A hand-off of the option to a running window: CamCraft has no
  single-instance hand-off.

**Known limits.**

- A live `save_project` with a new path writes that file, but the window
  keeps its own path and its read-only state. The window does not follow
  the path of a live save today.
- A project that is opened again from the recent list opens for writing.
  The recent list keeps paths only.
- `get_job` does not report read-only.
- CamCraft has no autosave or recovery save, so no other route writes the
  project file.

**Cost to change later.** Low. The rule is one function in
`crates/app/src/project/read_only.rs`. The window part is in
`crates/ui/src/state_read_only.rs`. The tests in
`crates/app/tests/read_only_open.rs`, `crates/ui/src/launch_file_tests.rs`
and `crates/ui/src/cli_options_tests.rs` pin it.
