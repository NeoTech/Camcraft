## #2769 - Fanuc Manual Guide i blank: reconcile PR #3306 with merged PR #3307

**What was decided.**

- Main has one blank writer and one PARAM contract: the ones from PR #3307
  (`StockShape`, `StockShapeBottom`, `StockShapeDiameter`, and
  `manualGuideBlank()` in `fanuc.cps`). PR #3306 keeps that writer and that
  contract without change.
- PR #3306 drops its own contract: the `StockBlank` / `StockBlankZBottom`
  PARAMs, the `stockBlank = true` manifest flag, the engine filter
  (`post::stock_blank`) and `posting_stock_blank.rs`.
- PR #3306 adds only what main does not have:
  - the option help names the Series 30i background-simulation unit limit
    ("ignores G20/G21; confirm the control unit setting");
  - a pin that the shape PARAMs keep the line map and the trace record
    indices on the input stream;
  - a pin for a bar whose work origin is above its top face (main refuses
    it with `BAR END FACE BELOW THE WORK ORIGIN`).

**Why.** The review of PR #3306 asked for one writer and one contract. PR
#3307 is on main and is reviewed. Main's contract needs no engine filter:
the Klartext post lists the three shape PARAMs and ignores them. With no
filter, the record indices cannot move, so the index fix of PR #3306 is not
needed. The test keeps that invariant pinned. Main's writer already refuses
a negative `K`.

**Dropped.**

- Keep the `StockBlank` contract and change main to it. That reverts
  reviewed work on main and changes the stream of every post.
- Port the PR #3306 refusals for an inverted machine axis and a flipped
  WCS. The engine mirrors the stock box and the origin together
  (`post::invert`, `post::wcs_flip`), so main's `I J K` are correct in the
  control frame. A refusal would block a correct block.
- Port the "bar not round" refusal. Main reads the true diameter from
  `StockShapeDiameter`, not from the bounds.

**Cost to change later.** Small. The help text is one string in
`fanuc.cps`.
