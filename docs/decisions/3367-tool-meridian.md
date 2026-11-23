## #3367 - The tool meridian: one line-and-arc outline for each mill kind, split into convex atoms

**What was decided.**

- A new module `camcraft_tools::meridian` holds `Meridian` (the points of
  `SpunPoint`, plus one cutting flag for each segment), `Tool::meridian()`
  and `Meridian::atoms(bound, sag)`. No consumer reads it yet.
- The meridian covers the cutting band only: the tip face, the tip shape
  and the flute side to `flute_len`. The shank and the holder stay with the
  stack laws. The lollipop is the exception: its meridian is the whole ball
  to its pole at `2r`, because `Cutter::atoms` keeps the whole ball.
- A convex arc gives one torus zone atom (exact). A concave arc gives chord
  frusta (`Bound::Outer`) or tangent frusta (`Bound::Inner`). The split
  uses the smallest part count whose sag is under the limit, at most
  `MAX_SPLIT` (4096). A limit that needs more parts is refused
  (`SagTooFine`).
- A flat (zero-height) segment is a face, not an atom.
- An arc must be a function of z: a convex arc lies on the far half of its
  circle, a concave arc on the near half. Otherwise `ArcFolds`.
- When an arc end sits at the radius of the circle centre, the centre is
  snapped to that end (`zc = z +/- R`). Without it, float dust of 1e-16 in
  the centre gives 3e-8 mm of error in rho at that end, over the 1e-9 pin.
- A bull nose whose corner radius is over half the diameter is refused by
  name. `Cutter::of` clamps it; the meridian does not guess.
- An undercut cutter that `undercut::refusal` names is refused with that
  text.
- Kinds that are not mills (tap, bore, probe, lathe and EDM kinds) return
  `NotAMill`.

**Why.** The issue asks for one shape law that the gouge checks will read
later. A refusal is safer than a clamp for that law. The atom test lives in
`camcraft-sim` (a unit test next to `Cutter::atoms`, which is `pub(super)`),
so no crate edge and no visibility change is needed.

**Dropped.**

- Shank and holder bands in the meridian: they are not the cutting outline,
  and the stack laws already read them.
- Changing the visibility of `Cutter::atoms` for a test in `camcraft-tools`:
  the tools crate cannot import sim.
- A cutting flag inside `SpunPoint`: the drawn holder and cutter profiles
  share that type; a separate flag list keeps it unchanged.

**Cost to change later.** Low. No consumer reads the meridian yet.
