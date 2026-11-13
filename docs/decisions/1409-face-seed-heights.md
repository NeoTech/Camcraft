## #1409 - a new Facing's heights: Top of Stock down to Top of Model

**Decided:** `seed_facing` no longer takes `top`/`bottom` from the mill
fold. It seeds `top_z` = Top of Stock + 0 and `bottom_z` (the facing
plane) = Top of Model + 0, the same override the seed already makes for
the stepdown. Both new-op paths (MCP `add_operation` via `seed_params`,
GUI quick-add via `quick_params`) retype the same Face draft, so both
get it; retyping an existing op into a Facing gets it too.

The heights door (`depth_sane`) now lets a Facing's top equal its facing
plane, as it already did for Face. A part already at length then creates
and its generate refuses by name ("nothing to face"), instead of the door
calling a level window an upward cut.

**Why:** the fold's Bottom of Model (#214) is a mill depth's floor; for a
Facing it is the finished plane, and the ladder climbed through the
whole part. Facing removes the length allowance above the part, which
these two references name exactly.

**Considered and dropped:** a new "nothing to face" status (the
generator's existing `GenError::Empty` already reports it by name);
changing the fold's defaults (that would move every mill kind's seed);
rewording verify's saturated gouge depth (the issue's side note -
`gouge_mm` saturates at the nose radius by design; rewording it touches
every envelope kind's report, so it is left for its own issue if wanted).

**Cost to change later:** one seed function and one match arm.
