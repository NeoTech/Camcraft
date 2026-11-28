## #4484 - One command table and an in-app menu bar

**Decided.**

- `crates/ui/src/egui_app/panes/table.rs` is the one command table. Each
  fixed command has one row: id, label, ribbon group, menu and shortcut.
  The ribbon's anchor groups (File, Setup, Modify, Inspect, View), the
  menu bar and the keyboard layer all read it. The hover text, the enabled
  rule and the icon stay functions of `Cmd` in `cmd.rs` and `gates.rs`,
  because they read the document state each frame.
- The operation kinds and the section doors are not table rows. They come
  from the operation registry (`camcraft_app::operation`), as before. The
  menu bar's Operation menu has one submenu for each section that the
  machine can run, with every kind of that section (pinned kinds too).
- The menus are File, Edit, Setup, Operation, Inspect and View. They
  follow the ribbon groups; Edit holds the history and clipboard rows.
  There is no Help menu: no command of the shell belongs there yet.
- Four commands that had only a key now have a `Cmd` and a menu row:
  Copy, Paste, Quit and Toggle projection. They are not on the ribbon.
  So the menu lists the ribbon's ids plus these four, and the test checks
  exactly that.
- A menu row and a key press use `run_direct`: the ribbon's `run_cmd`,
  except that Model and Open go straight to their file dialogs (the keys
  did this before). The ribbon's recent-files lists hang under their own
  cells, so the menu does not open them.
- A disabled menu row says why on hover. `disabled_reason` adds short
  reasons for the gates that had none (Undo, Redo, Run, Copy, Paste,
  Delete, Simulate, Deviate). The Create rows keep their refusal tips. The
  ribbon tips do not change.
- The menu bar is a top panel over the ribbon, `menu_bar_h` high (one
  control row). The ribbon itself is unchanged; it moves down by that
  height.
- The top-level menu buttons have the AccessKit name "<title> menu"
  ("View menu"). The visible titles share words with other buttons (the
  browser's View door, the tool page's Edit), and the name keeps each
  label unique for screen readers and tests.

**Dropped.**

- A native macOS menu bar: it needs a new dependency (maintainer decision,
  #4245 follow-up).
- Labels, tips and icons as table columns: they depend on the state, so a
  column would hold a closure for each row and save nothing.
- Help and Tools menus with no commands in them.

**Cost to change later.** A new fixed command is one table row plus its
arms in `tip_of`, `enabled`, `icon_of` and `run_cmd` (the compiler names
each). A menu move is one field of the row.
