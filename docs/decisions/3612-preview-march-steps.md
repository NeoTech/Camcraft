## #3612 - Stock preview march: 256 steps and an exact verdict at exhaustion

**Decided.** The stock preview march (the CPU twin
`crates/sim/src/stock_sim/preview/march.rs` and the GPU kernel
`crates/ui/src/trace.wgsl`) has a step cap of 256 (it was 64). Both twins
have the same constant. When a ray uses all its steps, it gets a hit only
where material is proved on the ray: the last sample is inside, or the
sample one cell further along the ray is inside (the probe of the existing
hit law). Then 6 bisections of that one-cell bracket put the hit within
1/64 of a cell of the surface, on the material side. Each other exhausted
ray keeps status 2 (no verdict). The hit law and the miss law do not
change. `composite.wgsl` does not change.

**Why.** Measured on the CPU twin. The fixture is a 10 x 10 x 5 box on
1 mm cells, faced to z 4.5 with a flat r 2 tool. The frame is 64 x 64,
and the eye is 30 mm from the floor centre. The table gives the rays
with no verdict for each cap:

| Angle above the floor | 64 | 96 | 128 | 192 | 256 | 512 |
|---|---|---|---|---|---|---|
| 60 deg | 0 | 0 | 0 | 0 | 0 | 0 |
| 30 deg | 0 | 0 | 0 | 0 | 0 | 0 |
| 10 deg | 2 | 0 | 0 | 0 | 0 | 0 |
| 5 deg | 70 | 0 | 0 | 0 | 0 | 0 |
| 3 deg | 152 | 42 | 6 | 0 | 0 | 0 |
| 2 deg | 166 | 62 | 6 | 0 | 0 | 0 |
| 1 deg | 112 | 70 | 64 | 24 | 0 | 0 |

256 is the smallest measured cap that gives 0 down to 1 degree. At 64,
rays that surely meet the box read as a miss (5 deg: 8, 3 deg: 68,
2 deg: 30). At 256 there are none. No ray that surely misses the box is
painted at any cap, with or without the fallback.

The fallback alone at cap 64 changes 5 deg from 70 to 32 and 3 deg from
152 to 56, and removes the sure hits that read as a miss at 5 deg. It
cannot help a ray that passes just above the floor and misses: that ray
has no material in its bracket, so it keeps status 2, which is correct.
At 256 with the fallback, 0.5 deg leaves 16 rays with no verdict, and
none of them is a sure hit.

Cost: the CPU preview of the fixture (debug, the whole frame with the
column cache) took 577 ms at 64 and 643 to 661 ms at 256 for the 1 degree
view (about +11 %). The other views are within the noise. The fixed frame
cost is most of the time, not the march. On the GPU only the pixels that
used to run out pay for the extra steps.

**Dropped.**
- A hit for each exhausted ray within one cell of the surface (option 1 of
  the issue). It paints a ray that misses by less than a cell, so the
  see-through law is not exact.
- A larger `STEP_EPS`. It moves the depth by up to the new epsilon and
  breaks the depth tolerances of `trace_parity`.
- A cap of 512. It gives the same result as 256 on the fixture and doubles
  the worst-case cost.

**Cost to change later.** Two constants (`MAX_STEPS`, `BISECT`) in each
twin. `trace_parity` asserts that the two twins agree on unresolved
pixels, so a change of one twin alone fails that test.
