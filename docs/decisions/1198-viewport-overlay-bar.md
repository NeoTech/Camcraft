## #1198 - Viewport bottom control bar (slice a): the right-edge view buttons move into a ribbon

**What was decided.** The five view buttons (Frame, Iso, Shaded/Wireframe,
Toolpaths, World axes) that stacked under the view cube move into a
horizontal, rounded, translucent grey ribbon floating over the bottom edge
of the 3D pane, centred. The view cube stays top-right. The ribbon and its
glyph painting live in the new `crates/ui/src/egui_app/viewport_bar.rs`, so
`viewcube.rs` (already over 400 lines) shrinks rather than grows. The
buttons keep their effects exactly (Frame is the queued `frame_all`
command; the toggles are direct view-state writes, as before).

**Why.** The maintainer asked for one floating control bar at the bottom of
the viewport that also takes the right-edge buttons (#1198). Slice (a) is
the bar and the move only; the move/rotate modes and the drag handles are
slices (b)-(d) and are not in this PR.

**Layout gate.** The pane now needs room for the ribbon's width (about 158
px) plus margins, so the hud gate became `max(cube, ribbon) + 2 * margin`
wide. A pane between roughly 128 and 178 px wide loses its cube and buttons
where it used to keep them. Chosen over squeezing the ribbon, because the
ribbon is the one place the buttons can be reached.

**Considered and dropped.**
- Keeping the buttons on the right edge and only restyling them: does not
  meet "move the buttons to the bottom ribbon".
- A real egui widget bar (`egui::Area`/`Frame` with buttons): would add
  accesskit labels for these five buttons and change the hit path the
  pointer law (`shell.rs`, `viewcube::click`) depends on. Left for a later
  slice if the maintainer wants labelled buttons; the painted buttons keep
  the existing click law unchanged.
- Making the ribbon opaque: a translucent grey (alpha 160 over the scene)
  was picked so the scene still reads under it. Not checked on screen.

**Cost to change later.** Low. Layout is one function (`bar_rect`,
`buttons`); the gate constant is `BAR_W`.

**Not exercised.** Nothing was looked at on screen: the ribbon colour and
its contrast against light and dark grounds are unverified. The structural
test pins geometry (centred, bottom edge, buttons inside, no overlap with the
cube), not the look. The control hint at the bottom-right is not moved; the
centred ribbon will sit clear of it on normal pane widths.
