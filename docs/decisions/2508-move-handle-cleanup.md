## #2508 - Move handle cleanup

**Chosen:** Release the centre cache's facet `Arc` and cached centre when
`handles` finds no draggable part and Rotate has no eligible selected part.
Keep the cumulative `computes` counter. Keep the cache while Rotate uses
the selected, visible part.

**Why:** An inactive handle mode must not keep a deleted part's mesh alive.
Move and Rotate share this cache. Clearing it during every Rotate frame
would make Rotate walk the facets again each frame.

**Dropped:** Clear the whole cache on every failed Move query. That would
discard the counter and break Rotate's cache reuse. Replace the retained
facet reference with a weak reference. That would change the cache key's lifetime guarantee
without a need for that change.

**Chosen:** Move the viewport glyph drawing into
`viewport_bar_glyphs.rs`. Keep the bar layout and button actions in
`viewport_bar.rs`. Keep every existing test. Draw an original Move glyph
with a small rectangular part and three arms with round tips. Keep Pan's
four arrows.

**Why:** The split follows the drawing seam and keeps both source files
below about 400 lines. The part and three arms distinguish Move from Pan
at the button's size. The glyph needs no imported asset or licence change.

**Dropped:** Keep the four-arm Move cross. Its outline remains too close
to Pan. Import another icon set. The original drawing meets this need
with less work.

**Chosen:** Defer the split and CRLF conversion of `shell.rs`. Preserve
its bytes in this patch. Convert its line ends in a separate commit when
few open PRs change it. Split it along its domain seams after that.

**Why:** At the overlap check, PRs #2256, #2149 and #1798 change this file.
PR #1798 has `needs-maintainer`, so this issue must not stack on it.
A whole-file split or line-end conversion would increase those conflicts.
The issue permits a recorded decision for each item. At the start of this
patch, `shell.rs` has 1087 lines and CRLF line ends.

**Dropped:** Split or normalize the shell now. This would mix broad
mechanical changes with active work in the same file.

**Chosen:** Keep the crate-wide formatting check unused for this patch.
Format the new glyph module. Do not reformat unrelated code.

**Why:** A crate-wide formatting commit would include extensive unrelated
changes and conflict with active UI PRs. The focused diff is easier to
check. A future formatting change can run when the UI queue is quiet.

**Dropped:** Format the whole crate in this patch. The formatting changes
do not prove cache release or improve the Move glyph.

**Cost to change later:** The cache policy and glyph are local changes.
The deferred shell split needs a focused patch that preserves its tests.
The CRLF conversion needs its own commit. A crate-wide formatting change
needs coordination with open UI PRs and a fresh gate run.
