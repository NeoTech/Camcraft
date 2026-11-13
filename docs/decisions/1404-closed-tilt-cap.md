## #1404 - the span floor's cap is closed at 90 degrees

**Chosen:** `tilt_span::in_cap` admits letters at exactly +-90 degrees
(`|x| <= 90`, was `< 90`). A span whose A and B letters all stand in the
closed [-90, 90] reads no span floor; only letters past 90 do.

**Why:** the release lane's two full-band swarf tests went red at PR
#1306 (bisected: `a0d102834` green, merge `e74bca1f9` red) with 18
`SweepHit`s, every one on a blade-lying link that turns B +90 -> -90. The
sampled pose B 45 read the `-R` floor of its span toward B 90 (outside
the strict cap, so the interval bound gave cos range [0, 0.71] and
`sin_max = 1`). The turn cells pin that floor over B 45's footprint,
where the band stands about 34 mm above the tip. The dip it describes
belongs to the horizontal end, whose own cells already read it in place.
The module's law holds on the closed cap: `cos A, cos B >= 0` along the
lerp and `ln cos A + ln cos B` is concave, so `cos θ` is least at an end,
by continuity also at an end where it is 0. The check was wrong, not the
generator: the cells, which follow the real band positions, find no contact.

**Considered and dropped:**
- A position-aware floor (the span's floor placed at the pose where the
  dip happens). It is more machinery for the same answer here, because
  inside the closed cap the ends already bound the span.
- A tolerance over 90 (e.g. `<= 90 + 1e-9`). Past 90 a lerp can lean
  further than both ends. An exact 90 is what `axis_to_angles` gives a
  horizontal axis, and nothing past it is admitted.
- Touching the fixture, `verifies_clean` or the warning (forbidden: the
  real refusals stay).

**Pinned by:** `verify::sweep::tests::a_swing_through_vertical_reads_no_floor_from_its_horizontal_ends`
(clean 47 mm out at +10, still refused at +40 and under a horizontal
end's band) and the closed-cap brute lerp in
`verify::tilt_span::tests::inside_the_cap_the_samples_bound_the_span_and_past_it_the_floor_does`.
Both fail on the strict cap.
