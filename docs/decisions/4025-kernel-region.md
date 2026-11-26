## #4025 - Path kernel K1: one region model for selection, material side and bounds

Stage 1 of the K0 record (`## #4024` in `RECOMMENDED.md`, "Stage 1:
Region"). The kernel builds one region value with `region(sel, material,
bounds, plane, tool, leave)`. The motion adapter
(`crates/motion/src/machining2d/kernel_region.rs`) turns the present region
inputs into the kernel inputs. No old planner calls the adapter. With the
planner switch at its default, the output does not change.

**What was decided**

1. **One selection type for all picks.** An edge, a face, a chain and
   points that straight lines join all become a `Chain` (a polyline, open
   or closed). A `Selection` holds one or more chains. Nested closed chains
   are holes. A chain is refused when it has a point that is not finite,
   too few points, two neighbour points at the same place, all its points
   on one line (closed), or a self-crossing. A selection is refused when it
   is empty, or when two chains cross or touch. The crossing test is a
   sweep over the x-sorted segments. Dropped: a selection enum with one
   variant for each pick kind (the region law is the same for each kind);
   a union of crossing chains (the meaning is not clear, so the kernel
   refuses it by name).
2. **The meaning of `MaterialSide`.** `Inside` cuts the area that the
   closed chains enclose (a pocket, a bore, Contour "Inside"). `Outside`
   cuts the stock round that area (Contour "Outside", the band round a
   part). The carriers map so: Contour `Side::Inside` and `Side::Outside`
   map to the same names; `Side::Auto` maps to no side (the caller resolves
   Auto first); the pocket family is `Inside`; `StartSide::around_part`
   with the stock known is `Outside` of the part (the part is the stock
   less the band). `ChainSide` and `Containment` of the area controls are
   not mapped in K1: they act on open boundaries in `camcraft-app`, and the
   kernel refuses open chains (decision 3).
3. **An open chain is refused** (`OPEN_NEEDS_SIDE`): it encloses no area,
   so it has no inside or outside. This is the record's refusal "a material
   side that the selection cannot have (an open chain with no side)". The
   "on the line" region of Trace, Engrave and 2D Chamfer is the work of
   #3758. `KernelRegion` is `#[non_exhaustive]`, so that step can add it.
4. **The Outside centre reaches the stock outline.** The centre is the
   stock less the selection grown by the tool radius plus the stock to
   leave. A loop of the centre whose vertices are all on the stock outline
   (within 1 um) is a stock edge. Each other loop is a part wall, also a
   loop with parts of the two (the conservative reading: the tool must not
   cross it). The air is the band between the stock grown by one tool
   radius and grown by two radii, less the keep-out: there the disc does
   not touch the stock. Today the 2D Offset band erodes the whole band, so
   its centre stays the tool radius plus the stock to leave inside the
   stock outline. The pin
   `the_offset_band_keeps_off_the_stock_edge_and_the_kernel_reaches_it`
   names this difference. The part walls agree within the arc tolerance
   of the polygon offset: the band erodes the part's hole and the kernel
   grows the part, which gives two chord sets of one arc (4.8 um apart on
   the pin's part, the offset's arc tolerance is 5 um). Dropped: erode the stock
   outline by the radius (the disc then leaves the convex stock corners
   uncut, at the radius times the square root of 2); erode it by the radius
   plus the stock to leave (that also leaves a skin of stock at an edge
   that has no part).
   **Decided by the maintainer** (question dialog, 2026-11-26): "Yes, to
   the edge". For the material side Outside, the tool centre can go to
   the stock outline. The cutter overhangs the stock by up to one tool
   radius. A clamp at the stock edge needs a keep-out.
5. **The Inside centre is the exact erosion.** The centre is the selection
   eroded by the tool radius plus the stock to leave. It is not clipped to
   the stock, and it has no air: it is a closed area. The kernel keeps a
   very small piece of centre. Today the Pocket path drops a piece of 0.01
   mm2 or less (#2232); Contour Inside keeps it. The pin
   `the_pocket_drops_a_sliver_that_contour_and_the_kernel_keep` names this
   difference. The guide stage's completeness guard (0.1 mm2, record stage
   2) decides what is cut. Dropped: the sliver filter in stage 1 (a region
   is a pure geometric value; a filter there hides area from the guard).
6. **The keep-out is a part wall.** The centre keeps the tool radius from
   the keep-out (not the stock to leave: that applies to the part). Its
   loops are part walls.
7. **The stock to leave** is finite and greater than minus the tool radius
   (`BAD_LEAVE`). A negative value is accepted above that limit, as for
   an undersize finish.
8. **The bounds refusals.** `Outside` needs the stock (`NEEDS_STOCK`). A
   selection that does not meet a known stock is refused on each side
   (`OFF_STOCK`). An `Outside` selection that covers the stock has no
   material (`EMPTY_MATERIAL`). A tool that does not fit, or a keep-out
   that covers the centre, gives `EMPTY_CENTER`.
9. **A region is planned in the plane of its recipe.** `PlanarRegion`
   holds its `WorkPlane`, and `region` takes it. The 2D coordinates are
   along the plane's own axes. The pin
   `a_region_is_planned_in_the_plane_of_its_recipe_not_in_xy` builds a
   region in the lathe's Z-X plane.
10. **`PlanarRegion::new` checks the partition and the orientation.** Each
    loop of the centre is exactly one part wall or one stock edge, in the
    same direction (any start vertex). The loops of the centre are oriented
    by their nesting. The orientation check is a guard: `camcraft-geometry`
    gives only normalized regions, so no public value can break it, and no
    negative test can reach it. `region` makes the centre normal again when
    the guard finds a fault.
11. **`StartSide::around_part` stays on `StartSide` in K1.** The record
    moves it into the region in K1. The old planners read the field, and
    the old planners must not change on the side branch. The adapter maps
    it into the region now. The field goes with the old planners (record
    step RM).
12. **The tool is a `ToolShape`, not a radius** (record "What phase 1 keeps
    open", point 8). The caller gives the shape: the motion parameters have
    no flute length, so the adapter does not make one up.

**Why**: the record fixes the signature, the invariants and the refusals
of stage 1, but not the reading of the stock edge, the open chain, the
carriers' names or the sliver filter. The choices above keep each value
of the crate valid by construction (#4055 decision 12) and keep the old
planners as they are.

**Dropped**: see each item. Also dropped: a test in `camcraft-app` through
`selection_region` (the app's present path). The motion pins compare the
present centre functions of the same kinds (`non_sliver_center`,
`prepare_band`) and are lower in the crate graph.

**Cost to change later**: low. The region is internal to the kernel and to
the adapter. No project file and no posted program changes. A different
stock-edge reading changes `region_build.rs` and two pins.
