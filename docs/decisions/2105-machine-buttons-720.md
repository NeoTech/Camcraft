## #2105 - the machine page action row stands above the visible bottom at 720 px

**What was decided.** Two steps.

1. The "Refresh" button moves from its own prop_grid row onto the
   "Library" combo row. The row is built right-to-left, so the combo
   cannot push the button out of the cell. That change takes one row off
   the page.
2. A review of step 1 found it was not enough, and that the pin test
   measured the wrong edge. The page's visible bottom is not the bottom
   of the Properties leaf: `pages.rs` gives the page's ScrollArea the
   leaf height minus 40 px, and draws the separator and the `Close`
   footer in that bottom band. The pin test now measures each button's
   bottom against the top of `Close`. The page also reclaims height in
   three places:
   - its rows are drawn with a 2 px vertical gap of its own (`ROW_GAP`),
     where the shell draws 3 px;
   - the machine overview readout uses a 2 px row gap (`OVERVIEW_GAP`).
     `machine_grid` takes the gap as a parameter, and the machine editor
     header keeps its 4 px;
   - the two prose lines of the `ownership` block (the "where does the
     pick land" line and the one-machine-one-post line) are drawn small
     and dimmed, like the page's own help line. Each wrapped to three
     lines of 15 px in the 274 px pane; each now takes two lines of 10
     px. The words and the AccessKit labels are unchanged.

The page's three action buttons are "Load other", "Edit" and "Clear".
An earlier version of this file named one of them "New"; no such button
exists on the page.

**Measured.** In `shell_harness()` at 1280x720 with the Tormach 770MX
loaded: the Properties leaf spans y = 104..682, and the `Close` footer in
that pane starts at y = 648 - the page's body must end above it. Before
step 2 the action row spanned y = 644..668, below that edge: its bottom
stood 20 px under the top of `Close` (the review measured the page's
scroll area ending at about y = 642, so the row started outside it).
After step 2 the row spans y = 570..594: 54 px above the top of `Close`.

**Why.** The ribbon grew 22 px in #2027, and the machine page's action
row fell under the fold of a 720 px window: the user had to scroll to
reach "Load other", "Edit" and "Clear". The page is tall for its pane -
three titled groups, four rows of prose and radios, eight overview rows -
and the pane is 274 px wide, so the long lines wrap. One stacked row was
not enough; the page needs its own vertical rhythm and its prose lines
drawn smaller. No label, no command and no project field changed.

**Considered and dropped.**
- A smaller item spacing for every page: drops, it moves a shared value
  and changes every page. The page takes a gap of its own instead.
- Putting "Refresh" with the action buttons in one row: drops, the four
  buttons do not fit the pane's width.
- Reducing the page heading, or dropping the separator under it: drops,
  every other setup page keeps both, and the prose rows were the taller
  loss.
- Making the panel scroll-by-default: drops, the pin test says the
  buttons must be visible without scrolling.

**Cost to change later.** Low. The pin test
`the_machine_page_buttons_fit_without_scrolling_at_720_px` and the
`layout_page_setup_machine` anchors encode the Library row and the action
row; any new machine-file button must join one of those rows, and a
taller row on this page must pay for it from the same three gaps.
