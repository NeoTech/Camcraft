## #3694 - The Haas feed word keeps its decimals, so a tap feed gives the exact pitch

**Decided.** `assets/posts/haas.cps` writes every F word with its fraction:
mm/min at three decimals, in/min at four, trailing zeros trimmed, the point
always written (`F600.`, `F34.5`, `F133.2`, `F1.3583`). One formatter (`ff`)
serves every Haas feed, not only G84. A G84 block whose written words
(F and S) give a pitch more than 0.01 percent off the stream's pitch
(feed / rpm) is refused by name through `msg()`. In the app the counted
warning blocks the write, so no partial file is written there; the
standalone `post` binary writes the file with the warning, as for every
other refusal of this post. This also settles #3692 (the mm feed precision) for Haas.

**Why.** The control cuts the pitch F / S. The stream carries the tap feed
pitch x rpm at 0.001 mm/min (`crates/cl/src/cl/text.rs`), and the old
`Math.round` threw away the fraction: M1.4 x 0.3 at S115 (34.5 mm/min)
posted `F35.`, a 1.45 percent pitch error. Three decimals is the step of
the stream and of the post's own length words, so an mm feed posts exactly
as the stream states it. Four in/min decimals is the step of the post's
inch length words.

Source for the feed format: Haas Mill Operator's Manual 96-8000 (Settings,
"77 - Scale Integer F"), already cited by #3681: an F word with a decimal
point is read as written, in the program unit (mm/min under G21, in/min
under G20). The point is always written, so Setting 77 has no effect.
The number of decimals the control accepts in F was not checked against a
control; three (mm) and four (inch) are the least increments of the
post's other words.

**Worst case of the residual pitch error** (feed rounding only, exact S):

- mm: the stream step is 0.001 mm/min, the same as the output step, so
  the written F equals the stream feed. Error 0. Bound in theory:
  0.0005 / F; above 0.01 percent only when F < 5 mm/min.
- inch: rounding to 0.0001 in/min is at most 0.00005 in/min
  (0.00127 mm/min). The error is above 0.01 percent only when the feed
  is below 0.5 in/min (12.7 mm/min). Example: 2 mm/min (0.07874 in/min,
  written `F0.0787`) is 0.05 percent off and is refused. The issue's
  examples in an inch program: 34.5 mm/min writes `F1.3583` (0.0024
  percent), 133.2 mm/min writes `F5.2441` (0.0001 percent): both post.
- S: the post writes S as a whole rpm. A fractional stream rpm moves the
  pitch too (34.5 mm/min at 115.4 rpm writes S115: 0.35 percent off), so
  the check uses the written S, and such a tap is refused.

**Considered and dropped.**

- Three decimals for the G84 feed only, whole mm/min elsewhere: two
  derivations of one word, and #3692 shows the same loss on slow G1 feeds.
- Refuse each tap whose feed is not a whole mm/min (option 2 of the
  issue): it refuses common metric taps (M1.4 at S115) that the control
  can cut exactly.
- Keep two in/min decimals: the error is above 0.01 percent for each tap
  feed under 50 in/min, which is most taps.
- Change the other six mill posts here: they are #3699.

**Cost to change later.** One function (`ff`) and one check
(`tapPitchHolds`) in `haas.cps`; the pins are in
`crates/post/tests/haas_feed_decimal_point.rs`.
