## #1449 - pre-drilled entry, first slice: 2D Pocket plunges down proved holes

**Decided:** a new approach flag, `links.approach.predrilled` (file key
`predrilled`, written only when on; a `0xC7` stamp byte only when on). On
the 2D Pocket kind it makes each level's entry a straight plunge, at the
plunge feed, down a hole a Drill operation ahead of the pocket in the same
setup made - instead of the asked helix, ramp or arc - wherever:

- the drill's stated diameter is at least the pocket tool's plus 0.1 mm
  radial clearance a side (`predrilled_entry::CLEARANCE`), and
- the stock the program ahead leaves (the prior column grid the drilling
  and thread-mill laws already read) holds no material over the tool's
  disc plus clearance above the level's Z. The hole counts to the deeper
  of that grid reading and the drill point's cone at the disc's radius
  (tip + r * cot of the half-angle), and
- the hole centre is a legal cutter centre and the at-depth link from it
  to the first pass start is proved (inside the cutter-centre area,
  crossing no planned pass).

To make that link short and provable the pass plan is re-seated on the
deepest proved hole: the nearest pass point to the hole becomes the first
pass start (a ring is rotated, an open zigzag pass is split into a forward
tail and a reversed head, the head re-entered on the column the tail just
cleared). Being the nearest point of all passes, the link crosses none.

Anything else keeps the asked entry and is said on the path by name: no
drill ahead, a drill too narrow (with the numbers), a hole the grid cannot
read or a posed 3+2 pocket, a level below the hole's proved depth ("open
only to Z a, above this level at Z b"), a hole outside the cutter area, and
a tool that is not centre-cutting.

The flag on any other kind is refused at the command door
(`PREDRILLED_POCKET_ONLY`), never silently dropped. It rides the existing
`EditOperationParams` (undoable) and MCP `edit_operation` / `add_operation`
params patch (`{"links": {"approach": {"predrilled": true}}}`), so no new
command or MCP tool was needed.

**Why:** the Prime Directive - a plunge into what the program believes is
air is a gouge or a broken tool if the hole is not there. The drill's own
moves only say a hole was asked; the prior grid says it was cut, and the
pocket goes Stale whenever the program ahead changes (the existing prior
digest of every unposed op), so a deleted or moved drill cannot leave a
cached plunge behind. The diameter check uses the drill's stated size
because the column grid cannot resolve a sub-cell clearance.

**Considered and dropped:**
- Linking from the hole to the plan's original first pass start: on a
  zigzag or ring plan that chord crosses planned passes (verify's Crossing
  law) - it was never provable, so the plan is re-seated instead.
- A typed point with an explicit "the hole exists" choice, holes in the
  model, carried stock from a previous setup, and the "pre-drilled only"
  refusal mode: later slices on #1449.
- Non-centre-cutting tools down a hole: the plunge law reads only the
  path's own cuts, so it would refuse; it needs the proved holes fed into
  `plunge_law` (a later slice). Such a tool keeps its helix/ramp, named.
- 2D Adaptive, Slot and the 3D roughing kinds: refused by name for now;
  the HSM plan bores its own entry and needs its own seat.
- A user-set clearance: 0.1 mm a side is fixed in this slice; it can
  become an approach field without a file break (serde-default law).

**Cost to change later:** the flag is one optional DTO field; widening it
to other kinds means lifting the door refusal and handing holes to their
ladders, no file migration.
