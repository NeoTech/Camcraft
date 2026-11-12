## #1198 - Viewport bottom control bar (slice a): the right-edge view buttons move into a ribbon

**What was decided.** The view buttons (Frame, Iso, Shaded/Wireframe,
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
cube), not the look. The control legend at the bottom-right is not moved (it was never on the
ribbon's row: see below).

## Fix round: the ribbon moves to the bottom-left, not the centre

**Why.** The review found the centred ribbon (158 x 38 px, 10 px above the
bottom) sat on the control legend at the bottom-right: the legend is about
300 px wide at the Small text style, so any pane narrower than about 780 px
had them overlapping, and the legend is painted after the view furniture
only in the sense that the ribbon covered it. My first note's sentence
"will sit clear of it on normal pane widths" was wrong.

**What was decided.** The ribbon moves to the bottom-left corner (10 px in
from the pane's left edge, 10 px above the bottom). The legend keeps the
bottom-right corner, unchanged. Chosen over lifting the legend above the
ribbon, because lifting puts the hovered legend on top of the empty-document
line ("No model yet ...", 2-3 px above the ribbon's old row) on every pane.
Moving the ribbon is one function (`bar_rect`) and changes no other row.

**Where they still meet.** The two share the bottom strip only when the pane
is narrower than the ribbon plus the legend plus the margins, about 453 px
with six buttons (the legend measures 242.75 px at the Small text style on the test font;
the ribbon's right edge is at 198 px from the pane's left edge). Before the
#1204 merge the ribbon had five buttons and the threshold was about 423 px.
Below that the ribbon covers the legend's left end while the pointer is over
the viewport. Not fixed in this round: the next step, if wanted, is to lift
the legend above the ribbon only when the two would meet, which then sits
over the empty-document line on hover on those narrow panes.

**Pinned by.** `viewport_bar::tests::legend_clears_the_ribbon` (legend rect
from the shell's anchor and the Small text style, against the ribbon rect,
on a 700 x 600 and a 1600 x 900 pane) and the updated
`law_tests::the_hud_is_laid_out_from_the_pane_alone` (ribbon on the left
corner, on the bottom edge, buttons inside it).

**Empty-document line.** Unchanged, not made worse: the line is centred
at the pane's middle, 2-3 px above the ribbon's top at the bottom edge,
which is the same clearance as before since the ribbon's row did not move.

## Refresh against main 37482923: the Pick through button joins the bar

**Conflict.** #1204 (merged) added a sixth view button, the "Pick through"
toggle (`Nav::PickThrough`, #692), to the nav column in `viewcube.rs`, which
this PR moves into `viewport_bar.rs`.

**Resolution.** `viewcube.rs` keeps no nav column: our side was taken whole,
and main's additions were ported into `viewport_bar.rs`, unchanged in
behaviour: the `PickThrough` variant, the six-entry `NAVS` (sized by
`NAV_COUNT`), `pick_through_label` (moved here; `viewcube` re-exports it for
the tests only), the act arm (toggles `state.pick_through` and sets the
status line), the active state (lit while on, off by default) and the glyph
(a ray through a bar). No logic was changed on either side beyond the
button list.

**Ribbon width.** Six buttons: 6 x 26 + 5 x 4 + 2 x 6 = 188 px. The hud gate
is `viewcube::min_width()` (the wider of the cube and the ribbon, plus margins),
used by `Hud::new` and by the law test, so the test does not repeat the sum.

**Tests.** The four `pick_through` tests in `egui_app/tests/pick_through.rs`
are unchanged and find the button through `hud(...).nav_rect(...)`, which is
now the bottom bar. The layout tests count the buttons from `NAVS`, check
the last button is the last in the list, and check it sits inside the bar.

## Maintainer review, 2026-11-12: bottom-centre, the legend gives way

**The maintainer's words** (PR #1203 review, 2026-11-12): "No it should not be
bottom-left . it should have been bottom-center floating ontop of the 3D
viewer".

**Dropped.** The bottom-left placement from the previous fix round is
withdrawn. The ribbon is back at the bottom-centre of the pane, floating over
the 3D view, at the same margin above the bottom edge, with all six buttons.

**Layout rule (`viewport_layout::place`, pure on the pane, the cube and the
measured sizes; the shell and the tests read the same rects).**
1. The ribbon is centred on the bottom edge and has priority.
2. The control legend stays at the bottom-right while it clears the ribbon by
   a gap of 8 px; where it would meet the ribbon it sits on its own line
   directly above the ribbon, right-aligned.
3. The empty-document line ("No model yet ...") is centred at its base height.
   If it meets the ribbon it lifts above the ribbon; if the legend is drawn and
   it then meets the legend it lifts above the legend too, so it moves up by
   the legend's lift.
4. A piece that would meet the view cube, or leave the pane, is not drawn (the
   legend is a hover hint; the empty-document line is a hint too).

**Widths.** With the Small text style, the legend measures 242.75 px on one
line. The legend lifts above the ribbon below a pane width of about 714 px:
the legend's left edge is w - 254.75 and the ribbon's right edge is w/2 + 94,
so it lifts when w - 254.75 < w/2 + 94 + 8, i.e. w < 713.5. It wraps onto more lines below about 267 px (the legend's
width plus the 12 px inset on each side); the hud's minimum width is about
208 px, where the legend wraps and lifts. The empty-document line wraps by
the same rule, against its own measured width.

**Tests.** `viewport_layout::tests::legend_clears_the_ribbon` runs the rule on
three panes (1600 x 900, 700 x 600, and the hud minimum `viewcube::min_width()`
x 260), with and without a model, and asserts no pair of {ribbon, legend,
empty-document line, view cube} shares pixels and every piece is inside the
pane. At the wide pane the legend stays on the bottom line; at the minimum the
legend sits one gap above the ribbon. `viewport_bar::tests::bar_fits_its_buttons`
and `law_tests::view_cube_tests::the_hud_is_laid_out_from_the_pane_alone` pin
the centred ribbon and count the buttons from the list.

**Not exercised.** Nothing was looked at on screen. The rule is pinned by
rects measured with the frame's fonts, not by pixels; the wrapped multi-line
legend at the minimum width is not seen.
