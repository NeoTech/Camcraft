## #4493 - The Settings window (Appearance, Graphics, Graphics device, Updates)

**Decided.**

- The window is an in-app `egui::Window` (`settings_window.rs`), the same
  law as the tool shape window. A section list at the left, the rows of
  the chosen section at the right. Only one section shows at a time, so
  each row label stays unique for screen readers and tests.
- The ribbon top band's Settings button opens and closes the window. The
  one-row dropdown is gone. The window opening is view state
  (`State::settings_open`), not a command, as the doors are.
- The menu bar row is `file.settings`, "Settings…", in the File menu above
  Quit. The decision file of #4484 has no Help or Tools menu, and File is
  the desktop place for application settings on Windows and Linux. The
  row has no ribbon cell (`ribbon: None`): the top band's Settings button
  is the ribbon's entry, and a second cell for the same window would
  only take space. No shortcut: no key is free by convention on all three
  systems without a check of every key handler.
- Appearance: the theme and the colour vision as two combo boxes. A pick
  queues the same `SelectTheme` / `SelectColorVision` that the Theme door
  queues. The theme row is named "Colour theme", because the ribbon's
  Theme door already has the name "Theme". The Theme door stays as the
  quick switch; it does not get a row that opens this section in this PR.
- Graphics: the line width and the anti-aliasing, with the same `State`
  setters as before (`select_line_width`, `select_antialias`, which
  refuses a sample count that the device does not offer). They left the
  Display menu.
- The Display menu's group heading "Graphics" is now "View options". It
  keeps the grid, the origin axes, the projection, the quality presets and
  the shading switches: these are part of normal viewing (answer 3 on
  #3025), and #4493 moves only the line width and the anti-aliasing.
- Graphics device: read-only rows for the adapter and the backend, read
  from the startup GPU line (`gpu::GpuInfo::log_line`), and the sample
  counts that the device offers. Headless, both say "Not known". This
  section has no "Reset to defaults": it holds no setting.
- Updates: "Check for updates" as a `check_row`; off hides a shown update
  notice, as the dropdown row did.
- "Reset to defaults" uses the defaults of the file types
  (`prefs::ViewPrefs::default()`, `LineWidth::default()`,
  `Antialias::default()`, check for updates on).
- The per-user file formats do not change, so an old `prefs.json` and
  `view.json` load with the same values.

**Dropped.**

- A ribbon cell for Settings: the top band button is already there.
- Moving the whole Display "Graphics" group: the grid, axes and
  projection are quick viewing toggles.
- A "Settings…" row in the Theme door: optional on #3025; it can come with
  F2.
- An MCP tool or a CLI flag: none now (answer 4 on #3025).

**Cost to change later.** A new section is one `Section` variant, its
label and one draw function. A menu move of the row is one field in
`panes/table.rs`.
