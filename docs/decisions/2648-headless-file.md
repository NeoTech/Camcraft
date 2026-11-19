## #2648 - A headless start opens the file named on the command line

**What was decided.**

- `camcraft-ui --headless <file>` takes the same file list as the window
  (`cli_options::files`) and the same check (`launch_file::command`). A
  refused file (two files, an extension CamCraft does not read, a path that
  is not a file) is said by name and the run exits with 2, the code of a
  bad argument. No adapter is touched.
- A file that passes the check is opened through the same command as the
  window (`ImportFile` or `OpenProjectFile`) on a new state, in place of the
  seeded proof document (`crates/ui/src/headless_open.rs`). The run waits
  for the import (`State::settle`), frames the document (`FrameAll`) and
  draws it with the same View3D readback.
- The run fails (exit 1) and names the file when it does not open: a model
  file that gives no model geometry, or a project whose open is refused.
  The pixel check (at least 5% of the frame drawn) applies when the opened
  document has part geometry; a project without a part opens and draws
  without it.
- The last line of the run says what was drawn: `opened part.sc: 1
  model(s), N facets`, or the seeded document as before.
- `--help` and `docs/user-guide/start-options.md` say what `--headless`
  does with a file.

**Why.** The review of #2542 found that `--headless part.sc` ignored the
file without a word (#2648). A silent ignore is not allowed: the run opens
the file or refuses it by name.

**Considered and dropped.**

- Refuse every file on a headless start: simpler, but it throws away a
  useful check - a script can now ask "does this file import and draw?"
  with no window and read the exit code.
- Open the file on top of the seeded document: the seed's own checks
  (Contour1's toolpath) would then pass whatever the file did.

**Cost to change later.** Low: one module and the headless entry; the
tests in `crates/ui/tests/egui_app.rs` pin the exit codes.
