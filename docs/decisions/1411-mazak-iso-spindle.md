## #1411 - Mazak post: ISO spindle words (S.. M3 / M4, M5) instead of MOTON / MOTOFF

**Decided:** `assets/posts/mazak.cps` starts the spindle with `S<rpm> M3`
on one block (`S<rpm> M4` when the section states `PARAM MillSpindle
reverse`, a left-hand-cut mill, #988 - the post now declares
`millSpindleReverse = true` like `fanuc.cps`, and resets the direction at
each `OperationName`), and stops it with `M5` at every section end and in
a touch probe's preamble. The start keeps its place in the preamble: after
`T.. M6` and `G43 H..`, before the coolant word and the first move - the
same order as the Fanuc, Haas, Tormach and Doosan posts. The Mazak
goldens (`crates/app/tests/golden/nc/mazak/`, `.../inch/mazak/`,
`synth_mazak.nc`, `crates/post/tests/recorded/golden/mazak/`) change only
on those lines: `MOTON S<n>` becomes `S<n> M3`, `MOTOFF` becomes `M5`.

**Why:** the post's own header names public RS-274 / ISO 6983 as its
grammar, where a word is one address letter and a number; `MOTON` and
`MOTOFF` are not words of it, and `MOTON` states no direction ahead of a
G84 right-hand rigid tap, which needs the spindle clockwise. No public
document was found that gives `MOTON` / `MOTOFF` as Mazak's spindle
words (the issue's question), and the clean-room protocol allows only
public standards and our own posts as reference, so the standard words
are the answer. The rest of the post already follows the Fanuc-family
EIA/ISO words (G43 H, G84, G10 L2, #5061), so M3 / M4 / M5 are the
consistent choice.

**Considered and dropped:**
- Keeping `MOTON` / `MOTOFF` until a document is found: it leaves every
  Mazak program with a non-standard first spindle block, a P1 by the
  scale (wrong posted output) with no source behind it.
- Writing `M3` / `M5` but leaving `millSpindleReverse` undeclared: the
  engine would then refuse a left-hand-cut thread mill by name on Mazak.
  Safe, but the post already has everything the M4 case needs, and the
  same three lines Fanuc carries make it serve the case.
- A Mazatrol (conversational) form: out of scope - this post writes the
  EIA/ISO program only.

**Not exercised:** the output has not been run on a real Mazak control;
it is pinned by `crates/post/tests/mazak_spindle_words.rs` and the
byte-exact goldens only.

**Cost to change later:** if a public Mazak document ever gives other
spindle words, the change is the two `block([...])` calls in
`startPreamble` / `endPreamble` and the same golden lines again.

No other built-in post wrote `MOTON` / `MOTOFF` (checked over
`assets/posts/*.cps`), so no follow-up issue is filed.
