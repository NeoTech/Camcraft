## #1089 - G-code window: selectable text, cursor highlight, click to trace

**Decided**
- **The cursor stays the scroll's middle row; a click moves the scroll.**
  The viewer's one cursor law (the middle row is the published line, which
  drives the amber segment) is kept. Clicking a row (anywhere on it) asks
  the scroll to centre that row, and the cursor then reads it back. No
  second stored "picked line", so playback/cursor stay immediate-mode state
  and no command is involved.
- **The cursor row gets a background band** (theme selection fill at 45%),
  drawn in both the plain and the trace view.
- **Line numbers are a separate, non-selectable label**; the code is
  selectable. Drag-select and Ctrl+C copy the G-code without the gutter
  numbers (pasting numbered lines into a machine would be wrong). The
  copy path is egui's own label selection; no custom right-click menu.
- **Window width:** the trace view used egui's `Sides`, which gave the
  listing an unbounded width in an auto-sized window (5000+ px in a
  headless test). Replaced with two panes in fixed rects cut from the
  window's own free rect; the window keeps its size.
- **Trace default is unchanged.** `PostView::trace` is created `false`
  (`post_file.rs`); what the reporter saw as "tracing on" is the amber
  viewport segment of the listing's cursor, which is always shown while the
  viewer is open (M1.7d). Intended, not flipped.

**Considered and dropped**
- Padding the scroll area so every row can reach the middle: it would
  change the cursor at scroll 0 and the existing cursor-law tests.
  Consequence kept: a click on a row within half a page of the very top or
  bottom cannot centre it, so the cursor lands on the nearest reachable row.
- A stored picked-line field on `PostView`.
- Clicking a line with no matching move: it still moves the cursor; the
  viewport segment is simply absent for it (`line_pos` is `None`), as for
  scrolling onto such a line.

**Cost to change later:** low - all in `crates/ui/src/egui_app/gcode_view.rs`.
