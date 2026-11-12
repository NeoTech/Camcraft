## #1303 - a tilted band's column dips by the rim's exact R·sin θ

**Decided.** The column laws that read a leaning tool stack enclose each
band by its exact rim dip, not by its whole radius. One helper,
`verify::tilt_band::lean` (with `rim_dip` / `rim_sine`), builds the
column; the four copies of the formula are gone. Its consumers:

- law (c), reach against rest material (`reach.rs`);
- law (c3), its mid-turn sweep (`sweep.rs`);
- law (c2), the fixture band column (`stack_span::band_struck`);
- law (d2), the tilted rapid's flute column (`laws.rs`).

The upright branch of every law is untouched, bit for bit. Only the two
z planes of the tilted column moved; its centre and radius are the sums
they always were.

**The derivation.** A band is the cylinder of radius R about the unit
axis `a` (tilted θ off +Z) between the axial stations `above` and
`above + len` over the tip. Its points are `tip + s·a + ρ·w` with `s` in
the span, `0 <= ρ <= R`, and `w` any unit vector square to `a`.

- *In z.* Write +Z as `cos θ·a + sin θ·h`, `h` a unit vector square to
  `a` (the tilt direction, lifted). Then `w·Z = sin θ·(w·h)`, and
  `|w·h| <= 1`, so the disc part `ρ·w` changes z by at most `R·sin θ`,
  with equality at the rim point `w = ±h`. The axial part changes z by
  `s·az`, linear in `s`, so its extremes are at the two ends. The
  band's lowest point is therefore exactly
  `tip.z + min(above·az, (above + len)·az) - R·sin θ`
  and its highest `tip.z + max(..) + R·sin θ`. Nothing of the band's own
  height is left out: the two end stations are both in the `min`/`max`.
- *Sideways.* `s·a` stands within `len/2 · sin θ` of the mid-station's
  foot `mid·(ax, ay)` and `ρ·w` adds R or less, so the band lies in the
  vertical column of radius `R + len/2 · sin θ` about that foot. This
  part is unchanged.

So the column `[centre mid·(ax, ay), radius R + len/2·sin θ, z from
lo - R·sin θ to hi + R·sin θ]` contains the whole tilted band, and its
bottom and top planes are touched by it: the dip is exact, not slack.
The old column used `R` where this one uses `R·sin θ <= R`, so it was
the same enclosure with up to R of extra depth.

**Why it never under-reads.**
- `rim_sine` returns `min(1, max(h, h/|a|))` with `h = hypot(ax, ay)`.
  For a unit axis `h` is the sine itself. The laws place the band with
  the raw components, so for an axis off unit length the larger of the
  raw and the normalised reading is taken: never less than the sine of
  the true tilt, never more than 1.
- A zero or non-finite axis gets 1: the whole radius, the reading
  before this change. What the enclosure cannot read fails toward the
  deeper dip.
- A tool pointing down (θ > 90 degrees) is covered by the same algebra:
  `sin θ` is still the rim's reach and the `min` picks the far end.
- Float rounding of `R·hypot(ax, ay)` is one unit in the last place
  (about 1e-15 mm on a 12 mm disc), nine orders under the laws' kerf
  margins; no pad is added.
- A property test places sampled points of the tilted band (4000 random
  cases: tilt 0 to 180 degrees, any azimuth, R, foot and length; nine
  stations, three radii, 48 rim angles each) and requires every one
  inside the column, and the column's bottom and top to be reached by
  the samples. Two mutations were run by hand against the pins: the
  whole radius (the old reading) fails the continuity pins, half the dip
  (an under-read) fails the conviction pins and the property test.

**Why this is the fix.** With the whole radius, a band a hair off the
column read R lower than upright: an r 12 holder 5 mm clear of a part
was clean at 1e-9 rad (upright by #938) and `ShankHit { part: true }` at
2e-9 rad. Any commanded tilt paid it. With the exact dip the tilted
reading tends to the upright one as θ goes to 0, so the verdict turns
where the rim reaches the material and the #938 tolerance is no longer a
cliff.

**Verdicts that moved.** Only tilted moves, only toward clean, and only
where the old column's extra depth (between `R·sin θ` and `R` below the
band, or above it for the fixture laws) was what met the material.
- `crates/sim/tests/tilt_tolerance.rs` (#938's pins): the r 15 holder
  5 mm over the stock no longer convicts at 1.01e-9 rad or at 0.001
  degrees; that test now pins "reads like upright there, convicts at 20
  degrees where the rim is 1.9 mm into the stock". The one-report-per-
  move pin moved its tilt to 20 degrees so the holder group still
  convicts beside the shank group.
- No golden, no posted output and no other existing test changed.

**Considered and dropped.**
- A per-column dip (the band's true underside over each sample column,
  an ellipse-shaped floor instead of one plane): tighter away from the
  low rim, but a new intersection law and a larger change than the
  issue asks. The single plane at the true lowest point is exact where
  it matters (it is reached) and stays one comparison.
- Keeping the whole radius in the fixture laws (c2)/(d2) and changing
  only (c)/(c3): two copies of one formula with different depths, and
  the same cliff left in the column half of the fixture law.
- A safety pad on the dip (say 1e-9 mm): the laws already carry their
  own margins (`r_band + 1e-9`, the fixture hit tolerance); a second
  pad would be a second unexplained number.
- Raising the dip to the largest tilt of the neighbouring pose samples
  in (c3): see below, the sampled bottoms already bound the in-between
  ones.

**Left as is, and known.**
- Law (c3) still samples the turn (4 stations x 5 pose fractions). The
  old whole radius happened to cover any in-between tilt in z. It is
  not needed for that: the letters lerp linearly, `cos θ = cos A·cos B`
  makes `-ln cos θ` convex in (A, B) inside the upper hemisphere, so an
  in-between pose is never more tilted than the more tilted of its two
  neighbouring samples, and the band's bottom
  (`above·cos θ - R·sin θ`) falls with θ up to 90 degrees. The sideways
  sampling limit is the one the law always stated.
- The fixture law's public verdict on a posed move is still the OR of
  this column and the posed capsule sweep (`posed_steel.rs`), whose
  capsules reach one whole radius past each band end. That sweep still
  convicts a clamp just under a holder on any tilt above the tolerance
  (`the_fixture_band_law_reads_noise_as_upright_and_a_real_tilt_as_tilted`
  is unchanged). It is a different enclosure (a sphere-swept segment,
  not this column) and needs a cylinder-box distance to tighten; filed
  as #1305.
- Law (c)'s column has no top and convicts anything above its bottom
  inside its radius; that is the reach law's design, not the dip's.

**Cost to change later.** One function, `tilt_band::lean`, and its
tests. The pins are `crates/app/tests/tilt_band_dip.rs` (both ways,
continuity, the mid-turn sweep), the unit tests in
`crates/sim/src/verify/tilt_band.rs` (property, limits, the (c2)
column) and `crates/sim/tests/tilt_tolerance.rs`.
