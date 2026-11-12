## #950 - doosan-dnm.cps never assumes the table stands at A0

**Decided:** the post's modal angle memory (`lastA`/`lastB`/`lastC`) starts
as "unknown" (`null`) and is reset to unknown, not to 0, at every tool
change and section end. The first tilted block of a program and of each
section therefore writes its A word (with the M11/M10 brake pair around
it), A0 included.

**Why:** the post never commands A0 before the first section and a
section end moves nothing it can see, so an assumed A0 was a guess; when
the previous tool left the table at A30, an A0 operation posted no A word
and cut at A30 (P0). Writing one extra A word per section where the angle
already matches costs a block; omitting a needed one gouges.

**Considered and dropped:**
- Keep the memory across sections and tool changes (write A only when it
  really changes). Fewer blocks, but it trusts that nothing between
  sections (an operator stop, an M6 macro, a home) turned the table; the
  issue asks for the first A after anything that may have moved it.
- Write A0 in the program preamble. Adds a table move the stream never
  asked for, and still leaves the per-section question open.

**Side effect, kept:** with TCP (G43.4) the arm block already wrote all
three angles, but the brake was judged against the assumed A0, so a turn
back to A0 after A30 ran without M11/M10. The same reset now unclamps for
that turn too (pinned in `doosan_dnm_a_state`).

**Cost to change later:** one line per reset site in the post; the pinning
tests in `crates/app/tests/doosan_dnm_a_state.rs` say what must still hold.
