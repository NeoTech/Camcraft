## #2158 - Deterministic true-shape nesting in camcraft-geometry

**Decided.** Integrate the held CamCraft nesting kernel from
`origin/issue/2158-nesting-kernel` into the current geometry crate. Keep the
existing `nesting.rs` entry point and its public raster module. Add no ECS,
world access, or crate dependencies. The algorithm follows Bennell and
Oliveira (EJOR 2008) and Burke, Hellier, Kendall, and Whitwell (OR 2006,
EJOR 2007); no third-party nesting implementation supplied code.

- Compute NFPs from convex decomposition and Minkowski sums on the existing
  `geo2d::SCALE` integer grid. Preserve zero-width seams swallowed by polygon
  unions so exact-fit cavities remain candidates. Compute rectangular IFPs
  from bounds and closed-hole IFPs from the container complement's NFP.
  Keep triangulation and its slab fallback local to nesting.
- Place parts with deterministic bottom-left-fill. Sort by decreasing outer
  area, then stable part id, then instance index. Try permitted rotations
  and mirrors, deduplicate equivalent footprints, and cache local NFPs.
  Outer-area ordering places a hollow frame before parts that can use its
  holes even when the frame's net material area is smaller.
- Apply separate-cut inflation by cut side: `cut_width + part_gap / 2`
  outside, `cut_width / 2 + part_gap / 2` on line, and `part_gap / 2`
  inside. Buffer open chains with `Region::sweep`. Offer an explicit router
  single-slot mode and refuse it when `part_gap < cut_width`.
- Reduce the sheet by `edge_margin` with `Region::offset_sharp`, then keep
  each cut footprint inside that usable sheet. The footprint already
  includes outer cutter clearance, avoiding double-counting at the edge.
- Search usable holes before allocating a new sheet. Treat placed hole
  occupants as filled outer obstacles within their parent's hole. Expose
  their own holes separately and report the immediate containing part.
  Require empty intersection and outside-difference regions during exact
  verification; a small positive overlap is not accepted as a tolerance.
- Check requested-quantity addition and refuse totals above 10,000
  instances by name. This bounds placement work and report allocation.
  Report counts, net-area utilization, used bounds, waste regions, and
  named unplaced quantities without producing toolpaths or G-code.

**Why.** Reusing the held clean-room work preserves its existing geometry
coverage while adapting only the entry point and refusal checks to current
main. Convex decomposition makes concavities and swept-chain polygons use
the same geometry operations. Explicit seams retain exact fits, while the
final region checks enforce clearance independently of candidate generation.

**Dropped.** Orbital NFP sliding needs more state around collinear edges and
pinched cavities. Net-area sorting can place a small solid before its
larger hollow host. Raster-only placement cannot establish exact vector
clearance. Changing `geo2d::Region::triangles()` would broaden this issue;
the fallback remains local. Unbounded quantities risk impractical loops or
allocation, and positive-area overlap tolerances weaken the zero-overlap
contract.

**Validation.** The unchanged upstream API failed the pinning build. The
restored kernel passed 66 focused tests. Nine runtime mutations each failed
their pin and passed after restoration: hole containment, tiny overlap,
router gap, negative input, quantity overflow, quantity cap, kerf inflation,
NFP seams, and filled-hole obstacles. Restoring upstream `nesting.rs` also
failed the API pins and passed after restoration. The quick gate passed all
936 steps, and geometry all-targets clippy passed with warnings denied.
No G-code or CL goldens changed. App and motion heavy lanes were planned
but not run; the PR remains a draft.

**Cost to change.** NFP, IFP, and placement search stay behind internal
functions. Another search heuristic or a configurable quantity budget can
replace those policies without changing the geometry primitives. Raising
the quantity cap requires performance and memory measurements. Changing the
public input or report structs requires updating downstream nesting users.
