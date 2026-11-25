# Decision record - issue #2824

## #2824 - Rotary Flow kernel: surface and law choices taken unattended

**Decided:** the kernel's public surface is
`rotary_flow(chains: &[Vec<Point3>], facets: &[Triangle], ax: &Axis,
p: &RotaryFlowParams, c: &AxisControl) -> Result<Toolpath, GenError>`
with `RotaryFlowParams` carrying the `Tool` itself (the drilling
`CycleParams` precedent), so the ball-nose requirement is refused by
name from the tool's own kind. Three laws have a named constant
reading: the clearance and retract read as BALL-CENTER radii about
the stated axis (never world Z - a rotor may stand or lie); the
development's arc-length radius `R_ref` is the band's OUTERMOST chain
radius (a developed stepover then reads widest at the rim, and
ground inside it gets finer, never coarser); and one band per
ADJACENT chain pair is machined, both edges included (a shared
middle chain is walked once per band that uses it - the re-cut is
the same geometry at the same depth, safe and simple).

**Why:** the issue names the axis parameter (the #2772 seam), the
ball requirement and the nearest-whole-turn alignment but leaves
these readings open. The `Tool`-carrying params follow the only
in-crate precedent for a kind-refusing generator (drilling); the
ball-center radius reading is the only one that names a safe place
the same way for a standing and a lying rotor; the outermost-radius
development keeps the stepover promise one-sided (never coarser
than asked); adjacent pairs generalise the two-chain slice without
a second law.

**Considered and dropped:** carrying only `tool_diameter` and
refusing nothing (the issue asks for the non-ball refusal by
name); clearance as a world-Z height (wrong for a lying axis);
developing at the mesh's largest radius as `rotary_offset` does
(its ladder wraps the whole work, while a flow band's own radius
is where the passes run); skipping the shared edge pass between
bands (saved one pass, cost a second alignment law and a special
first pass per band).

**Cost to change:** the cylinder readings and the pair law live in
`rotary_flow.rs` and the emit module's doc header; the app wiring
sub-issue of #2804 will translate its own heights into these radii,
and a later change only touches that seam.

## #2824 - Rotary Flow review round (PR #3630): swept-ball proof and approach refusal

**Decided:** every emitted move is proved clear as a swept ball by the
exact segment-to-triangle distance (a new `rotary_flow_guard` module in
the motion crate). The ball center leaves the straight segment when the
controller blends two poses; the guard adds `r * (1 - cos(theta / 2))`
to the clearance it demands. Flow stations are pushed out to
`r + tol` (the chord tolerance, `min(ARC_FIT_TOL, tolerance)`), and a
chord is accepted only when its sweep keeps `r + tol / 2`; otherwise
the walk bisects it. A chord that no subdivision proves, and a plunge,
retract, link or exit that the guard cannot prove, refuses the whole
program by name at the station's spot. A closed guide keeps its
closing point. The band width is read at the union of both guides'
knots. The push-out march has a fixed step count and refuses a radius
that cannot advance in the float spacing.

**Why:** the review showed that safe stations do not prove a safe cut
(a 0.0065 mm chord dip on the rotor fixture) and that a radial plunge
can cross an overhang. A refusal is the Prime Directive's answer when
no proof exists. The `r + tol` station margin is what lets a chord be
proved without a station count that grows past the chord cap: with no
margin a chord of a convex path always dips under the ball radius.

**Considered and dropped:** searching for a different approach (an
axial or a tilted plunge) when the radial one is blocked - it needs its
own collision law and is a feature, not a review fix; a sampled
Lipschitz proof (`entry_ramp::segment_clear`) - it needs a 0.025 r
extra margin and many samples per chord; reading the sim's
`dist_seg_tri` - the motion crate does not depend on the sim, and a new
edge between our crates needs its own decision.

**Cost to change:** the margin is one argument of `ball_center` and one
factor in `Walk::flatten`; an alternative approach law would replace the
refusal in `prove_path` for the link moves only.
