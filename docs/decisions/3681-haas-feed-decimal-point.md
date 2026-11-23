## #3681 - Haas feed words carry a decimal point

**What was decided.** `assets/posts/haas.cps` writes each F word with a
decimal point, in mm and in inch (`F600.`, `F2.`, `F23.62`). The feed
formatter `ff()` puts its result through `pt()`, the same helper that the
length words use (#1836). The zero-feed literal is `0.`. The mm feed stays
rounded to a whole mm/min, as before.

Two edge cases changed with it:

- A positive feed that rounds to zero (under 0.5 mm/min, or under
  0.005 in/min) keeps its fraction (mm at three decimals, inch at four).
  Before, it posted `F0`.
- A positive feed below that step too (under 0.00005 in/min in an inch
  program) is refused by name through `msg()`. It is never posted as `F0.`.

The words that stay integers: dwell P (ms), S, T, H, D, G10 P/L2, G187 P.

**Why.** Haas Setting 77 (Scale Integer F) scales an F word that has no
decimal point. Source: Haas Mill Operator's Manual 96-8000 (2009),
Settings, "77 - Scale Integer F". The reporter cited rev V of this manual.
A web check of the same manual agrees: at the default value, an mm control
reads an integer F with three implied decimals, so F600 is 0.6 mm/min. The
manual recommends a decimal point on each F word. The post cannot know the
value of Setting 77 on the machine. With the point, each value of the
setting reads the same feed.

**Considered and dropped.**

- Higher feed precision in mm (for example one decimal). Dropped: it
  changes more text than the defect needs. The precision question is a P7
  follow-up issue.
- A fix in all 21 other post scripts in this PR. Dropped: each control
  has its own rule for an integer F. A follow-up issue holds the table.
- Refusing a feed that rounds to zero in mm. Dropped: the fraction is a
  correct feed, so the post writes it.

**Cost to change later.** One function in `haas.cps` and a regeneration
of the Haas goldens.
