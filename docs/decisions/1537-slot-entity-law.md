## #1537 - A Slot on the whole finished part is judged by the part's law

**Decided:** `depth_law_of` gives every Slot `DepthLaw::Section` when the
job's source is the finished part (`part_solid`), not only a picked one. A
Slot whose source is a whole model (`SrcRef::Entity` naming the part) is
now verified against the part: a typed slot through a wall is a blocking
`Warning::Gouge` and holds the post. A Stock source (or an entity that is
the stock) is not part-solid, so it is cut unjudged as before.

**Why:** #1504's own expectation is "with nothing selected, the typed
endpoints still work, and the model gouge check still decides". With
`DepthLaw::None` nothing decided, and a gouge into the finished part passed
silently to the post (Prime Directive). The old contract ("the source is the
material to remove") is true for Stock, which `part_solid` already excludes.

**Changed pin:** `with_nothing_picked_the_typed_endpoints_rule_as_before`
pinned the unsafe Entity case as "no gouge"; it now keeps only the Stock
case, and the Entity case moved to
`a_whole_part_slot_through_the_wall_is_convicted_and_holds_the_post`
(with a clean whole-part slot inside the pocket as the over-conviction guard).

**Considered and dropped:** doing Circular in the same arm. Circular with
a whole-model source is the same question, but its existing tests and uses
(bosses, circular pockets on a whole model) need their own read; filed as a
separate issue so this P1 stays minimal.

**Cost to change later:** one match arm.
