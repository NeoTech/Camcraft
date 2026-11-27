## #2772 - the radial ball drop kernel (third slice)

**Decided:** this PR adds one new motion kernel,
`camcraft_motion::multiaxis::radial_drop::RadialDrop`. It gives the exact
ball-centre radius at an (axial station, angle) about a stated rotation
axis, for any model. It is the 4-axis version of the 3-axis drop cutter.
The ball comes in along the radial line from far outside and stops at its
first contact. The contact is the largest valid root over the face, edge
and vertex patches of each facet grown by the ball radius. No contact is
`None`, never a made-up radius. The angle zero follows the axis.rs basis
law (`rotary_flow_unwrap::basis`), the same frame that Rotary Flow uses.
No operation calls the kernel yet. The commit uses `Refs #2772`.

**Why:** the merged decisions of the first two slices keep the rotary
kinds locked on a part that is not a shape of revolution, because the
current raster plans contact for a turned surface. The next safe step is a
contact law that is exact for any mesh. An exact kernel can be proved
alone: the tests check the face, edge and vertex cases in closed form, and
an independent brute-force march agrees on a helical screw rotor. A
gouge-free test samples 200 centres outward of each contact on 216
stations. Wiring the kernel to Rotary Parallel also changes posting,
heights and verify. That work is the next slice, and it can then rely on a
proved kernel.

**Shank:** a ball nose on the same radial line has its shank inside the
union of the balls outward of the contact. So the shank is clear when the
ball came in clear. A tapered or wider holder is not covered. The wiring
slice must keep the existing holder checks.

**Cull:** each query tests only the facets in its axial bin (bin width
about one ball radius, at most 4096 bins), then a bounding-sphere test
against the ray's line. A pin sweeps stations across a facet's full
reach of +- r, so a cull that is too tight fails a test.

**Dropped:**
- Reuse of `finishing3d::support` (`SupSet`, `tangent_z`). It is built for
  a vertical drop in the world frame. A radial ray per station would need
  one turned copy of the mesh per angle.
- Turning the mesh per angle and calling a vertical drop. It is the same
  law with a copy of the mesh at each angle, so it costs more memory and
  time.
- Wiring the kernel to Rotary Parallel in this PR. That changes posted
  G-code, which is on the safety floor. It needs its own review.

**Cost to change later:** low. The kernel is one file with no callers.
The wiring slice can change its API freely.
