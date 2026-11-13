## #938 - the verifier judges "upright" with one stated angular tolerance

**Decided.** The static verifier (`crates/sim/src/verify/`) no longer
branches on the exact `camcraft_cl::is_vertical`. One predicate,
`verify::upright::is_upright`, judges a pose upright when its tilt off +Z
is at most `UPRIGHT_TILT_RAD = 1e-9` rad (about 5.7e-8 degrees), measured
as `hypot(ax, ay) <= tol * |axis|` with `az > 0`. Every verify law that
branched on "tilted" now reads it: reach (c) and its mid-turn sweep (c3),
the fixture band and rapid laws (c2)/(d2), the posed steel sweep, the
mesh-steel law, the solid law, the posed contact law, the posed-rapid
second reading and the all-air verdict. A tilt inside the tolerance takes
the prismatic branch, bit for bit; anything above it, a downward or
sideways axis, a zero vector or a non-finite component takes the tilted
(enclosing) reading. Law (c) also reports one `ShankHit` per move and
verdict: a leaning shank and holder in the same stock no longer report
the same hit twice. The one existing pin that recorded the duplicate
(`crates/app/tests/tool_body_bands.rs`, an upright fat shank under a
modelled body: shank and holder groups both convict) now expects the
single report; the conviction itself is unchanged.

**Why this value is safe.** Judging a tilt of θ as upright misplaces the
tool stack by at most L·sin θ sideways (L the stack length above the tip)
and a band's rim by at most R·sin θ vertically. At θ = 1e-9 rad a 1 m
stack moves 1e-6 mm and a 100 mm holder disc dips 1e-7 mm: four orders
below the tightest gouge tolerance a setup states (0.01 mm) and at or
under the laws' own kerf margins (1e-6 mm). Rounding noise of a composed
rotation is ~1e-16 per component; #938's 1e-10 degrees is 1.7e-12 rad,
three orders inside. A tilt anyone commands (0.001 degrees = 1.7e-5 rad)
is four orders outside, and the test pins a tilt of 1.01e-9 rad - just
above the tolerance - as still tilted.

**Why in the verifier, not in `is_vertical`.** `is_vertical` is exact on
purpose: the CL dump and the post take the prismatic code path on it, so
goldens and posted G-code never learn float drift. Loosening it would
change what is posted for a near-vertical pose; the verdict question is a
different one and gets its own predicate. No golden or posted output
changes.

**Considered and dropped.**
- Loosening `camcraft_cl::is_vertical` itself: changes post output and
  goldens, and the issue asks for none.
- A tighter, smaller-than-noise bound (1e-12): too close to accumulated
  rotation noise on chained setups; 1e-9 keeps margin on both sides.
- Replacing the tilted reading's "bottom dropped by the whole band
  radius" with the exact R·sin θ dip: would remove the cliff entirely but
  changes every tilted verdict; it is a separate, larger change to the
  enclosure law.
- Folding a move's shank and holder hits into the single worst one (part
  over stock): would drop a `part: false` report beside a `part: true`
  one and change existing verdicts; only the identical duplicate goes.

**Left as is.** The stock simulation (`stock_sim`) and the motion
generators (`motion::multiaxis`, `VERT_EPS = 1e-12` in `avoid.rs`) keep
their exact upright tests: they shape geometry, not verdicts, and the
generator reading a noise pose as leaning is the conservative side.
`avoid.rs`'s comment ("as strict as the is_vertical the verifier branches
on") is now one notch stale; it is still the stricter of the two.

**Cost to change later.** One constant and one function in
`crates/sim/src/verify/upright.rs`; the pins are in
`crates/sim/tests/tilt_tolerance.rs`.
