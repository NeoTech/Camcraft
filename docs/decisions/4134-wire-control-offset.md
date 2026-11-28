## #4134 - Wire Contour: control-side wire offset mode (COMP records for the LN2W post)

**Decided.** A Wire Contour states a `compensation` mode: `computer` (the
default, as before), `control` or `wear`. The words are those of the mill
contour's mode (`links::Compensation`), so one word has one meaning.

- **The moves stay the wire-centre path.** The generator makes and judges
  the same wire-centre path as before. With `control` or `wear`, each pass
  then gets one `CompSpan` (`wire_edm::wire_comp_spans`, the motion crate).
  The post dump already writes a span's programmed geometry between `COMP`
  records (M13.11). So the LN2W post of #4139 gets the `COMP` records with
  no change to the post or the engine.
- **Item 3 of the issue: how the wire laws judge the path.** The kerf,
  threading and slug laws judge the wire-centre moves, as before. These
  moves are what the control cuts when each register holds the radius that
  its span states: the pass's kerf offset (wire radius plus the spark gap of
  the pass's condition). The post writes that radius in the comment before
  each `G41` / `G42` line. A register that holds a different value cuts a
  different path, and no law can judge a value that is set at the control.
  The program says so.
- **The programmed contour.** Each ring move of the pass is moved back by the
  offset along its normal: a line by translation, an arc by its radius about
  the same centre. At a sharp (mitred) joint the two lines meet at the corner
  that the wire path was offset from. A corner round that the offset put in
  (an arc of radius equal to the offset) collapses to its corner. The square
  test part programs `X5. Y5.` (the part), not `X4.835` (the wire centre);
  the 2-axis block programs its holes at the part radii 2.5 mm and 3 mm.
- **One register for each pass, from the rough pass.** The spans are in cut
  order: the rough pass first, then each skim with its own condition's
  offset. The post numbers them `H001`, `H002`, ... and writes `C001` with
  `H001` (the owner's rules on #4076).
- **Lead-in and lead-out.** They are the straight leads of the wire path,
  from the start hole to the part contour and back. Each must be at least
  the pass's offset long; a shorter one is refused by name (in the span
  builder, and again in the post's shared helper).
- **Wear.** The programmed moves are the wire-centre moves. The span states
  the pass's kerf offset as its radius; the post says that the register
  holds only the wear delta.
- **Refused at the command door** (and again at generation) with control or
  wear: an open run, a 4-axis cut, a taper, a burnout, tabs, the arc lead
  and an exit pick. Each has no straight lead-in from the start hole and
  back, or the LN2W offset with it is not confirmed.
- **MCP.** The parameter rides the operation DTO (`compensation`:
  `"computer"` / `"control"` / `"wear"`), so `add_operation` and
  `edit_operation` take it. `list_operation_types` lists it, and the
  `camcraft-mcp` skill row names it (pinned by `mcp_wire_contour`).
- **No file or stamp moves.** The DTO writes the key only when it is not
  `computer`, and the stamp digests it only then. Every existing golden and
  project file stays byte-identical.

**Why.** The span mechanism keeps one truth (the wire-centre moves) for the
laws, the simulation and the goldens, and it is the one that the mill
contour's control compensation uses. A second path for the post only would
need its own laws.

**Considered and dropped.**

- Caching the part contour as the path and judging a regenerated
  wire-centre twin at verify time. Dropped: two paths for one cut, and the
  verify would need to run the generator.
- Computing the programmed contour from the picked profile (the mill's
  method). Dropped: the wire's profile comes from several generators
  (facets, model arcs, drawn arcs); the offset-back of the wire path works
  the same for each of them.
- Refusing a seam at a corner of the wire path (the mill contour refuses
  it). Not needed here: the engage and cancel blocks end exactly at the
  wire path's seam point, because each programmed end is the wire end moved
  back along that move's own normal.
- Stacking on PR #3617 (a contributor PR with `needs-rework`). That PR adds
  the `overlap` field to the same structs and changes `announce_skims`. This
  change does not touch `announce_skims`, and it adds its field lines after
  `lead_arc_radius`, away from the lines of #3617. Stacking would hold this
  feature on another author's open rework. Whichever PR merges second
  refreshes; the conflicts are "both sides added lines".
- A UI control on the wire page. Not in this change: two open PRs change
  `wire_page.rs`. A follow-up issue owns it.

**Cost to change later.** Low. The mode is one field and one DTO key; the
spans are one function in `motion::wire_edm::comp`.
