## #600 - Orthographic depth maps to WebGPU's 0..1

**Decided:** the ortho branch of `Camera::mvp` maps view depth
`near..far` linearly to clip z `0..1` (`m[2][2] = 1/(near-far)`,
`m[3][2] = near/(near-far)`), keeping the same `near = 0.1` and
`far = max(40 dist, 1000)` as perspective and the same `Less` test
against a 1.0 clear. The perspective row is left as it is.

**Why:** WebGPU clips `0 <= z <= w`; the GL `-1..1` row put everything
nearer than `(near+far)/2` (20x the orbit distance) at `z < 0`, so the
whole scene was clipped. Keeping the ortho box's near plane at the eye
plane keeps draw and pick in step: `ray_through`'s ortho rays start on
that same plane, so nothing is drawn that a click cannot reach.

**Considered and dropped:** extending the ortho box behind the eye
(`near = -far`) so a deep ortho zoom never slices a large part at the
eye plane. It would also move the ortho pick origin back and re-pin the
#434 ray laws (they cast from the eye plane) - a wider change than the
blank-viewport fix; the slicing matches perspective's own near clip.
Rewriting the perspective row to the 0..1 form: it already survives
(clip.w > 0 keeps z positive beyond ~0.2 mm) and changing it moves every
depth value the snapshot lanes see, for precision only.

**Cost to change later:** one matrix row plus the ortho branch of
`ray_through` and its law tests.
