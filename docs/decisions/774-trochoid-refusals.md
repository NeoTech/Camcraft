## #774 - Trochoidal slot refusals: loop cap, microlift bound, helix-turn cap

**Microlift bound: the clearance plane, not a stated maximum.** A
trochoidal slot's microlift height may be at most the clearance plane's
height above the cut top (`clearance - top`, on resolved heights); a
taller lift is refused by name at command creation, parameter edit and
source change (the commands' heights door), and the generator refuses the
same law as a backstop.

- *Why:* the lifted return is a feed move, and the clearance plane is the
  top of the envelope the operation promises to stay under. That is the
  bound with a safety meaning: above it the return would feed through
  space the job never claimed. Measuring from the top (not from the first
  level) is the stricter of the two and needs no stepdown arithmetic to
  state.
- *Considered and dropped:* a fixed stated maximum (e.g. 5 mm) in the
  world-free `is_sane`. It would be arbitrary (no machining reason picks
  5 over 2 or 10, and it ignores units and job size), it would still let
  a lift cross a low clearance plane, and `is_sane`'s refusal is the
  generic "parameters are not usable" message, not a named one. Bounding
  to the retract plane was dropped too: feeding above the retract plane is
  slow, not unsafe.
- *Where it lives:* the rule is `OpParams::microlift_refusal(clearance,
  top)` in `operation/types/sane.rs` (world-free, so it is checked where
  the rest of the parameter sanity is), asked with resolved heights by
  `operation::microlift_sane`, which `commands::operation::gates::
  heights_gate` calls. `is_sane` cannot answer it alone because the
  heights are references (From: Stock top, From: Selection ...) that only
  resolve with the world in hand.
- *Cost to change:* one comparison in `microlift_refusal` and its twin in
  `TrochoidParams::validate`.

**Loop-cap refusal names its cause.** When every rung of the step ladder
needs more loops than `MAX_LOOPS` (20000), the refusal says the forward
step is too small, naming the step tried, the centre line length and the
cap, instead of an engagement reading that was never taken ("inf deg at
(0,0)"). When the longer rungs were measured and failed the engagement
limit and the shorter ones hit the cap, the engagement refusal stands and
adds the cap as the reason no shorter step was tried.

**Helix-turn cap is a refusal.** A level whose bore helix would need more
than `MAX_HELIX_TURNS` (1024) turns is refused by name (drop, helix
radius, the cap, and the fix: a smaller stepdown or a wider loop band),
instead of the old `unwrap_or(1.0)` that cut the whole drop in one turn.
`emit_level` is now fallible for it.

**Refactor for the 400-line law:** `contained` and `arc_samples` moved
from `trochoid_slot.rs` to `trochoid_loops.rs` (loop-layout geometry,
unchanged) to keep `trochoid_slot.rs` under 400 lines.
