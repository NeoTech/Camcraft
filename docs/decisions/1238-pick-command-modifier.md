## #1238 - pick add-to-selection uses the platform command modifier

**Decided:** the pick add key (edge pick toggle, feature-pick add, boundary
pick toggle) reads egui's `modifiers.command` - Cmd on macOS, Ctrl elsewhere -
the same key the keyboard shortcuts use. `interact::Mods.ctrl` is renamed to
`Mods.command` and built from `egui::Modifiers` via `From`. Shift keeps its
meaning unchanged (add in feature modes, whole loop in Edge mode, close in
boundary mode).

**Why:** on macOS Control-click is the system's secondary (right) click, so a
Mac user could not add an edge chain to an operation's picks. On Windows and
Linux egui sets `command` whenever Ctrl is held, so their behaviour is
unchanged.

**Considered and dropped:**
- Accepting Ctrl *or* Command on macOS. Kept Ctrl out of the add key there so
  the Mac gesture is one key, matching the shortcuts; a Control press now
  replaces the selection instead (pinned).
- Changing the UI hint strings to the per-platform key. Out of scope: #1239.

**Cost to change later:** low. The conversion is one `From` impl in
`crates/ui/src/interact.rs`; flipping the add key back means editing that
mapping and the pinning test.

**Not exercised:** no Mac was used. The macOS behaviour rests on egui's
`command` mapping and is untested on hardware; the Windows/Linux path is
pinned by the same tests.
