## #977 - Fanuc Lathe writes inch programs

**Decided.**

1. **The document's unit, converted where a word is written.** The
   script keeps the stream's mm for all its geometry and refusal checks
   (the half-micron lattice, the G76 range checks, C tolerances) and
   converts only in its word formatters. `programUnits = "mm inch"`;
   `postOptions.Units === "inch"` writes `G20` in the opening block in
   place of `G21`.
2. **Units and digits under G20.**
   - Lengths (X diameter, Z, I K, G76's first-block R, the tool note's
     D / R): / 25.4 at four decimals (0.0001 in, the inch least input
     increment; mm stays at three).
   - G99 feed per revolution: in/rev at four decimals (mm/rev also four).
   - Thread lead (G32 F, G76 F): inches at six decimals. An inch lead
     such as 13 TPI (0.076923 in) needs them, and a rounded lead error
     accumulates over the thread's length; mm stays at four.
   - No-point increment words (G76 P(k) Q(dmin) Q(first depth), G83 Q
     peck): whole 0.0001 in under G20, where G21 counts whole microns -
     the same "least increment, no decimal point" convention.
   - G96 S: surface feet per minute, `round(m/min / 0.3048)`
     (= m/min x 3.28084).
   - Never scaled: G97 S and G50 S (rpm), G83 P dwell (ms), G76's
     P(m)(r)(a) counts and angle, M / T / O words.
   - The bar feeder's PART LOAD advance comment says `<n> IN` under G20,
     so a bare mm number never sits in an inch program; the mm comment is
     unchanged.
3. **Refused by name, never guessed.**
   - A C-axis live-tool section on an inch program. Its F is a composite
     rate that counts C degrees as the linear unit. Whether a Fanuc-style
     control counts degrees as inches under G20 was not confirmed from a
     public manual, and a wrong guess would put the C part of the feed
     off by 25.4. The refusal says to post the document in mm.
   - A feed per revolution that rounds to F0 at the program's digits
     (under 0.00005 in/rev), and a G76 depth that rounds to zero
     increments. Either would be a word the control cannot cut. These
     checks run in mm too, but there they fire only on the same
     degenerate input (an F0 / Q0 the post used to write).
4. **Pinned.** `lathe_post_inch`: each turning stream of the lathe
   suites (facing, rough + finish, groove + part, axial drill, G76 and
   G32 threads) posts in mm byte-identical to its committed mm golden.
   The inch program is checked word by word against the mm one, then
   committed under `golden/nc/inch/fanuc-lathe/`. The exact inch G76
   words, SFM, the peck increment, the gate path of an inch document
   and the refusals have their own tests.

**Sources (public, behaviour only).** RS-274 / ISO 6983 (G20 / G21).
The publicly documented Fanuc 0i-T lathe vocabulary the post was
already specced from (G96 / G97 / G50, G99, the two-block G76 with P / Q
as no-point increments of the least input unit, 0.0001 in in inch
input). Published lathe programming guides showing inch two-line G76
examples with four-digit P / Q in 0.0001 in. No vendor post was read.

**Considered and dropped.**

- In/rev feeds at five or six decimals (as the Tormach LinuxCNC post
  writes five). 0.0001 in/rev is the inch least increment on the
  control family, so four decimals is the honest word; a feed that
  rounds to zero is refused instead.
- Converting the C-axis composite feed with degrees counted as inches.
  It is plausible, but unconfirmed, and it affects feed on a spindle, so
  it is refused (a follow-up can add it once a public source confirms it).

**Cost to change later.** The precision of any word is one formatter
(`fu`, `fr`, `fl`, `incr`) plus a goldens regeneration. Lifting the
C-axis refusal is `liveStart`'s inch branch plus an inch golden of
`c_axis_helix`.
