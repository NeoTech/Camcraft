## #2339 - Save and restore panel setups

**What was decided**

- **Several named setups.** A saved panel setup is one named arrangement.
  The user types a name, saves the arrangement under it, and gets a row per
  saved setup with `Load` and `Delete`. One setup alone could not keep the
  maintainer's "setups" plural, and a name is what makes the list a list.
- **Stored in the application settings, not the project file.** The setups
  live in `panel-setups.json` in the per-user config folder, beside the
  theme and colour-vision preferences (`camcraft_app::prefs`). A setup is
  the screen's, not the part's: it survives New and Open, and it never
  marks the document changed. A file of its own (not a key inside
  `prefs.json`) was chosen so a corrupt or older setups file can never take
  the theme down with it, and because `prefs.json`'s `camcraft-prefs/1`
  tag would have had to change.
- **Auto-load is on by default, and the user can turn it off.** The last
  setup that the user saved or loaded is the one that comes back at the
  next start. The window has one tick box for this. A corrupt saved setup
  at startup is refused by name on the status line and the default
  three-pane layout stands.
- **`Reset to default` stays the View door's existing `Reset layout`
  command**, and the window carries a second door to it. No second reset
  command was added.
- **A load is not undoable.** It is view state, like the dock itself: the
  dock lives beside `State`, never in the world, and the history holds
  commands on entities. A saved-setup load rides the same one-frame
  mailbox that the View door's toggles use (`PanelSetupAsk`), so Ctrl+Z
  still undoes the last document edit and never a panel move. A test pins
  that no command is queued and the history stays empty.
- **The arrangement is the dock's own JSON.** egui_dock's `serde` feature
  is now on (a feature of an existing pinned dependency, not a new
  dependency), and `Tab` derives `Serialize`/`Deserialize`. This keeps
  exactly what the user can arrange by hand - split sides and fractions,
  leaf sizes, tab order, the active tab, collapsed leaves and float rects -
  which a hand-written summary of the default three panes would have lost.
- **The file carries its own version tag** (`camcraft-panel-setups/1`). An
  older or unknown tag is an empty store with a diagnostic, and the dock
  starts as the default. A setup whose dock this build cannot read is
  dropped at the load, and a load that fails puts the default layout back.
- **A saved dock is validated against the permanence law**: exactly one
  viewport, docked in the main area, and a main area with a root. A dock
  that breaks it is refused by name, never handed to egui_dock's asserting
  `split`.
- **Unlaid-out rects are saved as a placeholder.** An infinite `Rect`
  (`Rect::NOTHING`, a fresh float or split) becomes `null` in JSON and
  cannot be read back; `layout::setups::encode` replaces it with a
  320x240 placeholder, which the next frame recomputes.

**Why**

The maintainer asked: "we should be able to 'Save' panel setups what is
opened, where they are attached and how." A saved arrangement that lost
its float position or its split size would not be the arrangement the user
made. The command law stops at the document: the dock is deliberately not
in the world, so a save or load is a mailbox ask, not a history entry.

**Considered and dropped**

- One unnamed setup only. Dropped: the plural was asked for, and a list of
  names makes "load the one I use for roughing" possible.
- Storing the arrangement in the project file. Dropped: the issue's open
  point recommends application settings, and a per-part dock would move
  when a part is opened.
- A hand-written arrangement summary (open panels, a side, a size). Dropped:
  it cannot keep a user's own multi-split dock, so a load would rearrange
  their layout.
- Saving the arrangement automatically on every dock change. Dropped: the
  user asked for a save, and a silent rewrite of the file would make the
  saved setup move under them.
- Undoable saves and loads. Dropped: the dock is not document state, and
  Ctrl+Z after a panel drag is not the expected law here (the existing
  `layout` module's decision).

**What it would cost to change later**

- A different file format (for example our own summary instead of the
  dock's JSON) is a format tag bump: older files then load as the default
  with a diagnostic, which is already the tested path.
- A named setup as a project file field would contradict the store law
  above and would need a new project-file version.
- Making a load undoable would need a command that carries the whole dock
  as its payload; nothing in the current design blocks it, but the history
  would then hold view state.
