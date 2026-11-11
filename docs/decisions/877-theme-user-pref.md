## #877 - Theme and colour-vision mode are user preferences, not project state

**What was decided.**

- The theme and the colour-vision mode are kept per user in `prefs.json`, our own `camcraft-prefs/1` format, in the same config folder as the recent-files list (#662: `%APPDATA%\camcraft`, `~/Library/Application Support/camcraft`, `$XDG_CONFIG_HOME/camcraft`, or `CAMCRAFT_CONFIG_DIR`). The new `app::prefs` module reuses `recent::config_dir_for` and the atomic `write_atomic`. The file is read once at startup by the real window only, and written at every pick. Headless States (every test) have no store and never touch the user's file.
- The reporter decided this on the issue: the theme has nothing to do with the project, and the colour-vision mode is saved as a user preference too and survives restarts.
- The project file no longer writes a `theme` key, and opening ignores one. Serde already ignores unknown keys in `ProjectFile`, so older files with `"theme": {...}` open cleanly. No project-file version bump: an older build reading a new file finds no key and wears Dark, which is what it always did for a key-less file. File > New no longer resets the theme to Dark.
- `SetTheme` and `SetColorVision` are no longer undoable commands. A pick validates (the existing JSON pre-parse plus `commands::check_theme`, or the colour-vision gate), sets the world singleton and writes the file. Ctrl+Z does not undo it, and it is not an unsaved change (`history.is_dirty()` never sees it). This matches the recent-files list, which is also preference state outside the history. AGENTS.md's "Everything Is a Command" covers document state; it already exempts ephemeral view state, and a user preference is not part of the document.
- A pick no longer bumps `doc_version`. That counter invalidates document caches and marks a refused post as out of date, and a theme change does neither. The overlay colours read the singleton every frame, so nothing needed the bump.
- Loading the file:
  - A missing file is a quiet default start.
  - A corrupt or foreign-format file starts at the defaults, with the reason on the status line.
  - An unknown built-in theme name or colour-vision word falls back for that preference alone and is named; the other preference still loads.
  - A theme FILE reference is kept as written even when the file is gone (a network drive may come back). `sync_theme` refuses it by name and keeps the current look, as before.
  - A stored colour-vision mode is gated against the theme it lands on, as a pick is. One that fails stays Off and is named.

**Why.** A theme or colour-vision mode is about the person at the screen. Saving it in the project meant every new project opened Dark, a colleague's project opened in its author's theme, and changing the look marked the document unsaved and sat on its undo stack.

**Considered and dropped.**

- Keeping the project key as a fallback for when no preference exists. The reporter ruled that the project has nothing to do with the theme, and a fallback would make the look depend on which file was opened first.
- Keeping `SetTheme` as an undoable command while also storing the preference. Undo would then rewrite the user's preference file from a document's history, which ties the two back together.
- Storing the preferences in `recent.json`. That file has its own format tag and purpose; a separate file keeps each format small, and one going corrupt does not take the other with it.
- Bumping the project-file version. Nothing about reading changes for old files, and new files simply lack an optional key.

**Tests.** The two tests that pinned the old design are removed: `commands_undo::theme_choice_undoes_redoes_and_refuses` and `commands_undo::color_vision_is_an_undoable_view_preference`. Their refusal and project-file checks moved to `view_prefs.rs`. `project_roundtrip::the_theme_choice_round_trips_and_a_v40_file_means_dark` is replaced by `the_project_file_never_carries_the_theme_and_an_old_theme_key_is_ignored`. Three ui tests that undid a pick now assert that undo leaves it.

**Cost to change later.** Small. The store is `app::prefs` (one file), the wiring is `ui::state_prefs`, and the project side is a removed key. Adding another view preference is one field in `ViewPrefs`.
