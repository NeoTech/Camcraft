## #1392 - a capped feed is judged on its posted word

**Decided:** fanuc-lathe.cps's C-axis live-tool G01 now computes the cap
(`capF`, the tighter of the linear-feed and C-rate caps in composite F
terms) on its own, and writes F through `capWord(v, cap)`: the one-decimal
word as before, unless that word is over the cap - then the word floored
to the 0.1 step at or under it. A word over the cap by no more than 1e-9
relative (the cap arithmetic's own float noise, e.g. `6000 * a / a`) is the
cap itself and is left as is.

**Why:** rounding to nearest could post up to 0.05 mm/min over a stated
machine rate. Flooring only when the word would exceed keeps every feed
that is not within half a step of its cap byte-identical, so no recorded
or golden program changed. The tolerance keeps a feed exactly at its cap
(e.g. a C-turning rapid capped to the C rate) from dropping a step on a
1e-13 artefact.

**Considered and dropped:**
- Flooring every capped word: changes goldens for feeds that never
  passed their cap, no safety gain.
- Heidenhain's drilling-cycle maximum check (`feed > machine.feedRateMax`,
  then Q206 at four decimals): the stream carries feeds on a 0.001 lattice
  (`cnc::serialize`), finer than no Q step, so the four-decimal word always
  equals the stream feed and cannot round over a maximum. Left unchanged.
- The fanuc / doosan-dnm uncapped rotary paths: on main they post no cap
  at all; the cap (and its rounding) is PR #951's code, so it is #951's
  to judge on the posted word.

**Cost to change later:** one helper in one post script.
