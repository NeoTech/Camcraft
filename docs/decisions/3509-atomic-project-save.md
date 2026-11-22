## #3509 - Save a project through a temp file and a rename

**What was decided.**

- `project::save` writes through `machine::store::write_atomic`: a temp
  file in the same folder, flushed, then renamed over the target. The
  window, the MCP tool `save_project` and the script call `cam.save` all
  use `project::save`, so one change covers them.
- `project::save` keeps two refusals of the plain write. A folder that
  does not exist is refused by name, because the shared writer creates
  missing folders. A project file that this process cannot write is refused
  by name, because on Linux and macOS a rename replaces any file that the
  folder allows.
- The write test of the project file is a write-open of the target, not
  only its mode (review of PR #3510). On Unix a file that another user
  owns can have a write bit (mode 0644) and still refuse a write; a rename
  would replace it, because the rename needs write permission on the
  folder only. The probe opens the target for writing, with no truncate
  and no create, and refuses on error. A file that the process cannot
  write is kept.
- `machine::store::temp_path_for` gives the temp path. `write_atomic`
  uses it, and the tests use it to make a save fail. A path that names
  no file is now refused by name before a folder is made. Before, the
  temp file took the name "machine" and the rename failed.
- The temp name has the process ID and a number for the calling thread
  (review of PR #3510). Before, it had the process ID only, so two saves
  of one file at the same moment (an autosave and a save by the user)
  wrote the same temp file. Each thread takes its number one time, from
  one counter. The saves of one thread run one after another, so its name
  can repeat.

**Why.** A plain `std::fs::write` truncates the old file first. A write
that stops (a full disk, a lost network share, a stopped app) left a
damaged project. Machine files, preferences, recent files and view
settings already used `write_atomic`.

**Considered and dropped.**

- Move `write_atomic` to a new module: it already has five callers in
  this crate, and a move gives no change in behaviour.
- Let the shared writer create a missing folder for a project, as it
  does for preferences: a typo in an agent's or script's path would make
  folders. That is a different change.
- Fail a save in the tests with read-only folders or open file handles:
  they act differently on Windows and Unix. A folder in the place of the
  temp file fails the same way on all systems.
- Merge the copy of the same steps in `fixture_library/store.rs`: a
  different change.
- A new number for each save (a counter in each name): a test could not
  know the name before the save, so it could not put a folder in its
  place. A number for each thread keeps two saves at the same moment
  apart and keeps the name known to the thread.
- A test with two threads that save one file many times: on Windows, two
  renames over one file at the same moment can fail for a reason that is
  not the temp name, so the test could fail at random. The test checks
  the names that two threads get.

**Known limits.**

- The new file gets the permissions, owner and ACL of a new file in that
  folder, not those of the old file. Machine files and preferences act
  the same way.
- A project path that is a symbolic link becomes a plain file.
- A project file that this process can write, in a folder where it cannot
  write, is now refused: the temp file cannot be made. The plain write
  succeeded. The old project stays whole.
- The folder is not flushed after the rename. Power loss directly after a
  save can keep the old file. It cannot leave a part-written file.
- On Windows, the rename fails when a program holds the old file open
  without delete sharing. The save then reports the error and keeps the
  old file.

**Cost to change later.** Low. The checks are in
`project::wire::write_project`, and the four tests in
`crates/app/tests/project_roundtrip.rs` pin them.
