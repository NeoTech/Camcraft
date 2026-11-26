## #3328 - Start and exit points: the hole order of Drill, Bore and Thread

Part of #1448. It builds on the shared mechanism of #3324 and on the
operation-level `picks` of #3339.

**What was decided.**

1. **The capability.** Drill, Bore and Thread (mill) answer
   `PickCapability::HoleOrder`. Each takes a start pick and an exit pick.
   Probe stays `NotTaken` ("a probe touches the part and cuts no path").
2. **Which hole a pick names.** A pick is on a hole when it is inside the
   circle of the hole (the distance to the centre is not more than the
   radius, with 1e-6 mm for a typed coordinate). When two circles hold the
   pick, the nearer centre wins. At the same distance, the earlier hole of
   the list wins.
3. **The order rule.** The list order, rotated so that the start hole is
   first. Then the exit hole moves to the end. With an exit pick only, the
   list keeps its first hole and the exit hole moves to the end. No pick is
   the list order: every earlier program is byte-identical.
   Example, list A B C D: start C gives C D A B; exit B gives A C D B;
   start C with exit D gives C A B D.
4. **Refusals, by name. A pick is never moved.**
   - A pick that is on no hole: "the start point (x, y) is not on a hole of
     this operation".
   - A start pick and an exit pick on the same hole, when the list has more
     than one hole. With one hole, the first hole is also the last hole.
   - A pick on a Drill with cross holes: the turret plunges them along X or
     Y, so an XY point does not name one.
   - A pick on a lathe's axial Drill: all its holes are on the spindle
     axis, so an XY point cannot name a first or a last one.
5. **Where the refusals run.** The first three run in `named_refusal`
   (create, edit, load, MCP), because the hole list is in the parameters.
   The generator runs the same check again, for a caller that skipped the
   door. The lathe refusal needs the machine, so it runs in the generator.
6. **The seam.** The rule is `camcraft_motion::drilling::hole_order`
   (indices) and `in_order`. The three arms of `generate` put their hole
   list in that order before the kernel reads it. The kernels `drill`,
   `bore_started` and `thread_mill` do not change.
7. **The page.** The three kinds draw the Start point and Exit point
   groups with typed rows ("First hole X / Y", "Last hole X / Y"). A group
   that is switched on seeds the first (or the last) hole of the list, so
   the seed is a legal point.

**Why.**

- The issue names the rule "the list order, rotated to the start" as the
  example. It keeps the user's own list order between the two picked holes,
  so the path between is the one that the user already saw.
- "Inside the circle" is a rule that the user can check on the screen. A
  search distance would take a pick between two holes for one of them.
- The first traverse law (`operation/first_traverse.rs`) and the reach laws
  read the generated path, so they judge the new first hole with no change.

**Considered and dropped.**

- A nearest-hole rule with no limit: it moves a far pick to a hole without
  a note. The maintainer's rule is "a pick is a wish; it is never moved".
- A new shortest-path order between the picked holes: a new algorithm, and
  the issue does not ask for it.
- The order change inside the three kernels: three copies of one rule, and
  a change to `CycleParams`, `BoreParams` and `ThreadParams`.
- Picks on the lathe's axial and cross holes: no XY order exists there.

**Not done here.**

- The viewport pick that snaps to a hole (#3324 section 5) is not in this
  PR. The rows are typed, and MCP `edit_operation` sets the picks.

**Cost to change later.**

Low. A different order between the picked holes changes one function
(`hole_order`) and the three goldens `hole_order_*.cl`.
