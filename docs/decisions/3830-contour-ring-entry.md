## #3830 - Every bore-side ring takes the entry

**What was decided.** A bore-side, untabbed ring after another ring at the same level takes the entry on
its first level and the level transition below it, through `engage_level`. The pass link still feeds
straight where the keep-down proof holds. `ring_link` returns `false` when the entry must engage.

**Why.** The pass link lifted and plunged straight into solid at the plunge feed, with no note. Three
square bores at a 5 degree ramp posted 6 axial dives and `notes: []` on main.

**Traverse plane (review of #3834).** The move between the rings rides the pass link's plane, not the
level link's: clearance for `Link::Clearance`, the hop plane otherwise, also for a keep-down link that
failed its proof (`contour_link::ring_entry`, `Step::traverse`). This is what `ring_link` did before.
The tool crosses to the new lead-in at that plane before the descent, so the arm certifies its chord
from the lead-in. Without the crossing, an Auto-angle ramp fell back to an axial dive on every ring
(9 dives on three bores instead of 3). Pinned by `a_ring_after_another_is_ramped_from_its_own_lead_in_not_from_the_last_ring`.

**Dropped.** Outside rings (the wall arc and the pass link there have their own tests, and the user's
report was about holes). Changing the arm of keep-down schedules (it would change posted output).

**Cost to change later.** Outside multi-ring contours still plunge through the pass link: a follow-up.
