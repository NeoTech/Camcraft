# #3788: Fill Rings and Clean Joins Where Two Lobe Offsets Merge

Date: 2026-11-25
Status: Accepted (unattended worker decision)

## Context

Issue #3788 fixes full-width cuts where offset rings of two lobes merge,
such as the waist of `fig8`.
PR #3790 added one-level (`RING = 4`) gap fills and `level_flanks::full_width_at`.
On `upstream/main` (`baad348f1`), four defects remained:

- On `fig8` Outside, 15 full-width points remained (5 points per Z-level across
  3 Z-levels).
- On `fig8` Inside, one segment still tripped the narrow check at
  `(35.8393, 159.0963)`.
- On `edge_route`, links could walk along an uncut `WALL` hole loop or jump
  to a wall-chord point across unmachined flanks.
- On synthetic two-lobe pockets, coarse `RING = 4` fills stopped above 1/16 step
  cusps (`x_c` jumps faster than `0.9 * tool_r` per step).

Root causes found:

1. Clipper 1-unit lattice steps (`1e-6` mm to `1.7e-5` mm) flipped local segment
   tangents inside rings and `WALL` loops.
2. Sub-band fills stopped at half-step and quarter-step (`RING = 4`) and only
   checked `gap` sweeps, not whether `full_width_flanks` still fired in Inside or
   Outside cut order.
3. Short cusp fills clipped to `zone.offset(tool_r)` could span less than the
   2.5-radius own-trail exclusion window (`OWN_LAG`), leaving two adjacent passes
   inside one exclusion pass.
4. Boundary `edge_route` walks did not reject chords whose two flanks (`0.9 * r`)
   cut uncut stock.

## Decision

1. **Collapse interior micro-steps before `share_ends`**:
   `ring_fill::clean_walk` removes interior steps shorter than `MICRO_SEG = 5e-5`
   mm (`< SNAP = 1e-4` mm) from rings, fills, `WALL` walks, and core loops. Open
   endpoints stay fixed so `share_ends` keeps exact boundary joins.
2. **Recursive binary sub-levels up to `RING = 16`**:
   `ring_fill::split_band` divides any band between `lo_lev` and `hi_lev` down to
   `1 / 16` step where `gap` (or `ring \ swept` at `depth >= 1`) or
   `merge_flank_hits` finds an uncovered zone or full-width flank in either
   Inside or Outside cut order (`CUT_MARGIN = 0.95`, `HIT_PAD = 0.2`).
3. **Dual reach and boundary tail extension in `level`**:
   Fills triggered only by `gap` use `zone.offset(tool_r)` so endpoints outside a
   ring stay reachable by `edge_route` without lifting. Fills where
   `merge_flank_hits` fired use `zone.offset(2.0 * tool_r)` so cusp fills exceed
   `OWN_LAG` and run smoothly into already-cut stock. `tail_extension` extends any
   open end within `3.5 * tool_r` of `band.c`'s outer boundary to that boundary.
4. **Full-width guard on boundary `edge_route` chords**:
   `level_order::edge_route` and `ring_links::chord_full_width` reject candidate
   boundary chords whose left and right `0.9 * r` flanks both lie in uncut
   stock, and `level_order::join` pushes the full multi-vertex `via` walk.

## Alternatives Rejected

- **Intersecting `depth == 0` `gap()` with `stock`**: Breaks the top air bridge
  on `fig8` Outside (`Y = 175.02`) and adds 3 extra lifts. Rejected.
- **Using `2.0 * tool_r` reach on all `gap()` fills**: Extends `disc` and `dog_bone`
  gap endpoints behind the next ring inside `c`, adding 3 lifts on `disc Inside`
  and lifts on `dog_bone Inside`. Rejected.
- **Disabling `fit_arcs` on small chords globally**: Violates
  `offset_rings_across_one_micron_vertex_step_post_crossing_free`. Rejected.

## Readings (Before -> After)

- `fig8 Outside` (`offset_pass_start`): 15 full-width points -> 0 full-width
  points; 0 narrow notes; 3 lifts (unchanged); 0 crossings.
- `fig8 Inside` (`offset_pass_start`): 10530 wall / 0 off-wall points / 1 narrow
  note -> 10293 wall / 0 off-wall points / 0 narrow notes; 0 lifts; 0 crossings.
- `two_lobe_waist` (`one_ring_set::two_lobe_ring_merge_cuts_no_full_width_at_waist`):
  Outside 70 off-wall points and 5 narrow notes -> 0 off-wall points and 0
  narrow notes for both Outside and Inside.

## Provenance

Clean-room implementation from contour-parallel distance level sets (Held, *On
the Computational Geometry of Pocket Machining*, Springer LNCS 500, 1991) and
the project's own flank and keep-down laws (`level_flanks`, `ring_links`). No
vendor software or documentation was consulted.
