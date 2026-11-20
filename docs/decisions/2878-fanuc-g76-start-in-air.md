## #2878 - Fanuc lathe G76 free-air start law (`start_in_air` on `ThreadCycle` and `AIR 1` in `THREAD`)

- **What was decided**:
  1. Carry `start_in_air: bool` on `cl::ThreadCycle`, serialize it as optional `AIR 1` on the intermediate `THREAD` record (absent = `false`) and `#[serde(default, skip_serializing_if = "is_false")]` in `ThreadCycleDto`, and expose it to post scripts as `properties.StartInAir`.
  2. Refuse G76 by name in `assets/posts/fanuc-lathe.cps` (`g76Why`) whenever `p.StartInAir !== true`, so a start point above material (or a cached pre-#2878 record or legacy stream without `AIR 1`) never writes a G76 cycle.
  3. Compute `start_in_air` from the revolved meridian across the full rapid-infeed axial band `[min(start_s, s_deep) - insert_width/2, max(start_s, s_deep) + insert_width/2]` (where `s_deep = start_s + dir * height * tan(infeed_angle/2)`): true only when the part silhouette stays at or below `root` across the band (in front of the end face or in a relief groove wider than the insert) AND no remaining stock stands above `root` there after prior Facing, Grooving, Turning or Drilling operations (`operation::g76_start`).
  4. Allow `turning::thread` to extend `k_hi` up to `top_z` past the bar's free end face and fix `revolve::lattice` not to re-push out-of-domain window endpoints after `lat.retain`, so a G76 thread can start in free air in front of the end face.
  5. Update the three Fanuc G76 golden fixtures (`thread_g76.nc`, `thread_g76_area.nc`, `inch/fanuc-lathe/thread_g76.nc`) to use a free-end bar starting at `Z16.` (`Z16.625` flank, `Z0.6299` inch) in free air instead of `Z14.` inside a solid Z=40 bar.

- **Why**:
  - A Fanuc-type G76 moves the insert from the start point to each pass's depth at rapid traverse. Checking `StockZTop` alone in `.cps` cannot distinguish a faced end or a machined relief groove from uncut stock, nor does the post script know the insert width or part silhouette. Computing `start_in_air` from the revolved part, stock and prior cuts in the generator/verifier and requiring `StartInAir === true` in `fanuc-lathe.cps` is fail-safe by default: older caches and hand-written streams without `AIR 1` refuse G76 rather than silently rapiding into steel, while G32 passes (which plunge with `G01`) stay unchanged.
  - Putting the `StartInAir` check at the end of `g76Why` preserves more specific plan/range/hand refusals (such as chip-break dips or alternating flank infeed).

- **What was considered and dropped**:
  - **Checking only `StockZTop` inside `fanuc-lathe.cps`**: dropped because it rejects valid relief grooves and faced bars (where `start.z < StockZTop`) and misses narrow end-face overlaps where `start.z > StockZTop` but `start.z - insert_width/2 < StockZTop`, as well as flank-infeed shifts.
  - **Auto-shifting the G76 start point inside `fanuc-lathe.cps`**: dropped because the post never shifts verified points (`fanuc-lathe.cps` law) and shifting past a solid bar would cut unmodeled length.
  - **Changing `tormach-lathe.cps` in the same PR**: dropped to stay scoped to #2878 (Fanuc lathe G76); LinuxCNC G76 can adopt the same `StartInAir` property in a follow-up without changing the stream format.

- **Cost to change later**:
  - Low: `AIR 1` is an additive optional flag on `THREAD` and `ThreadCycleDto`, and `tormach-lathe.cps` can read the exact same `properties.StartInAir` field whenever its G76 fixtures are updated.

### Review fix: prove continuous stock clearance

- **What was decided**: Bound each interval between stock profile breakpoints by a solid rectangle above the thread root. Prove that each rectangle is inside prior cut regions. Subdivide a rectangle when several cuts provide the coverage. Refuse G76 if the proof exceeds its depth or cell budget. Use the actual swept segment for blade cuts. Reduce blade and nose credit by the clearance margin.
- **Why**: Isolated axial and radial samples can miss a narrow stock rib or a gap between prior cuts. A rectangle proof covers the material between the samples. Keeping each capsule proof on one signed side also avoids credit across the spindle axis.
- **What was considered and dropped**: More samples cannot prove clearance between samples. Stock hole credit and tolerance-based gap closure were also dropped because they could credit material that the prior cuts did not remove.
- **Cost to change later**: The conservative bounds can refuse hollow stock or a cut union whose boundary cannot be proved within the budget. A later exact coverage method can reduce these refusals without changing the cycle record or post contract.
