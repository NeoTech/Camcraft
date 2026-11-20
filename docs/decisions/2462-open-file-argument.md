## #2462 - Open a model or project file that is given on the command line

**What was decided.**

- `camcraft-ui <file>` opens one file. A model file (`.stl`, `.obj`, `.step`,
  `.stp`, and a Solidcraft `.sc` on native builds) queues
  `CommandKind::ImportFile`. A `.camproj` file queues
  `CommandKind::OpenProjectFile`. These are the commands of the Model and
  Project dialogs and of a drop, so the worker, the update-or-spawn decision
  and the refusals are shared, not copied (`crates/ui/src/launch_file.rs`).
- The file arguments come from the same walk over the arguments as the
  option check (`cli_options::files`). Thus an option's value (`--gpu
  nvidia`) is never read as a file. A bare `-` is not a file. Arguments that
  `parse` refuses give no file.
- More than one file, an extension that CamCraft does not read, and a path
  that is not a file are refused by name on the status line
  (`open refused: ...`). The app then starts with an empty project. It does
  not exit: a user who double-clicks an unreadable file still gets the app.
- `--headless` ignores a file. The headless run is a self-check of the 3D
  view, not a document session. (Changed by #2648: a headless start now
  opens the file or refuses it by name; `2648-headless-file.md`.)
- `--help` shows `Usage: camcraft-ui [options] [file]` and a `File:` section.

**Why.** Solidcraft proposes a "Send to CamCraft" command (Solidcraft RFC
0001, phase 1; tangency-labs/solidcraft#18936) that starts CamCraft with the
design's path. The operating system's "Open with" needs the same argument.

**Considered and dropped.**

- Several files at once (import each model): the issue asks for one file,
  and a mix of a project and models has no clear meaning. A later issue can
  add it.
- A running window that takes the file from a second start (single
  instance): out of scope in the issue; it needs an IPC channel.
- macOS: `open -a CamCraft part.sc` delivers the file as an Apple Event, not
  as an argument. This change does not read Apple Events. Starting the
  binary inside the bundle with the path works.

**Cost to change later.** Small. The rule lives in `launch_file::command`
and `cli_options::scan`; the tests in `launch_file_tests.rs` pin it.
