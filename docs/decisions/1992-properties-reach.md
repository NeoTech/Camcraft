## #1992 - Properties pane reach at 1280 x 720

**What was decided.**
- The STEP facet budget row puts its label on its own line, with the field
  and `Set` on the line under it. On one row it was about 15 px wider than
  the Properties pane at 1280 x 720, and `Set` stood past the right edge
  (x 1252-1279 against a pane edge at 1264).
- The setup Tool list table keeps its sideways scroll (6c0c62c61) and gets
  a gutter under the grid: the scroll bar width and its two margins. The
  floating scroll bar's hover strip covered the last row, so a centre click
  on `Edit` after a scroll across did not reach the button.
- `Delete operation` already fits the pane on main. The Operation page
  layout test now names it, so a change that pushes it out is red.
- Each case is pinned at the harness size 1280 x 720: `Set` and
  `Delete operation` as anchors of `layout_page_model` and
  `layout_page_operation`, and the Edit click by
  `a_centre_click_on_edit_opens_the_tool_after_a_scroll_across`, which
  scrolls with wheel events and clicks the button's centre.

**Why.** A control that a user cannot see or click is a defect of the page,
whatever the AccessKit tree says. The issue asked to fix each row that the
user cannot reach.

**Considered and dropped.**
- A narrower budget field (`desired_width` 110 to 90). It saves 20 px at
  this width only, and a larger font or a narrower pane breaks it again.
- Solid (not floating) scroll bars for the table. That changes the scroll
  style of one table against all the others in the pane.
- A gutter on the time and travel readout table too. It has no buttons,
  so nothing there needs a click.

**Cost to change later.** Small: one layout line and one spacer.
