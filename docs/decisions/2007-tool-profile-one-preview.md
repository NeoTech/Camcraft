## #2007 - the tool profile page: one preview, a full points table, plain words

**Decided.**

- The Profile tab has no side view of its own. The one to-scale preview
  beside the tool fields draws the draft of the tab. With Custom selected,
  a drawn holder that validates shows as the bands that the collision
  check sweeps (`tool_body::stack`, #2017). On Draw cutter, a drawn cutter
  that validates replaces the steel outline, and the holder goes on top
  of it as bands. A draft that does not validate shows the holder in use,
  and its refusal stays in red under the table. All other tabs show the
  holder in use, as before (`tool_profile_page::preview_inputs`).
- The points table has a least height equal to its greatest height
  (150 px). egui gives a nested scroll area only the space that is left
  in the view (egui 0.36.2 `scroll_area.rs`), so a table low on the
  scrolled page fell to its 64 px least height and showed about two rows.
  This cause was seen in the running app before the change.
- The points table has no 40 px least column width (egui's Grid default).
  With it the six columns were about 300 px wide in the 260 px column, so
  in a narrow pane the "×" buttons stood outside the pane. The tab row
  and the three file buttons wrap in a narrow column.
- The page says "In use: ..." in place of "landed: ...". The revolved note,
  the table headers ("height", "radius", "arc", with hover text) and the
  holder line ("from .. to .. mm above the tip") use plain words.

**Why.** The maintainer reopened #2007 with "the tool editing dialogs still
look bad". Two drawings of one tool at different scales showed the same
holder twice. The removed view gave the flute, stickout and gauge marks;
the holder line under the table now gives the stickout and the gauge.

**Dropped.**

- Hide the main preview while the Profile tab shows and keep the side
  view: then the tool has a different drawing on each tab, and the side
  view did not draw the bands that the check reads.
- Draw the Parametric draft when a body is in use: Parametric then shows
  a cylinder while the check still reads the landed body. The preview
  shows a draft only where Apply profile would land it.
- Remove the inner scroll area of the table: a long imported outline
  (tens of points) then pushes Apply profile far down the page.

**Cost to change.** Small. `preview_inputs` is the one place that picks
what the preview draws, and `TABLE_H` is one constant.
