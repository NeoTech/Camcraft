## #796 - Tormach microARC 4: wrapped (simultaneous) A in the PathPilot post

Sources: the four PathPilot Hub readings mkinney recorded on issue #796
(2026-11-17, PathPilot v2.14.4, 770MX SIM, 4th axis on, "4" 4th Axis"):

1. A is a plain linear axis in degrees, no rollover: from `G0 A350`,
   `G1 A10 F360` turned back 340 degrees and read 10.000.
2. G94 on a combined move: `G1 X4 A180 F20` took about 12 s - F is the
   X Y Z distance only, A follows in the same time.
3. G93 is accepted and timed: `G93 G1 X4 A180 F5` took about 12 s.
4. Pure A in G94: `G1 A180 F360` took about 30 s - F is degrees per minute.

Plus the LinuxCNC G-code reference (G93/G94, F). No Fusion/HSMWorks post or
output was consulted.

- **Decided: the turning XYZA feed is G93 inverse time.** F = 1 / t, t in
  minutes = max((tip path relative to the part + one rounding step) /
  programmed feed, |posted dA| / stated A rate), floored to 0.001.
  **Why:** reading 2 shows G94 times a combined move by X Y Z alone, so a
  mostly-rotary move would let A run as fast as its tiny XYZ distance
  allows, and its time would not be the cutting feed relative to the part
  (the part turns under the tool). G93 lets the post state the time
  exactly (reading 3). The path bound uses the rounded XYZ/A words the
  control interpolates, transformed back into the part frame; a turning
  trajectory can be longer than the straight stream segment. Floored F and
  the rounding allowance slow a move.
  **Dropped:** G94 with the #885 `rotaryFeed` "path" cap - it holds the A
  rate but times the move by the machine XYZ distance, which on a wrapped
  op is not the tip's speed over the part. `rotaryRate` (the #885 reader of
  the stated A rate) is reused; its cap function is not.
- **Decided: an A-only feed stays G94, F in degrees per minute** (reading 4),
  under the same time law; a feed that does not turn A is today's G94 move.
  G94 is restated before any motion that is not a G93 feed and at every
  section end, so cycles, arcs, probes and the next operation never run in
  G93. G93 rides the G1 line that enters it; each G93 block carries F.
- **Decided: A is cumulative.** Each target is the stream's angle's
  equivalent nearest the held one (179 -> 181, 359 -> 361), from reading 1:
  the control never wraps or takes the short way. The stated travel bounds
  the count, refused by name past it ("A counts on through 360").
- **Decided: A is reset between operations, never carried.** Every section
  already restates its first A as -180..180 after `G53 G0 Z0` (#785); a
  wrapped op that ended at A300 unwinds there before the next op.
- **Decided: split a turning move until it stays on its segment.** The
  machine moves X Y Z in a straight line while A turns linearly, so the tip
  relative to the part bows off the stream's straight segment. The post
  splits each turning move into 2^k equal pieces until the bow, sampled at
  1/4, 1/2 and 3/4 of every piece, is within 0.005 mm (0.0002 in); more
  than 256 pieces is refused by name. Rapids that turn A are split the same
  way and carry no F.
- **Decided: the microARC 4 file states `max_normal_speed` 360 deg/min for
  A.** **Why:** it is the rate the Hub ran as asked (readings 1 and 4); no
  Tormach page found states a feed rate we could cite with confidence. It is
  conservative: a faster real table only runs these programs slower than it
  could. A machine file with no A rate is refused by name ("states no A
  feed rate"). **Cost to change:** one number in
  `assets/machines/tormach-770mx-microarc4.mch`; the goldens regenerate.
- **Kept: the -360..360 travel.** The Hub runs show no soft limit inside it;
  PathPilot's real A limits are an open Hub check. A wrap that would count
  past it is refused, not wrapped back.
- **Not exercised:** a real 770MX + microARC 4 cut. The PR lists the Hub
  check to run first.

- **Review fix: bound the posted trajectory, not its endpoint chord.**
  With posted XYZ interpolated as `m(u)` and posted A as `theta(u)`, the
  part-relative trajectory is `q(u) = R(theta(u)) m(u)`. Its speed is
  bounded by the midpoint speed plus one quarter of a uniform bound on
  `|q''|`: `2 |dtheta| |dm_yz| + dtheta^2 max(|m0_yz|, |m1_yz|)`.
  Integrating `|u - 1/2|` gives the quarter factor. XYZ rounding and angle
  rounding are included before evaluating this bound; the existing one-step
  allowance remains conservative. **Dropped:** the stream chord plus a
  rounding allowance, which misses the curved path even when its two
  endpoints coincide. Independent dense interpolation tests pin feed and
  A rate against the posted words.
- **Review fix: bound the wrapped golden's G93 F in inverse time across platforms.**
  Keep line count, word order, modes, XYZ and ordinary feeds exact. Allow
  at most 0.001 degree per endpoint in G0/G1 A, and on turning G93 F allow
  at most the inverse-time shift caused by that 0.002-degree `dA` rounding
  spread at 360 deg/min (`|1/got - 1/want| <= 0.002 / 360` min) plus
  0.001 min^-1 for three-decimal flooring (`0.001 + (got + 0.001) * (want + 0.001) * (0.002 / 360)`).
  This allows `0.0012` on the slowest `F5.335` line and `~1.50` on `F518.731`
  (covering the Windows spread `0.049`..`0.749` without relaxing `F5.335`).
  Turning rapids in `turnMove` also restate `G94` before `G0` so no rapid
  runs in `G93`. Comparator and post pins reject values outside the
  inverse-time bound, `F5.335` speed errors, turning rapids in `G93`,
  changed motion words, changed XYZ, missing words and extra lines.
  **Dropped:** a flat `0.04` F allowance (smaller than the real inverse-time
  spread on faster lines) or a flat `0.75` allowance (14% error on `F5.335`).
