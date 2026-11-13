## #1529 - Top/Bottom of Model measure the parts, never the fixtures

**Decided.** The two model-referenced heights (`TopOfModel`,
`BottomOfModel`; the only references read from model bounds - stock,
work offset and selection read their own sources) resolve through a new
`setup_align::part_aabb_in`: the bounds of every model that is not a
fixture (no `FixtureDef` - which covers built-in vises, accessories such
as soft jaws, plates and chucks, and tagged STEP or mesh fixtures),
in the asking setup's frame (the #326 alignment still applies). The
"no model loaded" diagnostic reads the same set. The digest reads the
resolved Z through `resolve_height`, so adding or moving a fixture no
longer stales an op whose heights do not depend on it.

**Why "every part", not the op's own source.** The existing semantics
(#326's doc on `resolve_height`)
are the document's model bounds, not the selected source; switching to
the source would change the heights of every op whose source is a
sub-feature or stock and move goldens. The defect was only that the set
included fixtures, so the fix is the smallest one that matches the
issue's "Done means".

**Setup scoping.** Parts are document-wide in CamCraft - a setup holds
the same parts in its own frame; only fixtures carry a setup binding.
Leaving every fixture out (of any setup) therefore makes a vise in
Setup 2 unable to move Setup 1's heights, with no per-setup part list to
invent.

**Considered and dropped.**
- Also leaving out the "raw stock only" source model (#1094's
  `stock_only_model`): that verdict reads every operation of the
  program, which is too heavy for a per-height resolve called by panels,
  and it is not what the issue reports. Left as is.
- Changing `model::is_model` or `solid_aabb_in` themselves: they feed
  the browser, stock fit-to-models and the collision checker's notion of
  "a model"; the fix is local to the heights so the collision laws still
  see every fixture.
- Changing the "no model loaded" wording: the UI echo pins it verbatim.

**Cost to change later.** One function (`part_aabb_in`) decides the set;
narrowing it to the op's source or adding the stock-only skip is a
one-line filter plus a golden review.
