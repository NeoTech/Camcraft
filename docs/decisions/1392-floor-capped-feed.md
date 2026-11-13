## #1392 - a capped feed is judged on its posted word

**Decided:** fanuc-lathe.cps's C-axis live-tool G01 now computes the cap
(`capF`, the tighter of the linear-feed and C-rate caps in composite F
terms) on its own, and writes F through `capWord(v, cap)`: the one-decimal
word as before, unless that word is over the cap - then the word floored
to the 0.1 step at or under it. (A 1e-9 relative tolerance first allowed
here was dropped on review, 2026-11-13 - see the review note below.)

**Why:** rounding to nearest could post up to 0.05 mm/min over a stated
machine rate. Flooring only when the word would exceed keeps every feed
that is not within half a step of its cap byte-identical, so no recorded
or golden program changed.

**Considered and dropped:**
- Flooring every capped word: changes goldens for feeds that never
  passed their cap, no safety gain.
- Heidenhain's drilling-cycle maximum check (`feed > machine.feedRateMax`,
  then Q206 at four decimals): the stream carries feeds on a 0.001 lattice
  (`cnc::serialize`), finer than no Q step, so the four-decimal word always
  equals the stream feed and cannot round over a maximum. Left unchanged.
- The fanuc / doosan-dnm rotary paths: their cap (and its rounding) is
  #1395's code (merged), judged there on the posted word.

**Review note (2026-11-13, M3's safety review of PR #1397, fixed author-side):**
- *Merge with #1395:* `liveMove` keeps #1395's `rotaryFeedCap`; its
  uncapped value (`rotaryFeedCap(Infinity, ...)`) is the cap `capWord` floors
  under, as M3 resolved it.
- *Item 1, F0:* a word under the 0.1 mm/min step is refused by name and
  not written - one refusal when the machine-rate cap leaves it there,
  another when the planned feed itself is that small. Considered: posting
  F0.1 (the old behaviour) - over the cap, so dropped. The F words of C
  sections are mm/min only (inch C sections are refused), so the judgement
  is in mm, as `rotaryFeedWord` judges the mill's.
- *Item 2 = #1398:* the cap is measured between the POSTED words - X read
  back from its rounded diameter word, Z origin-relative at 0.001, C at
  0.001 - the same rule as `rotaryFeed`'s `q()`/`org`. The planned F is
  still scaled by the raw composite length, so a feed away from its cap
  posts byte-identically (no golden changed). Measuring a C or Z word's
  rounding only shows on a stream off the 0.001 lattice (the engine reads
  such a stream; the app never writes one); a test feeds one by hand so
  those lines are pinned too.
- *Item 3, tolerance:* dropped rather than pinned. Without it a feed
  exactly at its cap still keeps its word in every pinning test and every
  golden; float noise below the cap can only floor a word one step lower,
  which is on the safe side. Pinning it would need a cap whose noise
  lands just over a step - test arithmetic, no safety value.
- *Equal-word move:* a move whose posted words do not change still posts
  as before (uncapped, its F unchanged) - no change of behaviour taken here.

**Cost to change later:** one helper in one post script.
