## #1543 - A whole-model Circular is judged against the finished part

**Decided:** a Circular whose source is a whole model that is the finished
part (`SrcRef::Entity`, `part_solid` true) takes `DepthLaw::Section`, the
same full model law Contour, Face and the picked pocket family answer to.
The law is one function, `authored_law(src)` in
`crates/app/src/operation/generate.rs`, written for the kinds that cut
geometry typed by hand (Circular's centre and radius; the Slot's endpoints
and width, #1537): an `Entity` source is `Section`, a Stock or a pick is
`None` (the pick keeps its kind's own arm). `depth_law_of` routes Circular
through it after its `part_solid` early return, so a Stock source (or an
entity that is the setup's stock) still cuts unjudged, as before.

**Why:** the old contract ("the source is the material to remove, not a
part to keep") holds for a Stock source only. When the source names the
finished part, nothing about a typed circle says it stays inside the part's
own hole, so a circle typed through a wall generated with no warning on the
operation. On main the post was still held in the readings taken, but only
by the program-level stock sim (`post blocked - gouge: program cuts below
the model ...`): the operation itself read clean (status ok, no finding in
MCP `verify`), named no moves, and nothing held it wherever the program sim
does not run. Now the operation carries the blocking `Warning::Gouge` on
the moves that enter the wall.

**Considered and dropped:**
- `DepthLaw::Contain` (the drill/bore solid law with tessellated rims
  excused). It also convicts the wall (mutation reading), but its rim
  excusal is declared per hole (`rims_of`); declaring the typed circle as a
  rim would excuse exactly the radius the user typed, which is the number
  under suspicion. `Section` matches the Slot (#1504, #1537) and the picked
  pockets.
- Judging a picked Circular too. The caller's scope and the issue were the
  entity source; a picked Circular's circle is the picked rim's, but its
  depth is not judged either - filed as #1550 (a ring below a picked floor
  carries no operation warning; the post gate holds it).
- Making generate's MCP reply carry the warnings. Its `status` is
  freshness for every kind ("ok" = current), and the findings ride
  `verify`; changing that is a cross-kind surface change, not this fix.

**Known cost (conservative side):** under `Section` a typed true circle
finishing to size (stock to leave 0) in a coarsely tessellated hole is
convicted by the chord sagitta: a 32-chord radius-10 hole (sagitta 0.048)
refuses it, a 128-chord one (0.003, under the 0.01 gouge tolerance) does
not. STEP imports carry their own excess (#864), which loosens the
allowance; an STL is read as the truth. Leaving 0.05 mm, or a finer mesh,
clears it. Changing it later means a rim-aware variant of the section
law for typed circles.

**Goldens:** none change - no golden or existing test runs a Circular with
an entity source (mill2d_b and units_in_refusals use Stock or picks;
circular_pick uses picks), so every existing Circular path and verdict is
untouched.
