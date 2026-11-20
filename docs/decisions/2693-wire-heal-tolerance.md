## #2693 - Explicit `heal_tolerance` on `WireRuled` bounds 2D swallowtail acceptance

**What was decided.**

- Add `pub heal_tolerance: f64` directly to `WireRuled`
  (`crates/motion/src/wire_edm/ruled.rs`), distinct from twist
  `tolerance`, and validate `is_finite() && >= 0.0` at the
  `wire_contour_ruled` door. Existing callers (`operation/wire_ruled.rs`
  and existing tests) pass `0.0` until #2696 exposes the field in
  `WireFourAxis`, UI, and MCP.
- Add `laws::cover_healed` returning `Option<UncutRest>` alongside
  unchanged `laws::cover` and `laws::cover_run` for 2-axis, open, and
  burnout cuts. It accepts `d <= k + heal_tolerance.max(COVER_TOL)`,
  records the worst `d - k` and `Point3` spot whenever `d - k > COVER_TOL`,
  and keeps the `heal_tolerance == 0.0` refusal byte-for-byte identical.
  When `heal_tolerance > 0.0` is exceeded, the refusal states the spot,
  measured depth `d - k`, and stated `heal_tolerance`.
- In `ruled::emit`, enforce strict `laws::kerf` (`PATH_TOL`) on both the
  lower `Toolpath` moves and the upper ring + leads, and append one
  uncut-rest note to `Toolpath::notes` naming the worst rest across both
  planes, its `X/Y/Z` spot, and the `heal_tolerance`.
- In `ruled_widen`, combine `UncutRest::worse` across the `s = 1.0` cover
  checks and the post-widening `scaled(z, wide)` cover checks so twist
  widening on one flank cannot hide uncut corner rest on an unwidened
  flank, while still catching corners whose widening increases rest.

**Why.** `Region::offset_sharp` already trims local inside-corner
swallowtails and bridges shallow slots in 2D, and `laws::cover`
previously refused every case past `COVER_TOL` (`0.002 mm`). Bounding
that acceptance with an explicit opt-in `heal_tolerance` preserves all
strict refusals at `0.0`, keeps `laws::kerf` (`PATH_TOL`) and ruled laws
intact, and lets the user see the exact worst uncut rest left on the
part.

**Considered and dropped.**

- Reusing `WireRuled::tolerance`: dropped because twist widening moves
  the wire outward past the kerf offset on non-developable flanks,
  whereas `heal_tolerance` bounds uncut material left inside tight
  corners or shallow slots.
- Emitting one note per plane: dropped in favour of one note naming the
  worst rest and its 3D spot (`X/Y/Z`) across both planes.
- Passing `heal_tolerance` into `WireRuled` via a separate helper
  function instead of a struct field: dropped because the struct field is
  the explicit public contract for `wire_contour_ruled` and #2696.

**Cost to change later.** Low: contained within `wire_edm/{laws,ruled,ruled_widen}`
and `WireRuled` construction sites; pinned by `ruled_heal_tests`.
