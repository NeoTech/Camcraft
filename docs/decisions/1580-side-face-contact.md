## #1580 - Resolve carried side-face and step contact below one grid pitch

**What was decided.** Keep the dexel column checks (`on_columns` and
`between_columns` in `crates/app/src/fixture_carried.rs`) as the
coarse pass, and add an adaptive sub-pitch boundary pass
(`boundary_box_penetration` and `boundary_mesh_penetration` in
`crates/sim/src/stock_sim/carried_boundary_scan.rs`) over continuous
carried-stock history (`History`):

- **Retain continuous provenance.** Keep exact box bounds `x1, y1` in
  `Dexel::block`, preserve `cylinder` bounds across `Dexel::air_outside_circle`
  and `Dexel::translate_origin`, and retain closed `FixtureMesh`
  provenance in `Body::on_mesh`.
- **Route continuous column reads.** In `fixture_carried::material`, use
  continuous exact rays (`Body::needs_continuous_column`) only where a
  column can differ from the sampled grid (mesh-backed, turned/active,
  cylindrical, or an outer padded column), and skip the `!any_inside`
  column-centre fallback when continuous history is present.
- **Classify and subdivide boundary regions.** `Body::classify_region`
  (`carry_boundary.rs` and `carry_boundary_prune.rs`) classifies an XY
  box as empty, full over an interval, or mixed, pruning via interval
  bounds, signed-permutation frame pullback, closed `FixtureMesh`
  queries, uniform flat straight cuts, XY circular arc cuts,
  horizontal `SideTurn` cuts, and retained `Clip` grids.
- **Pair with guided mesh wall checks.** `FixtureMesh::steel_in_axis_box_uv`
  and `steel_on_axis_line` extend prism and line queries to all signed
  coordinate axes and report the `(u, v)` span of boundary facets. When
  box or mesh subdivision sees mixed regions near `CARRIED_CONTACT_TOL`
  (0.01 mm) or `FIXTURE_HIT_TOL` (1e-9 mm), k-d bisection down to
  1e-3 mm / 2e-3 mm leaves (plus guided boundary wall splits and
  eroded side-tolerance sampling `Body::eroded_spans_at`) catches
  sub-pitch penetration while clearing legal tangent contact and
  recessed overhangs. Regions already bounded by `<= worst` are pruned.

**Why.** On `upstream/main`, `Dexel::block` rounded history bounds to
multiples of `cell`, cylinders and model stocks discarded continuous
side-face geometry (`Dexel::air_outside_circle` cleared `history` and
`Body::on_mesh` dropped the source mesh), and `between_columns` only
checked what four corners of a grid cell agreed on. A vise jaw or
step up to one grid pitch into a block, cylinder, closed model, or
machined wall could sit outside the sampled column centres and pass as
clear. Resolving mixed regions against continuous geometry closes the
gap for sub-pitch overlaps without lowering grid resolution.

**Considered and dropped.**
- Scanning an upfront grid of every face break across the fixture box:
  exponentially slower on large fixtures and still blind to tilted or
  curved walls; replaced with adaptive bisection with `d <= worst`
  pruning and guided `steel_in_axis_box_uv` splits.
- Refining the dexel grid pitch everywhere: increases memory and sim
  runtime quadratically while still shifting true face positions to cell
  edges by up to half a pitch.
- Placing the boundary subdivision solver in `crates/app`: keeps sim
  geometry queries inside `camcraft-sim` and preserves `fixture_carried.rs`
  under 400 lines.

**Cost to change later.** Carried-stock operations that discard
continuous history (such as raw unbacked grids or open/non-watertight
model meshes where `FixtureMesh::build` fails) continue to use the
column grid. Any new continuous cut primitive added to `CarryHistory`
should add its conservative region classifier in `carry_boundary_prune.rs`
so mixed tiles do not fall back solely to corner sampling.
