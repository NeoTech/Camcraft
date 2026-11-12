# #1095: law (d)'s lateral rapids re-read against the stock's top under them

- **Chosen:** `verify::acquit_by_stock_top` (#1091, PR #1098) now also re-reads an upright LATERAL rapid convicted against the stock's highest point. Each band of the stack (flute, shank, holder) is read over the capsule it sweeps along the move, rather than a dive's disc, against the highest vertex of any stock or part facet whose plan comes within its radius. The tip is taken at the rapid's lowest end for the whole move.
  - Why: the same upper bound as the dive (for a closed shell, the topmost surface over a point lies on a facet covering it), applied along a segment.
  - Judging a climbing or falling traverse as if it ran level at its low end can only over-report.
  - Like the dive, it only removes convictions, and it fails closed on an open shell, a non-finite end, a posed rapid or a column-free mode.
- **Chosen:** one code path for both. A dive is the capsule of a zero-length segment, and `dist2_seg_tri` of a point is exactly `dist2_tri`, so the dive's verdicts are unchanged.
- **Changed pin:** #1098's `only_upright_column_dives_are_re_read` asserted that a lateral rapid is never re-read. That was this issue's gap. Its diagonal dive over the low step is now acquitted; posed, it stays convicted.
- **Left as is (#1098's design):** a stock-sourced op with no judged stock solid (the Drill, Face, Pocket family from `SrcRef::Stock`) is not re-read. Its column law reads the op's region in the stock's place, and #1098's wiring returns before any re-reading. The link fixture is therefore a part-sourced drill under a from-model casting, the issue's own case.
- **Dropped:** re-marching the lateral rapid's footprint stations against the solids. The capsule bound is exact for "no facet within reach" and costs one pass over the facets per convicted rapid.
