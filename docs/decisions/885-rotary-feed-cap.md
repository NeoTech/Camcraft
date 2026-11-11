## #885 - A feed that turns a rotary is capped at the machine's stated rates, in every post

**Decided:** one rule, `rotaryFeedCap` / `rotaryFeed` in the script
prelude (`crates/post/src/post/rotary_cap.rs`, appended to the prelude
every `.cps` sees), caps the F of every feed block that turns a rotary so
no rotary runs past the deg/min its machine file states (feed speed,
failing that rapid) and no linear part past the fastest stated feed. The
cap follows the reading of F each post declares:

- **composite** - Fanuc without tool centre point control (doosan-dnm.cps
  on a 4th axis that states no tilting direction; fanuc-lathe.cps C-axis
  live tooling, #860): the block is timed over sqrt(lin^2 + sum d^2),
  degrees counted as mm (FANUC 0i-F operator's manual, feedrate for
  rotation axes). fanuc-lathe.cps now calls the shared rule instead of
  its own copy; its goldens are unchanged.
- **path** - under G43.4 (fanuc.cps, doosan-dnm.cps on a tilting
  machine): F is the tool tip's speed relative to the work, the rotaries
  carried along (FANUC 30i-B operator's manual, tool center point
  control): F <= rate * tip length / |d|. With the tip still, F is read as
  the rotary rate itself, so F <= the lowest stated rate of the turning
  rotaries - safe under any reading in which each rotary moves at most F.
- **arcs** that turn a rotary use the path rule over the chord in the
  arc's plane: the chord is at most the length any reading of an arc's F
  is measured along (planar arc, helix, composite with the rotary), so
  the cap is never too loose.

A capped F is written floored to 0.1 mm/min (Math.round could post a hair
over the cap); a cap under 0.1 is refused by name. A feed that turns a
rotary from an angle the post never placed (a tilted program opening on a
feed) is refused by name when that rotary states a rate - its turn, and so
its rate, is unknown. An unstated rate caps nothing (the unstated-limit
law every post keeps), so every existing golden and every test machine
without rotary `speeds` posts byte-identical text.

**Why:** the issue asks for one helper, not a copy per post, matched to
the feed mode the post declares. The prelude is the only code every
script shares; the posts keep their own words and state (which letters
they write, which angles the control holds).

**Considered and dropped:** G93 inverse-time output - the exact answer
for simultaneous moves, but a new dialect feature per control and a
change to every tilted program, not a cap; capping the machine's linear
axes under G43.4 - their speed depends on the pivot length, which no
machine file states (the control clamps its own compensating motion); a
per-post copy of the lathe's code (what the issue rules out).
tormach.cps needs no cap: it writes A only on a G0 index and refuses an
A that changes inside an operation by name (#785), so none of its feeds
turns a rotary. haas, mazak, sinumerik, heidenhain and carvera refuse
every tilted record.

**Cost to change later:** the rule is one prelude function; a post that
later gains G93 swaps its call for its own inverse-time word.
