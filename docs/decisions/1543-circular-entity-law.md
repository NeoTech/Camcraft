## #1543 / #1550 - A Circular is judged against the finished part, whole model or pick

**Decided:** a Circular whose source is the finished part (`part_solid`
true) takes `DepthLaw::Section`, the same full model law Contour, Face,
the picked pocket family and the Slot answer to - whether the source is
the whole model (`SrcRef::Entity`, #1543) or a pick
(`SrcRef::Selection`, #1550). The law is one function,
`authored_law(src)` in `crates/app/src/operation/generate.rs`, for the
kinds that cut geometry typed by hand: Circular's centre and radius and
the Slot's endpoints and width. `depth_law_of` routes both the Circular
and the Slot arm through it (`Slot | Circular => authored_law(src)`), so
the sharing is real code, not a comment: an `Entity` or `Selection`
source is `Section`, a Stock is `None`. Both arms sit after
`depth_law_of`'s `part_solid` early return, so a Stock source, or a
source naming the setup's stock (a pick off a from-model stock), still
cuts unjudged.

The Slot's behaviour is unchanged by the routing: on main since #1504 /
#1537 it was `Section` for every part-solid source, and `part_solid` is
never true for `SrcRef::Stock` (`part_solid_of`), so `authored_law`'s
`Stock -> None` arm is unreachable from `depth_law_of` and kept only so
the function read alone stays truthful.

**Why:** the old contract ("the source is the material to remove, not a
part to keep") holds for a Stock source only. When the source names the
finished part, nothing about a typed circle says it stays inside the
part's own hole, or above the floor it was picked off. On main:
- a whole-model circle typed through a wall generated with no warning on
  the operation; the post was held only by the program-level stock sim.
- a picked Circular cut 3 mm below its picked floor read clean on the
  operation, and with a rotated stock the program sim refuses to judge
  ("the program sim models an axis-aligned stock"), `uncovered()`
  (`posting_sim.rs`) accepts the operation's own empty verdict, and only
  an unrelated `RapidInStock` stopped the post in the review's repro - a
  gouge that could post (#1550, raised to P1). The same holds wherever
  the sim refuses (posed or tilted moves, rotary, lathe frames).
Now the operation itself carries the blocking `Warning::Gouge` on the
moves that enter the part, and the post is refused by name on those
moves ("C1: move N cuts past the finish surface"), sim or no sim.

**Considered and dropped:**
- Re-scoping the comments to Circular only and leaving picks at `None`
  (the review's first option). That leaves #1550's gouge able to post
  and makes `authored_law` disagree with the Slot on picks. Judging picks
  is the law the Slot's picks already take (#1504).
- `DepthLaw::Contain` (the drill/bore solid law with tessellated rims
  excused). For a Circular it convicts exactly what `Section` does:
  `Section`'s solid law (b2) runs the same `SolidLaw` with the same
  allowance (`tightened` is `excess_floor(..).unwrap_or(tol)`, as
  Contain's), and `rims_of` declares no rim for a Circular, so
  `hole_walls` excuses nothing. Readings under the mutation
  `Entity -> Contain`: the wall case, a whole-model ring 3 mm below a
  blind floor (3079 gouges under both), a ring at the floor (clean under
  both), and the review's chord-sag table (identical) - no gouge verdict
  differs, so the choice is pinned by the law table
  (`authored_law_tests`), not by an integration test. What `Section`
  adds is the plane law (b) and the model-fed reach laws (hidden stock,
  the shank's part label); declaring the typed circle as a rim would
  excuse exactly the radius the user typed, the number under suspicion.
  `Section` matches the Slot and the picked pockets.
- Making generate's MCP reply carry the warnings. Its `status` is
  freshness for every kind ("ok" = current), and the findings ride
  `verify`; changing that is a cross-kind surface change, not this fix.

**Known cost (conservative side), measured:** under `Section` a typed
true circle finished to size in a coarsely tessellated hole is convicted
by the chord sagitta - refused exactly when the sagitta exceeds the
stock to leave plus about 0.01 mm (`GOUGE_TOL`); the mesh says the
material is there. On a radius-10 hole with a 6 mm end mill (gouge
counts from the review, pullrequestreview-5898906681):

| chords (sagitta, mm) | leave 0 | leave 0.01 | leave 0.02 | leave 0.05 |
|---|---|---|---|---|
| 32 (0.048) | refused (1026) | refused (1024) | refused (768) | clean |
| 48 (0.021) | refused (960) | refused (384) | clean | clean |
| 64 (0.012) | refused (512) | clean | clean | clean |
| 96 (0.005) | clean | clean | clean | clean |
| 128 (0.003) | clean | clean | clean | clean |

So a 32-chord hole is refused up to leave 0.02, a 48-chord one up to
leave 0.01, a 64-chord one at leave 0 (it does NOT pass at leave 0), and
96 chords or more never. STEP imports carry their own excess (#864),
which loosens the allowance; an STL is read as the truth. Leaving
0.05 mm, or a finer mesh, clears it. A pick hands the fitted circle, so
the same reading applies to a picked Circular finished to size.
Changing it later means a rim-aware variant of the section law for
typed circles.

**Goldens:** none change - no golden runs a Circular on a part-solid
source with an overcut (mill2d_b and units_in_refusals use Stock or
legal picks; circular_pick's picks cut to or above their floors), and
the Slot's law is unchanged.
