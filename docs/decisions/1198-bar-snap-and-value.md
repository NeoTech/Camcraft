## #1198 - Viewport bar snapping and typed values (slice d)

**What was decided.**
- **Expand the bottom overlay bar upward only when Move or Rotate mode is
  active.** When neither handle mode is on, the bar remains the 12-button
  strip (`bar_rect`, 368 x 38 px), keeping the default shell layout and pixel
  snapshots unchanged. When Move or Rotate is on, the same rounded grey
  background extends upward by one compact control strip (`panel_rect`,
  368 x 66 px) hosting the axis selector (`X`, `Y`, `Z`), the typed distance
  or angle field (`TextEdit` + `Set` / Enter), and the snap step parameter.
  `viewcube::Hud::contains` covers the full expanded bar so clicks and drags
  on those widgets never start viewport orbit or picking, and
  `viewport_layout::furniture_with_bar` lifts the legend and empty-document
  line above the expanded height.
- **Hold Shift to snap; defaults are 1.0 mm (Move) and 90.0 degrees
  (Rotate).** `BodyDrag` and `TurnDrag` track the unsnapped raw drag
  (`raw_delta`, `raw_angle`) separately from the active previewed/released
  value (`delta`, `angle`) so pressing or releasing Shift mid-drag toggles
  snapping smoothly without accumulation drift. Both snap parameters are
  positive bounded fields in the bar (`0.001..=10_000.0` mm through the active
  unit lens, `0.1..=360.0` degrees), and `DEFAULT_ROTATE_SNAP_DEG = 90.0`
  matches the 90-degree requirement in #1198 while remaining editable in the
  bar for finer angles such as 15 or 45 degrees.
- **Typed entry applies through the existing drag commands.** Typing a
  distance or angle in the bar and pressing Enter or clicking `Set` converts
  from the active display unit (`units::from_display`) or degrees to radians,
  refuses non-finite / missing-selection inputs by name, skips zero-length
  deltas with no history step, and queues `CommandKind::MoveModelBy` or
  `CommandKind::TurnModelBy` — so undo, staleness, following stock and
  aligned-setup frames work identically to dragging a handle.

**Why.**
- Keeping the control strip hidden while Move and Rotate are off avoids
  unrequested bar clutter in normal navigation and avoids changing the
  default shell width (`viewcube::min_width`) or default pixel goldens.
- Using `Shift` for handle snapping avoids colliding with `Ctrl`/`Cmd`
  shortcuts and `Alt` window-manager drags on Linux.
- Reusing `MoveModelBy` / `TurnModelBy` preserves one command path (`MoveModel`)
  and requires no new app command or MCP tool.

**Considered and dropped.**
- **Always showing snap and value widgets in the main bar:** dropped because
  the inputs are mode-specific (mm/in distance vs degree angle) and widening
  the base bar would crowd narrow panes and shift every resting viewport
  golden.
- **Persisting snap steps in per-user `prefs.json`:** dropped to match the
  other viewport-bar toggles (`move_handles`, `rotate_handles`, `pick_through`),
  which are session view state in `crates/ui` and require no app preference
  format changes.
- **Using `DragValue` instead of `TextEdit` for the typed move/turn input:**
  dropped because `DragValue` fires `changed()` on intermediate keystrokes
  and silently discards invalid text, whereas `TextEdit` + `Set` / `Enter`
  commits exactly one command on submit and refuses non-finite text by name.

**Cost to change later.**
Low and local to `crates/ui/src/egui_app/viewport_bar_controls.rs`,
`crates/ui/src/move_drag.rs` and `crates/ui/src/turn_drag.rs`. Changing the
default snap steps, adding prefs persistence, or moving the controls onto a
combined move+rotate gizmo (#2624) touches only those UI fields and helpers.
